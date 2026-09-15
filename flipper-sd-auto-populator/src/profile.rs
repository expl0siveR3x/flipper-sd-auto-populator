use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Profile {
    pub name: String,
    pub is_default: bool,
    pub mappings: Vec<Mapping>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Mapping {
    pub source: PathBuf,
    pub dest: String,
    pub mode: CopyMode,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub(crate) enum CopyMode {
    ExactMirror,
    OnlyIfMissing,
}

impl CopyMode {
    pub fn description(&self) -> &str {
        match self {
            CopyMode::ExactMirror => {
                "Deletes all files in the destination path, and then copies all files/folders from the source to the destination."
            }
            CopyMode::OnlyIfMissing => {
                "Only copies the files from the source that the destination does not already contain"
            }
        }
    }

    pub fn label(&self) -> &str {
        match self {
            CopyMode::ExactMirror => "Exact Mirror (wipe & copy)",
            CopyMode::OnlyIfMissing => "Only If Missing",
        }
    }
}

impl Profile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_default: false,
            mappings: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.trim().is_empty() {
            errors.push("Profile name is empty".into());
        }
        for (i, m) in self.mappings.iter().enumerate() {
            if m.source.to_string_lossy().trim().is_empty() {
                errors.push(format!("Mapping {}: source is empty", i + 1));
            } else if m.source.is_absolute() && !m.source.exists() {
                errors.push(format!(
                    "Mapping {}: source path does not exist: {}",
                    i + 1,
                    m.source.display()
                ));
            }
            if m.dest.trim().is_empty() {
                errors.push(format!("Mapping {}: destination is empty", i + 1));
            } else if Path::new(&m.dest).is_absolute() {
                errors.push(format!(
                    "Mapping {}: destination must be a relative path (got {})",
                    i + 1,
                    m.dest
                ));
            }
        }
        errors
    }
}

pub(crate) fn profiles_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        return dir.join("profiles.json");
    }
    PathBuf::from("profiles.json")
}

pub(crate) fn load_profiles(path: &Path) -> io::Result<Vec<Profile>> {
    let data = fs::read_to_string(path)?;
    let profiles: Vec<Profile> = serde_json::from_str(&data)?;
    Ok(profiles)
}

pub(crate) fn save_profiles(path: &Path, profiles: &[Profile]) -> io::Result<()> {
    let json = serde_json::to_string_pretty(profiles)?;
    fs::write(path, json)
}

pub(crate) fn default_profiles() -> Vec<Profile> {
    let flipper_dirs = ["subghz", "infrared", "nfc", "lfrfid", "ibutton", "badusb"];

    let mappings: Vec<Mapping> = flipper_dirs
        .iter()
        .map(|dir| Mapping {
            source: PathBuf::new(),
            dest: dir.to_string(),
            mode: CopyMode::OnlyIfMissing,
        })
        .collect();

    vec![Profile {
        name: "Default".into(),
        is_default: true,
        mappings,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_round_trip() {
        let profiles = default_profiles();
        let json = serde_json::to_string_pretty(&profiles).unwrap();
        let loaded: Vec<Profile> = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Default");
        assert_eq!(loaded[0].mappings.len(), 6);
    }

    #[test]
    fn validation_empty_name() {
        let mut p = Profile::new("");
        p.mappings.push(Mapping {
            source: PathBuf::from("/tmp/src"),
            dest: "subghz".into(),
            mode: CopyMode::OnlyIfMissing,
        });
        let errors = p.is_valid();
        assert!(errors.iter().any(|e| e.contains("name is empty")));
    }

    #[test]
    fn validation_empty_source() {
        let mut p = Profile::new("Test");
        p.mappings.push(Mapping {
            source: PathBuf::new(),
            dest: "subghz".into(),
            mode: CopyMode::OnlyIfMissing,
        });
        let errors = p.is_valid();
        assert!(errors.iter().any(|e| e.contains("source is empty")));
    }

    #[test]
    fn validation_absolute_dest() {
        let mut p = Profile::new("Test");
        p.mappings.push(Mapping {
            source: PathBuf::from("/tmp/src"),
            dest: "/absolute/path".into(),
            mode: CopyMode::OnlyIfMissing,
        });
        let errors = p.is_valid();
        assert!(errors.iter().any(|e| e.contains("relative path")));
    }
}
