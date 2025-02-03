use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;

#[derive(Clone, Deserialize, Debug)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(Clone, Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    /// # `connection_string`
    ///
    /// Provides a convinient way to obtain a connection string compatible
    /// with `sqlx::PgConnection::connect`
    pub fn connection_string(&self) -> SecretString {
        SecretString::new(
            format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port,
                self.database_name
            )
            .into(),
        )
    }

    /// # `without_db`
    ///
    /// Generates a connection option compatible with `sqlx::PgConnection::connect_with`
    /// that we will be using in integration test. The database name is ommitted and
    /// will have to be provided.
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            // Try an encrypted connection, fallback to unencrypted if it fails
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    /// # `with_db`
    ///
    /// Same as `without_db` but with a database.
    pub fn with_db(&self) -> PgConnectOptions {
        let options = self
            .without_db()
            .database(&self.database_name)
            .log_statements(tracing_log::log::LevelFilter::Trace);
        options
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub hmac_secret: SecretString,
    pub cookie_secure: bool,
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
