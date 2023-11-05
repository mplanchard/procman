//! Program-related types

use itertools::Itertools;

#[derive(Debug, thiserror::Error)]
#[error("invalid program name: {name}: {kind}")]
pub struct ProgramNameError {
    name: String,
    kind: ProgramNameErrorKind,
}

#[derive(Debug, thiserror::Error)]
pub enum ProgramNameErrorKind {
    #[error("invalid characters: {}", .0.iter().join(", "))]
    InvalidCharacters(Vec<char>),
}

#[derive(Debug, Hash, PartialEq, Eq, serde::Deserialize)]
#[serde(try_from = "String")]
pub(crate) struct ProgramName(String);
impl ProgramName {
    pub(crate) fn try_new<T: ToString>(name: T) -> Result<Self, ProgramNameError> {
        let name = name.to_string();

        let invalid_chars = name
            .chars()
            .filter(|c| !Self::is_valid_char(c))
            .collect::<Vec<_>>();

        if !invalid_chars.is_empty() {
            return Err(ProgramNameError {
                name,
                kind: ProgramNameErrorKind::InvalidCharacters(invalid_chars),
            });
        }

        Ok(Self(name))
    }

    fn is_valid_char(c: &char) -> bool {
        c.is_ascii_alphanumeric() || matches!(c, '_' | '-')
    }
}

impl TryFrom<String> for ProgramName {
    type Error = ProgramNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
