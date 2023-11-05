//! Supported config file formats

#[derive(Debug, Clone, Copy, strum::EnumIter, strum::EnumString, strum::IntoStaticStr)]
pub(crate) enum ProcmanConfigFormat {
    #[strum(serialize = "toml")]
    Toml,
    #[strum(serialize = "json")]
    Json,
    #[strum(serialize = "yaml", serialize = "yml")]
    Yaml,
}
