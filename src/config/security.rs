use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use ring::{digest, pbkdf2};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
const SALT_LEN: usize = 16;
const PBKDF2_ITERATIONS: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(100_000) };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurePassword {
    pub salt: String,
    pub hash: String,
}

pub struct PasswordManager;

impl PasswordManager {
    pub fn encrypt_password(password: &str) -> Result<SecurePassword> {
        let salt = Self::generate_salt();
        let hash = Self::derive_key(password, &salt)?;
        
        Ok(SecurePassword {
            salt: general_purpose::STANDARD.encode(&salt),
            hash: general_purpose::STANDARD.encode(&hash),
        })
    }

    pub fn verify_password(password: &str, secure_password: &SecurePassword) -> Result<bool> {
        let salt = general_purpose::STANDARD.decode(&secure_password.salt)?;
        let expected_hash = general_purpose::STANDARD.decode(&secure_password.hash)?;
        
        let actual_hash = Self::derive_key(password, &salt)?;
        
        Ok(actual_hash == expected_hash)
    }

    fn generate_salt() -> Vec<u8> {
        use ring::rand::{SecureRandom, SystemRandom};
        
        let rng = SystemRandom::new();
        let mut salt = vec![0u8; SALT_LEN];
        rng.fill(&mut salt).expect("Failed to generate random salt");
        salt
    }

    fn derive_key(password: &str, salt: &[u8]) -> Result<Vec<u8>> {
        let mut derived_key = vec![0u8; CREDENTIAL_LEN];
        
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            PBKDF2_ITERATIONS,
            salt,
            password.as_bytes(),
            &mut derived_key,
        );
        
        Ok(derived_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_encryption_and_verification() {
        let password = "test_password_123";
        
        let secure_password = PasswordManager::encrypt_password(password).unwrap();
        
        assert!(PasswordManager::verify_password(password, &secure_password).unwrap());
        
        assert!(!PasswordManager::verify_password("wrong_password", &secure_password).unwrap());
    }

    #[test]
    fn test_different_salts_produce_different_hashes() {
        let password = "same_password";
        
        let secure1 = PasswordManager::encrypt_password(password).unwrap();
        let secure2 = PasswordManager::encrypt_password(password).unwrap();
        
        assert_ne!(secure1.salt, secure2.salt);
        assert_ne!(secure1.hash, secure2.hash);
        
        assert!(PasswordManager::verify_password(password, &secure1).unwrap());
        assert!(PasswordManager::verify_password(password, &secure2).unwrap());
    }
}