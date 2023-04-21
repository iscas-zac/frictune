use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Conf {
    pub db_uri: String,
}

impl ::std::default::Default for Conf {
    fn default() -> Self { Conf { db_uri: "./tags.db".to_string() } }
}

/// Get a configuration value from the file.
pub fn read_config() -> Result<Conf, confy::ConfyError> {
    let cfg: Conf = confy::load("./settings.toml", None)?;
    Ok(cfg)
}