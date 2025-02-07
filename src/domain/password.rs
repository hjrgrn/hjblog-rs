use secrecy::{ExposeSecret, SecretString};

#[derive(Debug)]
pub struct ValidPassword(SecretString);

impl ValidPassword {
    pub fn parse(s: &SecretString) -> Result<Self, anyhow::Error> {
        if s.expose_secret().trim().is_empty() {
            return Err(anyhow::anyhow!("Invalid password: password is empty."));
        }

        if s.expose_secret().len() < 3 {
            return Err(anyhow::anyhow!("Invalid password: password is too short."));
        }

        if s.expose_secret().len() > 200 {
            return Err(anyhow::anyhow!("Invalid password: password is too long."));
        }

        Ok(Self(s.clone()))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}
