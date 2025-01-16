use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Clone, Deserialize, Debug)]
pub struct Settings {
    pub application: ApplicationSettings,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

impl ApplicationSettings {
    pub fn get_full_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// The possible runtime environment
pub enum Environment {
    Test,
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Test => "Test",
            Environment::Local => "Local",
            Environment::Production => "Production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "test" => Ok(Self::Test),
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(anyhow::anyhow!(
                "{} is not a supported enviroment. Use either `local` or `production`.",
                other
            )),
        }
    }
}

pub fn get_config() -> Result<Settings, anyhow::Error> {
    let base_path = std::env::current_dir()?;
    let configuration_directory = base_path.join("configuration");

    // Detect running enviroment
    // Default to `test` if unspecified
    let enviroment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "test".into())
        .try_into()?;
    let enviroment_filename = format!("{}.toml", enviroment.as_str());

    // Initialize our configuration reader
    let settings = config::Config::builder()
        // Adding Base
        .add_source(config::File::from(
            configuration_directory.join("Base.toml"),
        ))
        // Adding Test, Local or Production
        .add_source(config::File::from(
            configuration_directory.join(enviroment_filename),
        ))
        // Add in settings from enviroment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001` would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    // Try to convert the configuration values it reads into our Settings type
    Ok(settings.try_deserialize::<Settings>()?)
}
