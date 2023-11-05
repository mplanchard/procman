//! Procman Configuration

mod config_file;
mod format;
mod parser;

use std::{collections::HashMap, path::Path};

use crate::{config::parser::ProcmanConfigFileParser, program_name::ProgramName};

use self::{
    config_file::{ProcmanConfigFile, ProcmanConfigFileError},
    parser::ProcmanConfigParseError,
};

#[derive(Debug, serde::Deserialize)]
pub struct ProcmanConfig {
    programs: HashMap<ProgramName, ProgramConfig>,
}
impl ProcmanConfig {
    pub fn load_from_file(path: Option<&Path>) -> Result<Self, ProcmanConfigError> {
        let config_file = if let Some(path) = path {
            ProcmanConfigFile::try_from(path)?
        } else {
            ProcmanConfigFile::find()?
        };

        let parser = ProcmanConfigFileParser::try_from(config_file)?;

        Ok(parser.parse()?)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ProgramConfig {
    command: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProcmanConfigError {
    #[error("failed to find config file")]
    FindFile(#[from] ProcmanConfigFileError),
    #[error("failed to parse config file")]
    ParseFile(#[from] ProcmanConfigParseError),
}

#[cfg(test)]
mod test {
    use std::fs::read_dir;

    use super::*;

    #[test]
    fn validate_examples() {
        let files = read_dir(format!(
            "{}/src/config/examples/",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();

        for f in files
            .into_iter()
            .map(|res| res.unwrap())
            .filter(|f| dbg!(f.file_name()) != "README.md")
        {
            let conf = ProcmanConfig::load_from_file(Some(&f.path())).unwrap();
            dbg!(conf);
        }
    }
}
