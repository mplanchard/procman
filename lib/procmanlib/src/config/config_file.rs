//! Utilities for finding and parsing the config file

use itertools::Itertools;
use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use ProcmanConfigFileError::*;

use super::format::ProcmanConfigFormat;

pub(crate) struct ProcmanConfigFile {
    pub(crate) path: PathBuf,
    pub(crate) format: Option<ProcmanConfigFormat>,
}

impl TryFrom<&Path> for ProcmanConfigFile {
    type Error = ProcmanConfigFileError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        // see if we can determine an expected format from the extension
        let format = path
            .file_name()
            // this should never happen if we were able to open the file, but :shrug:
            .ok_or_else(|| InvalidPath(path.to_owned(), InvalidPathReason::NoBasename))?
            .to_string_lossy()
            .rsplit_once('.')
            .and_then(|(_, ext)| ext.parse().ok());

        Ok(Self {
            path: path.to_owned(),
            format,
        })
    }
}

impl ProcmanConfigFile {
    pub(crate) fn find() -> Result<ProcmanConfigFile, ProcmanConfigFileError> {
        let candidates = [
            std::env::current_dir().map_err(CouldNotDetermineCwd)?,
            directories::ProjectDirs::from("", "", "procman")
                .ok_or(CouldNotDetermineDirectory)?
                .config_dir()
                .to_owned(),
        ];

        for candidate in &candidates {
            if let Some(path_ext) = Self::config_in_dir(candidate)? {
                return Ok(path_ext);
            }
        }

        Err(NotFoundInSearch(candidates.to_vec()))
    }

    fn config_in_dir(dir: &Path) -> Result<Option<ProcmanConfigFile>, ProcmanConfigFileError> {
        let mut files = read_dir(dir).map_err(|e| CouldNotReadDirectory(dir.to_owned(), e))?;

        let candidates = files
            .try_fold(vec![], |mut acc, i| {
                let f = i?;
                match f.file_name().to_str() {
                    // if the fname isn't valid utf-8, we don't care about it
                    None => {}
                    // if it's procman or procman.conf, we can't tell the file
                    // type, but it's potentially a config file
                    Some("procman" | "procman.conf") => {
                        acc.push(ProcmanConfigFile {
                            path: f.path(),
                            format: None,
                        });
                    }
                    // if the name is something else, check for procman with
                    // our known config formats.
                    Some(fname) => {
                        if let Some(("procman", ext)) = fname.rsplit_once('.') {
                            if let Ok(format) = ext.parse() {
                                acc.push(ProcmanConfigFile {
                                    path: f.path(),
                                    format: Some(format),
                                });
                            }
                        }
                    }
                }
                Ok(acc)
            })
            .map_err(|e| CouldNotReadDirectory(dir.to_owned(), e))?;

        match candidates.len() {
            0 | 1 => Ok(candidates.into_iter().next()),
            _ => Err(MultipleCandidates(
                dir.to_owned(),
                candidates.into_iter().map(|v| v.path).collect(),
            )),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ProcmanConfigFileError {
    #[error(
        "could not determine config directory: this could be due to the current \
             user not having a home directory - please consider specifying the path \
             manually if this is the case"
    )]
    CouldNotDetermineDirectory,

    #[error("failed to determine current working directory")]
    #[non_exhaustive]
    CouldNotDetermineCwd(#[source] std::io::Error),

    #[error("could not read directory at {0}")]
    #[non_exhaustive]
    CouldNotReadDirectory(PathBuf, #[source] std::io::Error),

    #[error("multiple candidate configs in directory {0}: {}",
            .1.iter().map(|i| i.to_string_lossy()).join(", "))]
    #[non_exhaustive]
    MultipleCandidates(PathBuf, Vec<PathBuf>),

    #[error("could not find config file in search paths: {}",
            .0.iter().map(|i| i.to_string_lossy()).join(", "))]
    #[non_exhaustive]
    NotFoundInSearch(Vec<PathBuf>),

    #[error("invalid path at {0}")]
    #[non_exhaustive]
    InvalidPath(PathBuf, #[source] InvalidPathReason),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum InvalidPathReason {
    #[error("could not determine basename")]
    NoBasename,
}
