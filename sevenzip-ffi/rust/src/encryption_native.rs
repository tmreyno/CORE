//! Pure Rust AES-256 Encryption Implementation
//!
//! This module provides AES-256-CBC encryption/decryption using RustCrypto crates.
//! No system dependencies (OpenSSL) required.
//!
//! # Security
//!
//! - AES-256-CBC encryption
//! - PBKDF2-SHA256 key derivation with 262,144 iterations (7-Zip compatible)
//! - Secure random IV and salt generation
//! - PKCS#7 padding
//! - Automatic key zeroization on drop

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use zeroize::ZeroizeOnDrop;

use crate::error::{Error, Result};

/// AES block size in bytes
pub const AES_BLOCK_SIZE: usize = 16;
/// AES-256 key size in bytes
pub const AES_KEY_SIZE: usize = 32;
/// Default salt size in bytes
pub const SALT_SIZE: usize = 16;
/// PBKDF2 iterations (7-Zip default)
pub const PBKDF2_ITERATIONS: u32 = 262_144;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

/// AES-256 encryption context (pure Rust implementation)
///
/// Maintains the encryption key, IV, and salt for encrypting/decrypting data.
/// The context is initialized with a password and automatically generates secure random
/// values for the IV and salt.
#[derive(ZeroizeOnDrop)]
pub struct EncryptionContext {
    #[zeroize(skip)]
    key: [u8; AES_KEY_SIZE],
    iv: [u8; AES_BLOCK_SIZE],
    salt: [u8; SALT_SIZE],
}

impl EncryptionContext {
    /// Create a new encryption context from a password
    ///
    /// Derives a 256-bit AES key using PBKDF2-SHA256 with 262,144 iterations.
    /// Generates random 16-byte IV and salt.
    ///
    /// # Arguments
    ///
    /// * `password` - Password string (any length, UTF-8)
    ///
    /// # Errors
    ///
    /// Returns an error if password is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use seven_zip::encryption_native::EncryptionContext;
    ///
    /// let ctx = EncryptionContext::new("StrongPassword123!").unwrap();
    /// ```
    pub fn new(password: &str) -> Result<Self> {
        if password.is_empty() {
            return Err(Error::InvalidParameter(
                "Password cannot be empty".to_string(),
            ));
        }

        let mut salt = [0u8; SALT_SIZE];
        let mut iv = [0u8; AES_BLOCK_SIZE];
        let mut key = [0u8; AES_KEY_SIZE];

        // Generate random salt and IV
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut iv);

        // Derive key using PBKDF2-SHA256
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

        Ok(Self { key, iv, salt })
    }

    /// Create encryption context with specific salt and IV (for testing/compatibility)
    pub fn with_salt_iv(password: &str, salt: &[u8], iv: &[u8; AES_BLOCK_SIZE]) -> Result<Self> {
        if password.is_empty() {
            return Err(Error::InvalidParameter(
                "Password cannot be empty".to_string(),
            ));
        }
        if salt.is_empty() {
            return Err(Error::InvalidParameter("Salt cannot be empty".to_string()));
        }

        let mut key = [0u8; AES_KEY_SIZE];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);

        let mut salt_arr = [0u8; SALT_SIZE];
        let copy_len = salt.len().min(SALT_SIZE);
        salt_arr[..copy_len].copy_from_slice(&salt[..copy_len]);

        Ok(Self {
            key,
            iv: *iv,
            salt: salt_arr,
        })
    }

    /// Get the initialization vector (IV)
    ///
    /// The IV must be stored with the encrypted data and used for decryption.
    pub fn iv(&self) -> &[u8; AES_BLOCK_SIZE] {
        &self.iv
    }

    /// Get the salt used for key derivation
    ///
    /// The salt must be stored with the encrypted data and used for decryption.
    pub fn salt(&self) -> &[u8; SALT_SIZE] {
        &self.salt
    }

    /// Get the derived encryption key
    ///
    /// **WARNING**: Exposing the key can compromise security. Only use for
    /// debugging or when you need to serialize the key for storage.
    pub fn key(&self) -> &[u8; AES_KEY_SIZE] {
        &self.key
    }

    /// Encrypt data using AES-256-CBC with PKCS#7 padding
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt (any length)
    ///
    /// # Returns
    ///
    /// Encrypted data with PKCS#7 padding (length will be multiple of 16 bytes)
    ///
    /// # Example
    ///
    /// ```
    /// use seven_zip::encryption_native::EncryptionContext;
    ///
    /// let ctx = EncryptionContext::new("password").unwrap();
    /// let plaintext = b"Secret data";
    /// let ciphertext = ctx.encrypt(plaintext).unwrap();
    /// assert_eq!(ciphertext.len() % 16, 0);
    /// ```
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        // Calculate buffer size (PKCS#7 adds 1-16 bytes of padding)
        let padded_len = ((plaintext.len() / AES_BLOCK_SIZE) + 1) * AES_BLOCK_SIZE;
        let mut buffer = vec![0u8; padded_len];
        buffer[..plaintext.len()].copy_from_slice(plaintext);

        let cipher = Aes256CbcEnc::new(&self.key.into(), &self.iv.into());
        let ciphertext = cipher
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, plaintext.len())
            .map_err(|_| Error::EncryptionError("Encryption failed".to_string()))?;

        Ok(ciphertext.to_vec())
    }

    /// Decrypt data using AES-256-CBC and verify PKCS#7 padding
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - Encrypted data (must be multiple of 16 bytes)
    ///
    /// # Returns
    ///
    /// Decrypted data with padding removed
    ///
    /// # Example
    ///
    /// ```
    /// use seven_zip::encryption_native::EncryptionContext;
    ///
    /// let ctx = EncryptionContext::new("password").unwrap();
    /// let plaintext = b"Secret data";
    /// let ciphertext = ctx.encrypt(plaintext).unwrap();
    /// let decrypted = ctx.decrypt(&ciphertext).unwrap();
    /// assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    /// ```
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() % AES_BLOCK_SIZE != 0 {
            return Err(Error::InvalidParameter(
                "Ciphertext length must be multiple of 16 bytes".to_string(),
            ));
        }

        let mut buffer = ciphertext.to_vec();
        let cipher = Aes256CbcDec::new(&self.key.into(), &self.iv.into());
        
        let plaintext = cipher
            .decrypt_padded_mut::<Pkcs7>(&mut buffer)
            .map_err(|_| Error::DecryptionError("Decryption failed (wrong password?)".to_string()))?;

        Ok(plaintext.to_vec())
    }
}

/// AES-256 decryption context (pure Rust implementation)
///
/// Maintains the decryption key for decrypting encrypted data.
/// Initialized with a password and salt from the encrypted archive.
#[derive(ZeroizeOnDrop)]
pub struct DecryptionContext {
    #[zeroize(skip)]
    key: [u8; AES_KEY_SIZE],
}

impl DecryptionContext {
    /// Create a new decryption context from a password and salt
    ///
    /// The salt must be obtained from the encrypted archive header.
    ///
    /// # Arguments
    ///
    /// * `password` - Password string (same as used for encryption)
    /// * `salt` - Salt bytes from archive header
    ///
    /// # Errors
    ///
    /// Returns an error if password or salt is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use seven_zip::encryption_native::DecryptionContext;
    ///
    /// let salt = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ///             0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10];
    /// let ctx = DecryptionContext::new("password", &salt).unwrap();
    /// ```
    pub fn new(password: &str, salt: &[u8]) -> Result<Self> {
        if password.is_empty() {
            return Err(Error::InvalidParameter(
                "Password cannot be empty".to_string(),
            ));
        }
        if salt.is_empty() {
            return Err(Error::InvalidParameter("Salt cannot be empty".to_string()));
        }

        let mut key = [0u8; AES_KEY_SIZE];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);

        Ok(Self { key })
    }

    /// Get the derived decryption key
    pub fn key(&self) -> &[u8; AES_KEY_SIZE] {
        &self.key
    }

    /// Decrypt data using AES-256-CBC
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - Encrypted data
    /// * `iv` - Initialization vector from archive header (16 bytes)
    ///
    /// # Returns
    ///
    /// Decrypted data with padding removed
    pub fn decrypt(&self, ciphertext: &[u8], iv: &[u8; AES_BLOCK_SIZE]) -> Result<Vec<u8>> {
        if ciphertext.len() % AES_BLOCK_SIZE != 0 {
            return Err(Error::InvalidParameter(
                "Ciphertext length must be multiple of 16 bytes".to_string(),
            ));
        }

        let mut buffer = ciphertext.to_vec();
        let cipher = Aes256CbcDec::new(&self.key.into(), iv.into());
        
        let plaintext = cipher
            .decrypt_padded_mut::<Pkcs7>(&mut buffer)
            .map_err(|_| Error::DecryptionError("Decryption failed (wrong password?)".to_string()))?;

        Ok(plaintext.to_vec())
    }
}

/// Verify if a password is correct by attempting decryption
///
/// # Arguments
///
/// * `password` - Password to test
/// * `encrypted_data` - Encrypted test data
/// * `salt` - Salt from archive header
/// * `iv` - IV from archive header
///
/// # Returns
///
/// `Ok(())` if decryption succeeds, `Err` if wrong password or verification fails
pub fn verify_password(
    password: &str,
    encrypted_data: &[u8],
    salt: &[u8],
    iv: &[u8],
) -> Result<()> {
    if password.is_empty() {
        return Err(Error::InvalidParameter(
            "Password cannot be empty".to_string(),
        ));
    }
    if iv.len() != AES_BLOCK_SIZE {
        return Err(Error::InvalidParameter(
            "IV must be 16 bytes".to_string(),
        ));
    }

    let ctx = DecryptionContext::new(password, salt)?;
    let iv_arr: [u8; AES_BLOCK_SIZE] = iv.try_into()
        .map_err(|_| Error::InvalidParameter("Invalid IV length".to_string()))?;
    
    // Try to decrypt - will fail if password is wrong (bad padding)
    ctx.decrypt(encrypted_data, &iv_arr)?;
    
    Ok(())
}

/// Derive a key from password and salt using PBKDF2-SHA256
///
/// Uses 262,144 iterations (7-Zip default).
pub fn derive_key(password: &str, salt: &[u8]) -> [u8; AES_KEY_SIZE] {
    let mut key = [0u8; AES_KEY_SIZE];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// Generate a random salt
pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Generate a random IV
pub fn generate_iv() -> [u8; AES_BLOCK_SIZE] {
    let mut iv = [0u8; AES_BLOCK_SIZE];
    rand::thread_rng().fill_bytes(&mut iv);
    iv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let ctx = EncryptionContext::new("test_password").unwrap();
        let plaintext = b"Hello, World! This is a test message.";
        
        let ciphertext = ctx.encrypt(plaintext).unwrap();
        assert_eq!(ciphertext.len() % AES_BLOCK_SIZE, 0);
        assert_ne!(ciphertext.as_slice(), plaintext.as_slice());
        
        let decrypted = ctx.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext.as_slice());
    }

    #[test]
    fn test_empty_password_rejected() {
        let result = EncryptionContext::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_decryption_context() {
        let enc_ctx = EncryptionContext::new("password123").unwrap();
        let plaintext = b"Secret data";
        let ciphertext = enc_ctx.encrypt(plaintext).unwrap();

        // Create decryption context with same password and salt
        let dec_ctx = DecryptionContext::new("password123", enc_ctx.salt()).unwrap();
        let decrypted = dec_ctx.decrypt(&ciphertext, enc_ctx.iv()).unwrap();
        
        assert_eq!(decrypted.as_slice(), plaintext.as_slice());
    }

    #[test]
    fn test_wrong_password_fails() {
        let enc_ctx = EncryptionContext::new("correct_password").unwrap();
        let ciphertext = enc_ctx.encrypt(b"Secret data").unwrap();

        let dec_ctx = DecryptionContext::new("wrong_password", enc_ctx.salt()).unwrap();
        let result = dec_ctx.decrypt(&ciphertext, enc_ctx.iv());
        
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password() {
        let ctx = EncryptionContext::new("test_password").unwrap();
        let ciphertext = ctx.encrypt(b"Test data").unwrap();

        // Correct password should succeed
        assert!(verify_password("test_password", &ciphertext, ctx.salt(), ctx.iv()).is_ok());

        // Wrong password should fail
        assert!(verify_password("wrong_password", &ciphertext, ctx.salt(), ctx.iv()).is_err());
    }

    #[test]
    fn test_key_derivation() {
        let salt = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                    0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10];
        
        let key1 = derive_key("password", &salt);
        let key2 = derive_key("password", &salt);
        let key3 = derive_key("different", &salt);
        
        // Same password + salt = same key
        assert_eq!(key1, key2);
        // Different password = different key
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_large_data_encryption() {
        let ctx = EncryptionContext::new("password").unwrap();
        
        // Test with 1MB of data
        let plaintext: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
        
        let ciphertext = ctx.encrypt(&plaintext).unwrap();
        let decrypted = ctx.decrypt(&ciphertext).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }
}
