use std::fs;
use std::iter;
use std::path::Path;

use crate::error::AppResult;

use super::{Assessment, Check, Facts, Fix};

pub(super) struct LogFolder;

impl LogFolder {
    pub(super) fn writable(dir: &Path) -> AppResult {
        fs::create_dir_all(dir)?;
        let probe = dir.join(".doctor");
        fs::write(&probe, "")?;
        fs::remove_file(probe)?;
        Ok(())
    }
}

impl Check for LogFolder {
    const LABEL: &str = "Log folder:";
    type Problem<'a> = &'a Path;

    fn assessments(facts: &Facts) -> impl Iterator<Item = Assessment<'_>> {
        iter::once(if facts.writable {
            Self::passed("writable")
        } else {
            Self::failed("unwritable", &facts.log_folder)
        })
    }

    fn suggested_fix(log_folder: Self::Problem<'_>) -> Fix<'_> {
        Fix::MakeWritable(log_folder)
    }
}
