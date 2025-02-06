//! TODO: comment, unittest

use validator::ValidateEmail;

#[derive(Debug)]
pub struct ValidEmail(String);

impl ValidEmail {
    pub fn parse(s: &str) -> Result<Self, anyhow::Error> {
        // NOTE: considering the lenght in bytes, not in chars
        if s.len() > 200 {
            return Err(anyhow::anyhow!(
                "Invalid email: email provided is too long."
            ));
        }

        if s.validate_email() {
            Ok(Self(s.to_string()))
        } else {
            Err(anyhow::anyhow!(
                "Invalid email: \"{}\" is not a valid email",
                s
            ))
        }
    }
}

impl AsRef<str> for ValidEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ValidEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<&str> for ValidEmail {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ValidEmail::parse(value)
    }
}

impl TryFrom<String> for ValidEmail {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ValidEmail::parse(&value)
    }
}
