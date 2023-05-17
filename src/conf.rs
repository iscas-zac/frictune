use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Conf {
    pub db_uri: String,
}

impl ::std::default::Default for Conf {
    fn default() -> Self { Conf { db_uri: "./tags.db".to_string() } }
}

/// Get a configuration value from the file.
pub fn read_config() -> anyhow::Result<Conf> {
    let local_path = std::env::current_dir()?.join("settings");
    if let Some(path_str) = local_path.to_str()
    {
        let cfg: Conf = confy::load(path_str, "path")?;
        Ok(cfg)
    }
    else { anyhow::bail!("path to ascii str fail") }
}