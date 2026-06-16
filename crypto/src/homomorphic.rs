/// 同态计算接口
///
/// "知者不言"的最高境界：在密文上执行计算而不解密。
/// 当前提供 trait 定义 + mock 实现，后续可对接 TFHE-rs。
pub trait HomomorphicEngine: Send + Sync {
    /// 加密整数
    fn encrypt_int(&self, value: i64) -> Vec<u8>;

    /// 同态加法：enc(a) + enc(b) = enc(a + b)
    fn add(&self, a: &[u8], b: &[u8]) -> Vec<u8>;

    /// 同态乘法：enc(a) * enc(b) = enc(a * b)
    fn multiply(&self, a: &[u8], b: &[u8]) -> Vec<u8>;

    /// 解密结果
    fn decrypt_int(&self, ciphertext: &[u8]) -> i64;
}

/// Mock 同态引擎（明文运算，仅用于测试和接口验证）
pub struct MockHomomorphicEngine;

impl HomomorphicEngine for MockHomomorphicEngine {
    fn encrypt_int(&self, value: i64) -> Vec<u8> {
        // 简单：前缀标记 + 明文（仅用于演示接口）
        let mut out = vec![0xFE];
        out.extend_from_slice(&value.to_le_bytes());
        out
    }

    fn add(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        let va = self.decrypt_int(a);
        let vb = self.decrypt_int(b);
        self.encrypt_int(va + vb)
    }

    fn multiply(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        let va = self.decrypt_int(a);
        let vb = self.decrypt_int(b);
        self.encrypt_int(va * vb)
    }

    fn decrypt_int(&self, ciphertext: &[u8]) -> i64 {
        if ciphertext.is_empty() || ciphertext[0] != 0xFE {
            return 0;
        }
        let mut bytes = [0u8; 8];
        let len = (ciphertext.len() - 1).min(8);
        bytes[..len].copy_from_slice(&ciphertext[1..1 + len]);
        i64::from_le_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_homomorphic_add() {
        let engine = MockHomomorphicEngine;
        let a = engine.encrypt_int(42);
        let b = engine.encrypt_int(58);
        let sum = engine.add(&a, &b);
        assert_eq!(engine.decrypt_int(&sum), 100);
    }

    #[test]
    fn test_mock_homomorphic_mul() {
        let engine = MockHomomorphicEngine;
        let a = engine.encrypt_int(7);
        let b = engine.encrypt_int(6);
        let prod = engine.multiply(&a, &b);
        assert_eq!(engine.decrypt_int(&prod), 42);
    }
}
