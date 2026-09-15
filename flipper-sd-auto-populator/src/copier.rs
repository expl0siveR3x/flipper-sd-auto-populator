use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};

use walkdir::WalkDir;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::profile::{CopyMode, Mapping, Profile};

#[derive(Debug, Clone)]
pub(crate) enum CopyEvent {
    BackupStarted,
    BackupProgress {
        files: usize,
    },
    BackupDone,
    BackupError(String),
    MappingStarted {
        index: usize,
        total: usize,
        dest: String,
        mode: CopyMode,
    },
    FileCopied {
        rel: String,
    },
    FileSkipped {
        rel: String,
    },
    FileError {
        rel: String,
        error: String,
    },
    MappingDone {
        copied: usize,
        skipped: usize,
        errors: usize,
    },
    Done {
        total_copied: usize,
        total_skipped: usize,
        total_errors: usize,
    },
}

#[derive(Clone)]
pub(crate) enum JobResult {
    Ok,
    Cancelled,
}

pub(crate) struct CopyJob {
    pub profile: Profile,
    pub target: PathBuf,
    pub backup_before: bool,
    pub backup_dir: PathBuf,
}

pub(crate) fn run_copy(
    job: CopyJob,
    tx: mpsc::Sender<CopyEvent>,
    cancel: Arc<AtomicBool>,
) -> JobResult {
    if job.backup_before
        && let Err(e) = create_backup(&job.target, &job.backup_dir, &tx, &cancel)
    {
        let _ = tx.send(CopyEvent::BackupError(e.to_string()));
        return JobResult::Cancelled;
    }

    let total_mappings = job.profile.mappings.len();
    let mut total_copied = 0;
    let mut total_skipped = 0;
    let mut total_errors = 0;

    for (i, mapping) in job.profile.mappings.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return JobResult::Cancelled;
        }

        let _ = tx.send(CopyEvent::MappingStarted {
            index: i + 1,
            total: total_mappings,
            dest: mapping.dest.clone(),
            mode: mapping.mode.clone(),
        });

        let (copied, skipped, errors) = apply_mapping(mapping, &job.target, &tx, &cancel);
        total_copied += copied;
        total_skipped += skipped;
        total_errors += errors;

        let _ = tx.send(CopyEvent::MappingDone {
            copied,
            skipped,
            errors,
        });
    }

    let _ = tx.send(CopyEvent::Done {
        total_copied,
        total_skipped,
        total_errors,
    });

    JobResult::Ok
}

fn apply_mapping(
    mapping: &Mapping,
    sd_root: &Path,
    tx: &mpsc::Sender<CopyEvent>,
    cancel: &Arc<AtomicBool>,
) -> (usize, usize, usize) {
    let source = &mapping.source;
    if !source.exists() {
        let _ = tx.send(CopyEvent::FileError {
            rel: String::new(),
            error: format!("Source does not exist: {}", source.display()),
        });
        return (0, 0, 1);
    }

    let dest = sd_root.join(&mapping.dest);
    if dest == sd_root || mapping.dest.trim().is_empty() {
        let _ = tx.send(CopyEvent::FileError {
            rel: String::new(),
            error: "Destination resolves to SD root — refusing".into(),
        });
        return (0, 0, 1);
    }

    if mapping.mode == CopyMode::ExactMirror {
        if dest.exists()
            && let Err(e) = fs::remove_dir_all(&dest)
        {
            let _ = tx.send(CopyEvent::FileError {
                rel: String::new(),
                error: format!("Failed to wipe destination: {}", e),
            });
            return (0, 0, 1);
        }
        let _ = fs::create_dir_all(&dest);
    }

    let mut copied = 0;
    let mut skipped = 0;
    let mut errors = 0;

    let walker = WalkDir::new(source).into_iter();
    for entry in walker.filter_map(|e| e.ok()) {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }

        let file_path = entry.path();
        let rel = file_path
            .strip_prefix(source)
            .unwrap_or(file_path)
            .to_string_lossy()
            .into_owned();

        let target = dest.join(&rel);

        if mapping.mode == CopyMode::OnlyIfMissing && target.exists() {
            let _ = tx.send(CopyEvent::FileSkipped { rel });
            skipped += 1;
            continue;
        }

        if let Some(parent) = target.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            let _ = tx.send(CopyEvent::FileError {
                rel: rel.clone(),
                error: e.to_string(),
            });
            errors += 1;
            continue;
        }

        match fs::copy(file_path, &target) {
            Ok(_) => {
                let _ = tx.send(CopyEvent::FileCopied { rel });
                copied += 1;
            }
            Err(e) => {
                let _ = tx.send(CopyEvent::FileError {
                    rel,
                    error: e.to_string(),
                });
                errors += 1;
            }
        }
    }

    (copied, skipped, errors)
}

fn create_backup(
    sd_root: &Path,
    backup_dir: &Path,
    tx: &mpsc::Sender<CopyEvent>,
    cancel: &Arc<AtomicBool>,
) -> io::Result<()> {
    let _ = tx.send(CopyEvent::BackupStarted);

    fs::create_dir_all(backup_dir)?;

    let timestamp = chrono_filename();
    let zip_path = backup_dir.join(format!("backup_{}.zip", timestamp));
    let zip_file = fs::File::create(&zip_path)?;
    let mut writer = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    let mut file_count = 0;

    for entry in WalkDir::new(sd_root).into_iter().filter_map(|e| e.ok()) {
        if cancel.load(Ordering::Relaxed) {
            writer.finish()?;
            let _ = fs::remove_file(&zip_path);
            return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled"));
        }

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let rel = path.strip_prefix(sd_root).unwrap_or(path).to_string_lossy();

        let mut f = fs::File::open(path)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;

        writer.start_file(rel.to_string(), options)?;
        writer.write_all(&buffer)?;

        file_count += 1;
        if file_count % 50 == 0 {
            let _ = tx.send(CopyEvent::BackupProgress { files: file_count });
        }
    }

    writer.finish()?;
    let _ = tx.send(CopyEvent::BackupDone);
    Ok(())
}

fn chrono_filename() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("flipper_copy_test_{tag}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src/sub")).unwrap();
        fs::write(dir.join("src/file1.txt"), "hello").unwrap();
        fs::write(dir.join("src/sub/file2.txt"), "world").unwrap();
        dir
    }

    #[test]
    fn only_if_missing_copies_new() {
        let dir = setup_test_dir("copies_new");
        let sd = dir.join("sd");
        fs::create_dir_all(sd.join("subghz")).unwrap();

        let mapping = Mapping {
            source: dir.join("src"),
            dest: "subghz".into(),
            mode: CopyMode::OnlyIfMissing,
        };

        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let (copied, skipped, errors) = apply_mapping(&mapping, &sd, &tx, &cancel);
        assert_eq!(copied, 2);
        assert_eq!(skipped, 0);
        assert_eq!(errors, 0);

        let events: Vec<_> = rx.try_iter().collect();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, CopyEvent::FileCopied { .. }))
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_if_missing_skips_existing() {
        let dir = setup_test_dir("skips_existing");
        let sd = dir.join("sd");
        fs::create_dir_all(sd.join("subghz/sub")).unwrap();
        fs::write(sd.join("subghz/file1.txt"), "already here").unwrap();

        let mapping = Mapping {
            source: dir.join("src"),
            dest: "subghz".into(),
            mode: CopyMode::OnlyIfMissing,
        };

        let (tx, _rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let (copied, skipped, _errors) = apply_mapping(&mapping, &sd, &tx, &cancel);
        assert_eq!(copied, 1);
        assert_eq!(skipped, 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn exact_mirror_wipes_and_copies() {
        let dir = setup_test_dir("exact_mirror");
        let sd = dir.join("sd");
        fs::create_dir_all(sd.join("subghz")).unwrap();
        fs::write(sd.join("subghz/old_file.txt"), "delete me").unwrap();

        let mapping = Mapping {
            source: dir.join("src"),
            dest: "subghz".into(),
            mode: CopyMode::ExactMirror,
        };

        let (tx, _rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let (copied, _skipped, _errors) = apply_mapping(&mapping, &sd, &tx, &cancel);
        assert_eq!(copied, 2);
        assert!(!sd.join("subghz/old_file.txt").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn backup_creates_zip() {
        let dir = setup_test_dir("backup");
        let sd = dir.join("sd");
        fs::create_dir_all(sd.join("subghz")).unwrap();
        fs::write(sd.join("subghz/test.txt"), "data").unwrap();

        let backup_dir = dir.join("backups");
        let (tx, _rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let result = create_backup(&sd, &backup_dir, &tx, &cancel);
        assert!(result.is_ok());

        let zips: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "zip"))
            .collect();
        assert_eq!(zips.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }
}
