//! Cryptographic utilities for secure communication and storage.
//! 
//! This module implements AES-256-GCM encryption/decryption and key management.

use crate::{Error, Result};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use std::sync::Arc;
use tokio::sync::RwLock;
use zeroize::Zeroize;

/// Encryption manager handles key management and crypto operations
pub struct CryptoManager {
    // In a real system, keys would be rotated and securely stored
    // For this CLI, we generate a session key or load from config
    key: Key<Aes256Gcm>,
}

impl Drop for CryptoManager {
    fn drop(&mut self) {
        // Zeroize the key to prevent it from being left in memory
        // Convert key to mutable by taking ownership or using interior mutability
        let key_ptr = &mut self.key as *mut Key<Aes256Gcm>;
        unsafe {
            let key_bytes = std::slice::from_raw_parts_mut(key_ptr as *mut u8, 32);
            key_bytes.zeroize();
        }
    }
}

impl CryptoManager {
    /// Create a new crypto manager with a generated key
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        Self { key }
    }

    /// Create from an existing key (e.g. loaded from secure storage)
    pub fn from_key(key_bytes: &[u8]) -> Result<Self> {
        if key_bytes.len() != 32 {
            return Err(Error::Config("Invalid key length for AES-256".to_string()));
        }
        let key = Key::<Aes256Gcm>::from_slice(key_bytes).clone();
        Ok(Self { key })
    }

    /// Encrypt data using AES-256-GCM
    /// Returns: base64(nonce + ciphertext)
    pub fn encrypt(&self, data: &[u8]) -> Result<String> {
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        
        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|e| Error::Internal(format!("Encryption failed: {}", e)))?;
            
        // Combine nonce and ciphertext
        let mut payload = nonce.to_vec();
        payload.extend_from_slice(&ciphertext);
        
        Ok(BASE64.encode(payload))
    }

    /// Decrypt data using AES-256-GCM
    /// Input: base64(nonce + ciphertext)
    pub fn decrypt(&self, encrypted_data: &str) -> Result<Vec<u8>> {
        let payload = BASE64.decode(encrypted_data)
            .map_err(|e| Error::Validation(format!("Invalid base64: {}", e)))?;
            
        if payload.len() < 12 {
            return Err(Error::Validation("Payload too short".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = payload.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(&self.key);
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| Error::Validation(format!("Decryption failed: {}", e)))?;
            
        Ok(plaintext)
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let crypto = CryptoManager::new();
        let data = b"Secret Data 123";
        
        let encrypted = crypto.encrypt(data).expect("Encryption failed");
        let decrypted = crypto.decrypt(&encrypted).expect("Decryption failed");
        
        assert_eq!(data.to_vec(), decrypted);
    }
    
    #[test]
    fn test_decrypt_invalid_data() {
        let crypto = CryptoManager::new();
        let result = crypto.decrypt("invalidbase64");
        assert!(result.is_err());
    }
}
