use std::path::PathBuf;

use sysinfo::Disks;

#[derive(Debug, Clone)]
pub(crate) struct DriveInfo {
    pub path: PathBuf,
    pub name: String,
    pub total_space: u64,
    pub is_removable: bool,
    pub is_flipper: bool,
}

const FLIPPER_DIRS: &[&str] = &[
    "subghz", "infrared", "nfc", "lfrfid", "ibutton", "badusb", "dolphin", "apps",
];

pub(crate) fn detect_drives() -> Vec<DriveInfo> {
    let disks = Disks::new_with_refreshed_list();
    let mut drives = Vec::new();

    for disk in disks.list() {
        let mount = disk.mount_point().to_path_buf();
        let name = disk.name().to_string_lossy().into_owned();
        let total_space = disk.total_space();
        let is_removable = disk.is_removable();

        let is_flipper = check_flipper(&mount);

        drives.push(DriveInfo {
            path: mount,
            name,
            total_space,
            is_removable,
            is_flipper,
        });
    }

    drives
}

fn check_flipper(root: &std::path::Path) -> bool {
    let matches = FLIPPER_DIRS
        .iter()
        .filter(|dir| root.join(dir).is_dir())
        .count();
    matches >= 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detect_not_empty() {
        let drives = detect_drives();
        assert!(!drives.is_empty(), "should find at least the root disk");
    }

    #[test]
    fn flipper_detection_false_on_empty_dir() {
        let dir = std::env::temp_dir().join("flipper_test_nonexistent_982374");
        let _ = fs::remove_dir_all(&dir);
        assert!(!check_flipper(&dir));
    }
}
