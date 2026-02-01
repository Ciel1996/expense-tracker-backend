use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct Oidc {
    issuer_url: String,
    audience: String,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct ExpenseTracker {
    port: u16,
    db_connection_string: String,
    cors_url: String,
}

impl ExpenseTracker {
    /// The port the api is reachable at.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The connection string used to connect the API to the database.
    pub fn db_connection_string(&self) -> &str {
        &self.db_connection_string
    }

    pub fn cors_url(&self) -> &str { &self.cors_url }
}

impl Oidc {
    pub fn issuer_url(&self) -> &str {
        &self.issuer_url
    }

    pub fn audience(&self) -> &str {
        &self.audience
    }
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    oidc: Oidc,
    expense_tracker: ExpenseTracker,
}

impl Settings {
    /// Looks for a file with the given name + extension and tries to deserialize it.
    pub fn new(file_name: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(config::File::with_name(file_name))
            .build()?;

        s.try_deserialize()
    }

    /// Gets the OIDC settings stored in your settings file.
    pub fn oidc(&self) -> &Oidc {
        &self.oidc
    }

    /// Gets the expense_tracker settings stored in your settings file.
    pub fn expense_tracker(&self) -> &ExpenseTracker {
        &self.expense_tracker
    }
}
