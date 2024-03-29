use config::{Config, ConfigError, Environment, File};
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::Deserialize;

#[derive(Clone)]
struct KeyHolder {
    enc: EncodingKey,
    dec: DecodingKey,
}

impl Default for KeyHolder {
    fn default() -> Self {
        Self {
            enc: EncodingKey::from_secret(&[]),
            dec: DecodingKey::from_secret(&[]),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct Cookie {
    pub name: String,
    pub secret: String,
    #[serde(skip)]
    key_holder: KeyHolder,
}

impl Cookie {
    pub fn encoder(&self) -> &EncodingKey {
        &self.key_holder.enc
    }
    pub fn decoder(&self) -> &DecodingKey {
        &self.key_holder.dec
    }
    fn set_keys(&mut self) {
        self.key_holder.enc = EncodingKey::from_secret(self.secret.as_bytes());
        self.key_holder.dec = DecodingKey::from_secret(self.secret.as_bytes());
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub name: String,
}

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub server: Server,
    pub cookie: Cookie,
    pub db: Database,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s: Self = Config::builder()
            .add_source(File::with_name("./settings.toml"))
            .add_source(Environment::with_prefix("strecke").separator("_"))
            .build()?
            .try_deserialize()?;
        s.cookie.set_keys();
        Ok(s)
    }
}
