//! TODO: comment, unittest

use std::fmt::Display;

#[derive(Debug)]
pub struct ValidUserName(String);

impl ValidUserName {
    pub fn parse(s: &str) -> Result<Self, anyhow::Error> {
        let is_empty_or_whitespace = s.trim().is_empty();
        if is_empty_or_whitespace {
            return Err(anyhow::anyhow!(
                "Invalid username: username contains whitespaces."
            ));
        }

        // NOTE: considering the number of bytes, not chars
        let is_too_long = s.len() > 60;
        if is_too_long {
            return Err(anyhow::anyhow!("Invalid username: username is too long."));
        }

        let forbidden_chars = ['/', '\'', '&', '\n', '\r', '\0', '"', '<', '>', '\\'];
        let contains_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));
        if contains_forbidden_chars {
            return Err(anyhow::anyhow!("Invalid username: username contains forbidden characters: '/', '\'', '&', '\n', '\r', '\0', '\"', '<', '>' or '\\'."));
        }

        Ok(Self(s.to_string()))
    }
}

impl Display for ValidUserName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for ValidUserName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for ValidUserName {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ValidUserName::parse(value)
    }
}

impl TryFrom<String> for ValidUserName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ValidUserName::parse(&value)
    }
}
