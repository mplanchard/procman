//! Wrapper around config file contents

use std::{path::PathBuf, string::FromUtf8Error};

use super::{config_file::ProcmanConfigFile, format::ProcmanConfigFormat, ProcmanConfig};
use itertools::Itertools;

use strum::IntoEnumIterator;
use ProcmanConfigParseErrorKind::*;

pub(crate) struct ProcmanConfigFileParser {
    path: PathBuf,
    contents: Vec<u8>,
    format: Option<ProcmanConfigFormat>,
}
impl TryFrom<ProcmanConfigFile> for ProcmanConfigFileParser {
    type Error = ProcmanConfigParseError;

    fn try_from(file: ProcmanConfigFile) -> Result<Self, Self::Error> {
        let contents = std::fs::read(&file.path)
            .map_err(|e| ProcmanConfigParseError::new(file.path.as_path(), CouldNotReadFile(e)))?;

        Ok(Self {
            path: file.path,
            contents,
            format: file.format,
        })
    }
}
impl ProcmanConfigFileParser {
    pub(crate) fn parse(self) -> Result<ProcmanConfig, ProcmanConfigParseError> {
        if let Some(format) = self.format {
            return format
                .parse(&self.contents)
                .map_err(|e| ProcmanConfigParseError::new(self.path.as_path(), e));
        }

        for format in ProcmanConfigFormat::iter() {
            if let Ok(conf) = format.parse(&self.contents) {
                return Ok(conf);
            }
        }

        Err(ProcmanConfigParseError::new(
            self.path.as_path(),
            CouldNotDetermineFormat,
        ))
    }
}

impl ProcmanConfigFormat {
    fn parse(&self, contents: &[u8]) -> Result<ProcmanConfig, ProcmanConfigParseErrorKind> {
        match self {
            ProcmanConfigFormat::Toml => {
                let contents = String::from_utf8(contents.to_vec())?;
                Ok(toml::from_str(&contents)?)
            }
            ProcmanConfigFormat::Json => Ok(serde_json::from_slice(contents)?),
            ProcmanConfigFormat::Yaml => Ok(serde_yaml::from_slice(contents)?),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("could not parse config file at {path}")]
pub struct ProcmanConfigParseError {
    path: PathBuf,
    #[source]
    kind: ProcmanConfigParseErrorKind,
}
impl ProcmanConfigParseError {
    fn new<
        P: ToOwned<Owned = std::path::PathBuf> + ?Sized,
        K: Into<ProcmanConfigParseErrorKind>,
    >(
        path: &P,
        kind: K,
    ) -> Self {
        Self {
            path: path.to_owned(),
            kind: kind.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProcmanConfigParseErrorKind {
    #[error("could not read contents of file")]
    CouldNotReadFile(std::io::Error),

    #[error("file contained invalid utf-8")]
    InvalidUtf8(#[from] FromUtf8Error),

    #[error("file contained invalid toml")]
    InvalidToml(#[from] toml::de::Error),

    #[error("file contained invalid json")]
    InvalidJson(#[from] serde_json::Error),

    #[error("file contained invalid yaml")]
    InvalidYaml(#[from] serde_yaml::Error),

    #[error("could not parse config file as any supported format ({})",
            ProcmanConfigFormat::iter().map(<&'static str>::from).join(", "))]
    CouldNotDetermineFormat,
}
