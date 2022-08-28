use digest::{Digest, FixedOutputReset};
use std::{fmt, error::Error};

#[derive(Debug, PartialEq, Eq)]
pub enum PrefixedApiKeyError {
    WrongNumberOfParts(usize),
}

impl Error for PrefixedApiKeyError {}

impl fmt::Display for PrefixedApiKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Display should use something other than debug display
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct PrefixedApiKey {
    prefix: String,
    short_token: String,
    long_token: String,
}

impl PrefixedApiKey {
    pub fn new(prefix: String, short_token: String, long_token: String) -> PrefixedApiKey {
        PrefixedApiKey {
            prefix,
            short_token,
            long_token,
        }
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn short_token(&self) -> &str {
        &self.short_token
    }

    pub fn long_token(&self) -> &str {
        &self.long_token
    }

    /// Hashes the long token using the provided hashing algorithm
    pub fn long_token_hashed<H: Digest + FixedOutputReset>(&self, hasher: &mut H) -> String {
        Digest::update(hasher, self.long_token.clone());
        hex::encode(hasher.finalize_reset())
    }

    pub fn from_string(pak_string: &str) -> Result<PrefixedApiKey, PrefixedApiKeyError> {
        let parts: Vec<&str> = pak_string.split('_').collect();

        if parts.len() != 3 {
            // Incorrect number of parts
            return Err(PrefixedApiKeyError::WrongNumberOfParts(parts.len()));
        }

        Ok(PrefixedApiKey::new(
            parts[0].to_owned(),
            parts[1].to_owned(),
            parts[2].to_owned(),
        ))
    }

    pub fn as_string(&self) -> String {
        format!("{}_{}_{}", self.prefix, self.short_token, self.long_token)
    }
}

impl TryInto<PrefixedApiKey> for &str {
    type Error = PrefixedApiKeyError;

    fn try_into(self) -> Result<PrefixedApiKey, Self::Error> {
        PrefixedApiKey::from_string(self)
    }
}

impl fmt::Display for PrefixedApiKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Displays the key without showing the secret long token
        write!(f, "{}_{}_***", self.prefix(), self.short_token())
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use crate::{PrefixedApiKey, PrefixedApiKeyError};

    #[test]
    fn as_string_is_expected() {
        let prefix = "mycompany".to_owned();
        let short = "abcdefg".to_owned();
        let long = "bacdegadsa".to_owned();
        let expected_token = format!("{}_{}_{}", prefix, short, long);
        let pak = PrefixedApiKey::new(prefix, short, long);
        assert_eq!(pak.as_string(), expected_token)
    }

    #[test]
    fn self_from_string_works() {
        let pak_string = "mycompany_abcdefg_bacdegadsa";
        let pak_result = PrefixedApiKey::from_string(pak_string);
        assert_eq!(pak_result.is_ok(), true);
        assert_eq!(pak_result.unwrap().as_string(), pak_string);
    }

    #[test]
    fn str_into_pak() {
        let pak_string = "mycompany_abcdefg_bacdegadsa";
        let pak_result: Result<PrefixedApiKey, _> = pak_string.try_into();
        assert_eq!(pak_result.is_ok(), true);
        assert_eq!(pak_result.unwrap().as_string(), pak_string);
    }

    #[test]
    fn string_into_pak_via_as_ref() {
        let pak_string = "mycompany_abcdefg_bacdegadsa".to_owned();
        let pak_result: Result<PrefixedApiKey, _> = pak_string.as_str().try_into();
        assert_eq!(pak_result.is_ok(), true);
        assert_eq!(pak_result.unwrap().as_string(), pak_string);
    }

    #[test]
    fn str_into_pak_with_extra_parts() {
        let pak_string = "mycompany_abcd_efg_bacdegadsa";
        let pak_result: Result<PrefixedApiKey, _> = pak_string.try_into();
        assert_eq!(pak_result.is_err(), true);
        assert_eq!(
            pak_result.unwrap_err(),
            PrefixedApiKeyError::WrongNumberOfParts(4)
        );
    }

    #[test]
    fn check_long_token() {
        let pak_string = "mycompany_CEUsS4psCmc_BddpcwWyCT3EkDjHSSTRaSK1dxtuQgbjb";
        let hash = "0f01ab6e0833f280b73b2b618c16102d91c0b7c585d42a080d6e6603239a8bee";

        let pak: PrefixedApiKey = pak_string.try_into().unwrap();
        let mut hasher = Sha256::new();
        assert_eq!(pak.long_token_hashed(&mut hasher), hash);
    }
}
