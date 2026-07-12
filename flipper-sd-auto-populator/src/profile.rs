use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Profile {
    name: String,
    is_default: bool,
    mappings: Vec<Mapping>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Mapping {
    source: PathBuf,
    dest: String,
    mode: CopyMode,
}
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub(crate) enum CopyMode {
    ExactMirror,
    OnlyIfMissing,
    //OverwriteNewer
}

impl CopyMode {
    fn description(&self) -> &str {
        match self {
            CopyMode::ExactMirror => {
                "Deletes all files in the destination path, and then copies all files/folders from the source to the destination."
            }
            CopyMode::OnlyIfMissing => {
                "Only copies the files from the source that the destination does not already contain"
            } //CopyMode::OverwriteNewer => {"Only copies the files from the source when they dont already exist in the destination or when the source files are older than the ones in the destination"}
        }
    }
}
