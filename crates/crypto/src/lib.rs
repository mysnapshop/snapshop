// hash.rs
pub mod hash {
    use bcrypt::{hash, verify, BcryptError};
    static COST: u32 = 10;

    /// Generate bcrypt hash from string
    pub fn make(s: &str) -> Result<String, BcryptError> {
        hash(s, COST)
    }

    /// Verify bcrypt hash from string
    pub fn check(s: &str, h: &str) -> bool {
        verify(s, h).unwrap_or(false)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_hash_and_check() {
            let password = "testpassword";
            let hashed = make(password).unwrap();
            assert!(check(password, &hashed));
            assert!(!check("wrongpassword", &hashed));
        }
    }
}

// crypto_aes.rs
pub mod crypto_aes {
    use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
    use aes::Aes256;
    use once_cell::sync::Lazy;
    use std::{
        env,
        sync::{Arc, Mutex},
    };

    type Aes256CbcEnc = cbc::Encryptor<Aes256>;
    type Aes256CbcDec = cbc::Decryptor<Aes256>;

    static CIPHER: Lazy<Arc<Mutex<Aes>>> = Lazy::new(|| {
        let secret: [u8; 32] = env::var("AES_KEY")
            .unwrap_or_else(|_| panic!("Failed to retrieve AES_KEY"))
            .as_bytes()
            .try_into()
            .unwrap();
        let iv: [u8; 16] = env::var("AES_IV")
            .unwrap_or_else(|_| panic!("Failed to retrieve AES_IV"))
            .as_bytes()
            .try_into()
            .unwrap();

        Arc::new(Mutex::new(Aes {
            encoder: Aes256CbcEnc::new(&secret.into(), &iv.into()),
            decoder: Aes256CbcDec::new(&secret.into(), &iv.into()),
        }))
    });

    struct Aes {
        encoder: Aes256CbcEnc,
        decoder: Aes256CbcDec,
    }

    impl Aes {
        fn encode(&self, text: &[u8]) -> Vec<u8> {
            self.encoder.clone().encrypt_padded_vec_mut::<Pkcs7>(text)
        }

        fn decode(&self, text: &[u8]) -> Vec<u8> {
            self.decoder
                .clone()
                .decrypt_padded_vec_mut::<Pkcs7>(text)
                .unwrap()
        }
    }

    pub fn encode(text: &[u8]) -> Vec<u8> {
        CIPHER.lock().unwrap().encode(text)
    }

    pub fn decode(buf: &[u8]) -> Vec<u8> {
        CIPHER.lock().unwrap().decode(buf)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_encode_and_decode() {
            let plaintext = b"teststring123";
            let encrypted = encode(plaintext);
            let decrypted = decode(&encrypted);
            assert_eq!(plaintext.to_vec(), decrypted);
        }

        #[test]
        fn test_invalid_decryption() {
            let invalid_data = b"invalid_data";
            let decrypted = decode(invalid_data);
            assert!(decrypted.is_empty()); // Ensure it's empty or invalid
        }
    }
}

// base64
pub mod base64 {
    use base64::{prelude::*, DecodeError};
    use rand::Rng;
    // Function to encode data to base64
    pub fn encode<T: AsRef<[u8]>>(i: T) -> String {
        BASE64_STANDARD.encode(i)
    }

    // Function to decode base64 encoded data
    pub fn decode<T: AsRef<[u8]>>(i: T) -> Result<Vec<u8>, DecodeError> {
        BASE64_STANDARD.decode(i)
    }

    // Function to generate a random base64 string of a given length
    pub fn random(length: usize) -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
        encode(random_bytes)
    }

    // Tests for the base64 functions
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_base64_encode_and_decode() {
            let plaintext = b"base64string";
            let encoded = encode(plaintext);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(plaintext.to_vec(), decoded);
        }

        #[test]
        fn test_invalid_base64_decode() {
            let invalid_base64 = "invalid_base64==";
            let result = decode(invalid_base64);
            assert!(result.is_err()); // Ensure it errors
        }

        // Test for random base64 string generation
        #[test]
        fn test_random_base64() {
            let random_string = random(16); // Generate a random base64 string of length 16
            assert_eq!(random_string.len() % 4, 0); // Base64 string length should be a multiple of 4
        }
    }
}
