use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use x25519_dalek::{EphemeralSecret, PublicKey};

/// 对称加密：ChaCha20-Poly1305
///
/// 用于数据单元在离设备前的加密，实现零知识存储的基础。
/// 存储节点只看到密文，不知明文。
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new_from_slice(key).expect("valid key size");
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .expect("encryption succeeds");

    // 前缀 nonce（12 bytes）+ ciphertext
    let mut output = Vec::with_capacity(12 + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    output
}

/// 对称解密
pub fn decrypt(encrypted: &[u8], key: &[u8; 32]) -> Option<Vec<u8>> {
    if encrypted.len() < 12 {
        return None;
    }
    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let cipher = ChaCha20Poly1305::new_from_slice(key).ok()?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).ok()
}

/// 生成 Ed25519 密钥对（用于签名/身份）
pub fn generate_ed25519_keypair() -> ed25519_dalek::SigningKey {
    ed25519_dalek::SigningKey::generate(&mut OsRng)
}

/// 生成 X25519 共享密钥（用于 ECDH 密钥协商）
pub fn generate_shared_secret(their_public: &PublicKey) -> (PublicKey, [u8; 32]) {
    let my_secret = EphemeralSecret::random_from_rng(OsRng);
    let my_public = PublicKey::from(&my_secret);
    let shared = my_secret.diffie_hellman(their_public);
    (my_public, *shared.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let plaintext = "zhi zhe bu yan — the server knows nothing".as_bytes();
        let encrypted = encrypt(plaintext, &key);
        let decrypted = decrypt(&encrypted, &key).expect("decrypt should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        let key = [0x42u8; 32];
        let plaintext = b"same plaintext";
        let c1 = encrypt(plaintext, &key);
        let c2 = encrypt(plaintext, &key);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = [0x42u8; 32];
        let mut encrypted = encrypt(b"secret", &key);
        encrypted[0] ^= 0xff;
        assert!(decrypt(&encrypted, &key).is_none());
    }
}
