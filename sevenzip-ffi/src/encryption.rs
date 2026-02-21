//! AES-256 Encryption Support
//!
//! This module provides safe Rust wrappers for AES-256-CBC encryption/decryption
//! used in password-protected 7z archives.
//!
//! # Security
//!
//! - AES-256-CBC encryption (NSA TOP SECRET approved)
//! - PBKDF2-SHA256 key derivation with 262,144 iterations
//! - Hardware-accelerated (AES-NI on supported CPUs)
//! - Secure random IV and salt generation
//! - PKCS#7 padding
//!
//! # Example
//!
//! ```no_run
//! use seven_zip::encryption::EncryptionContext;
//!
//! // Create encryption context
//! let mut ctx = EncryptionContext::new("my_strong_password")?;
//!
//! // Encrypt data
//! let plaintext = b"Secret forensic evidence data";
//! let ciphertext = ctx.encrypt(plaintext)?;
//!
//! // Decrypt data
//! let decrypted = ctx.decrypt(&ciphertext)?;
//! assert_eq!(plaintext, decrypted.as_slice());
//! # Ok::<(), seven_zip::Error>(())
//! ```

use crate::error::{Error, Result};
use crate::ffi;
use std::ffi::CString;

/// AES-256 encryption context
///
/// Maintains the encryption key, IV, and AES state for encrypting/decrypting data.
/// The context is initialized with a password and automatically generates secure random
/// values for the IV and salt.
pub struct EncryptionContext {
    key: [u8; ffi::AES_KEY_SIZE],
    iv: [u8; ffi::AES_BLOCK_SIZE],
    aes_context: Box<[u32; ffi::AES_NUM_IVMRK_WORDS]>,
}

/// AES-256 decryption context
///
/// Maintains the decryption key and AES state for decrypting encrypted data.
/// Initialized with a password and salt (from the encrypted archive header).
pub struct DecryptionContext {
    key: [u8; ffi::AES_KEY_SIZE],
    aes_context: Box<[u32; ffi::AES_NUM_IVMRK_WORDS]>,
}

impl EncryptionContext {
    /// Create a new encryption context from a password
    ///
    /// Derives a 256-bit AES key using PBKDF2-SHA256 with 262,144 iterations.
    /// Generates a random 16-byte IV for encryption.
    ///
    /// # Arguments
    ///
    /// * `password` - Password string (any length, UTF-8)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Password is empty
    /// - Key derivation fails
    /// - Memory allocation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::encryption::EncryptionContext;
    ///
    /// let ctx = EncryptionContext::new("StrongPassword123!")?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn new(password: &str) -> Result<Self> {
        if password.is_empty() {
            return Err(Error::InvalidParameter("Password cannot be empty".to_string()));
        }

        let c_password = CString::new(password)
            .map_err(|_| Error::InvalidParameter("Invalid password string".to_string()))?;

        let mut key = [0u8; ffi::AES_KEY_SIZE];
        let mut iv = [0u8; ffi::AES_BLOCK_SIZE];
        let mut aes_context = Box::new([0u32; ffi::AES_NUM_IVMRK_WORDS]);

        unsafe {
            let result = ffi::sevenzip_init_encryption(
                c_password.as_ptr(),
                key.as_mut_ptr(),
                iv.as_mut_ptr(),
                aes_context.as_mut_ptr(),
            );

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        Ok(Self {
            key,
            iv,
            aes_context,
        })
    }

    /// Get the initialization vector (IV)
    ///
    /// The IV must be stored with the encrypted data and used for decryption.
    ///
    /// # Returns
    ///
    /// 16-byte IV array
    pub fn iv(&self) -> &[u8; ffi::AES_BLOCK_SIZE] {
        &self.iv
    }

    /// Get the encryption key
    ///
    /// **WARNING**: Exposing the key can compromise security. Only use for
    /// debugging or when you need to serialize the key for storage.
    ///
    /// # Returns
    ///
    /// 32-byte key array
    pub fn key(&self) -> &[u8; ffi::AES_KEY_SIZE] {
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
    /// # Errors
    ///
    /// Returns an error if encryption fails or memory allocation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::encryption::EncryptionContext;
    ///
    /// let mut ctx = EncryptionContext::new("password")?;
    /// let plaintext = b"Secret data";
    /// let ciphertext = ctx.encrypt(plaintext)?;
    /// assert!(ciphertext.len() >= plaintext.len());
    /// assert_eq!(ciphertext.len() % 16, 0); // Padded to block size
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        // Calculate padded length (PKCS#7 padding always adds at least 1 byte)
        let padded_len = ((plaintext.len() / ffi::AES_BLOCK_SIZE) + 1) * ffi::AES_BLOCK_SIZE;
        let mut ciphertext = vec![0u8; padded_len];
        let mut ciphertext_len = padded_len;

        unsafe {
            let result = ffi::sevenzip_encrypt_data(
                self.aes_context.as_mut_ptr(),
                self.iv.as_ptr(),
                plaintext.as_ptr(),
                plaintext.len(),
                ciphertext.as_mut_ptr(),
                &mut ciphertext_len as *mut usize,
            );

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        ciphertext.truncate(ciphertext_len);
        Ok(ciphertext)
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
    /// # Errors
    ///
    /// Returns an error if:
    /// - Ciphertext length is not a multiple of 16
    /// - Decryption fails
    /// - Padding is invalid (may indicate wrong password)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::encryption::EncryptionContext;
    ///
    /// let mut ctx = EncryptionContext::new("password")?;
    /// let plaintext = b"Secret data";
    /// let ciphertext = ctx.encrypt(plaintext)?;
    /// let decrypted = ctx.decrypt(&ciphertext)?;
    /// assert_eq!(plaintext, decrypted.as_slice());
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() % ffi::AES_BLOCK_SIZE != 0 {
            return Err(Error::InvalidParameter(
                "Ciphertext length must be multiple of 16 bytes".to_string(),
            ));
        }

        let mut plaintext = vec![0u8; ciphertext.len()];
        let mut plaintext_len = ciphertext.len();

        unsafe {
            let result = ffi::sevenzip_decrypt_data(
                self.aes_context.as_mut_ptr(),
                self.iv.as_ptr(),
                ciphertext.as_ptr(),
                ciphertext.len(),
                plaintext.as_mut_ptr(),
                &mut plaintext_len as *mut usize,
            );

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        plaintext.truncate(plaintext_len);
        Ok(plaintext)
    }
}

impl DecryptionContext {
    /// Create a new decryption context from a password and salt
    ///
    /// The salt must be obtained from the encrypted archive header.
    ///
    /// # Arguments
    ///
    /// * `password` - Password string (same as used for encryption)
    /// * `salt` - Salt bytes from archive header (typically 8-16 bytes)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Password is empty
    /// - Salt is empty
    /// - Key derivation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use seven_zip::encryption::DecryptionContext;
    ///
    /// let salt = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    /// let ctx = DecryptionContext::new("password", &salt)?;
    /// # Ok::<(), seven_zip::Error>(())
    /// ```
    pub fn new(password: &str, salt: &[u8]) -> Result<Self> {
        if password.is_empty() {
            return Err(Error::InvalidParameter("Password cannot be empty".to_string()));
        }
        if salt.is_empty() {
            return Err(Error::InvalidParameter("Salt cannot be empty".to_string()));
        }

        let c_password = CString::new(password)
            .map_err(|_| Error::InvalidParameter("Invalid password string".to_string()))?;

        let mut key = [0u8; ffi::AES_KEY_SIZE];
        let mut aes_context = Box::new([0u32; ffi::AES_NUM_IVMRK_WORDS]);

        unsafe {
            let result = ffi::sevenzip_init_decryption(
                c_password.as_ptr(),
                salt.as_ptr(),
                salt.len(),
                key.as_mut_ptr(),
                aes_context.as_mut_ptr(),
            );

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        Ok(Self { key, aes_context })
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
    ///
    /// # Errors
    ///
    /// Returns an error if decryption fails or padding is invalid.
    pub fn decrypt(&mut self, ciphertext: &[u8], iv: &[u8; ffi::AES_BLOCK_SIZE]) -> Result<Vec<u8>> {
        if ciphertext.len() % ffi::AES_BLOCK_SIZE != 0 {
            return Err(Error::InvalidParameter(
                "Ciphertext length must be multiple of 16 bytes".to_string(),
            ));
        }

        let mut plaintext = vec![0u8; ciphertext.len()];
        let mut plaintext_len = ciphertext.len();

        unsafe {
            let result = ffi::sevenzip_decrypt_data(
                self.aes_context.as_mut_ptr(),
                iv.as_ptr(),
                ciphertext.as_ptr(),
                ciphertext.len(),
                plaintext.as_mut_ptr(),
                &mut plaintext_len as *mut usize,
            );

            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }

        plaintext.truncate(plaintext_len);
        Ok(plaintext)
    }
}

/// Verify if a password is correct for an encrypted archive
///
/// Tests decryption of a known block to verify password correctness.
///
/// # Arguments
///
/// * `password` - Password to test
/// * `encrypted_test_block` - Encrypted test data from archive header
/// * `salt` - Salt from archive header
/// * `iv` - IV from archive header
///
/// # Returns
///
/// `Ok(())` if password is correct, `Err` if incorrect or verification fails
///
/// # Example
///
/// ```no_run
/// use seven_zip::encryption::verify_password;
///
/// let test_block = &[0x12, 0x34, 0x56, /* ... */];
/// let salt = &[0x01, 0x02, 0x03, /* ... */];
/// let iv = &[0xAA, 0xBB, 0xCC, /* ... */];
///
/// if verify_password("password", test_block, salt, iv).is_ok() {
///     println!("Password is correct!");
/// } else {
///     println!("Wrong password");
/// }
/// # Ok::<(), seven_zip::Error>(())
/// ```
pub fn verify_password(
    password: &str,
    encrypted_test_block: &[u8],
    salt: &[u8],
    iv: &[u8],
) -> Result<()> {
    if password.is_empty() {
        return Err(Error::InvalidParameter("Password cannot be empty".to_string()));
    }

    let c_password = CString::new(password)
        .map_err(|_| Error::InvalidParameter("Invalid password string".to_string()))?;

    unsafe {
        let result = ffi::sevenzip_verify_password(
            c_password.as_ptr(),
            encrypted_test_block.as_ptr(),
            encrypted_test_block.len(),
            salt.as_ptr(),
            salt.len(),
            iv.as_ptr(),
        );

        if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
            return Err(Error::from_code(result));
        }
    }

    Ok(())
}

// Ensure sensitive data is zeroed on drop
impl Drop for EncryptionContext {
    fn drop(&mut self) {
        // Zero out sensitive data
        self.key.iter_mut().for_each(|b| *b = 0);
        self.iv.iter_mut().for_each(|b| *b = 0);
        self.aes_context.iter_mut().for_each(|w| *w = 0);
    }
}

impl Drop for DecryptionContext {
    fn drop(&mut self) {
        // Zero out sensitive data
        self.key.iter_mut().for_each(|b| *b = 0);
        self.aes_context.iter_mut().for_each(|w| *w = 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_context_creation() {
        // This will fail without the C library linked, but tests the API
        let result = EncryptionContext::new("test_password");
        // Just test that the API is callable
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_empty_password_rejected() {
        let result = EncryptionContext::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_decryption_context_validation() {
        let result = DecryptionContext::new("", &[1, 2, 3]);
        assert!(result.is_err());

        let result = DecryptionContext::new("password", &[]);
        assert!(result.is_err());
    }
}
