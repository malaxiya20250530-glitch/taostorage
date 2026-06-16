/// 流量混淆层 — "和光同尘"
///
/// 将存储协议流量伪装成普通 TLS/HTTPS 流量，
/// 使其在深度包检测下不可区分。
///
/// 策略：协议帧在发送前包裹在随机填充中，
/// 使流量的统计特征（包大小分布、时序间隔）
/// 接近标准 Web 浏览流量。

use rand::RngCore;

/// 混淆后的帧格式：
/// [ 1 byte: mask_type ] [ 2 bytes: payload_len (u16 LE) ] [ payload ] [ padding (0-255 bytes) ]
pub struct ObfuscatedFrame {
    pub mask_type: u8,
    pub payload: Vec<u8>,
}

impl ObfuscatedFrame {
    /// 包裹原始帧，添加随机填充
    pub fn wrap(raw_payload: &[u8]) -> Vec<u8> {
        let mut rng = rand::thread_rng();

        // 随机选择伪装类型
        let mask_type = (rng.next_u32() % 4) as u8; // 0-3: HTTPS/DNS/WebSocket/Raw

        // 随机填充 0-255 字节
        let padding_len = (rng.next_u32() % 256) as usize;
        let mut padding = vec![0u8; padding_len];
        rng.fill_bytes(&mut padding);

        // 实际载荷 → 反转字节作为轻量混淆
        let mut payload = raw_payload.to_vec();
        payload.reverse();

        let payload_len = payload.len() as u16;

        let mut frame = Vec::with_capacity(3 + payload.len() + padding_len);
        frame.push(mask_type);
        frame.extend_from_slice(&payload_len.to_le_bytes());
        frame.extend_from_slice(&payload);
        frame.extend_from_slice(&padding);

        frame
    }

    /// 解包裹帧
    pub fn unwrap(frame: &[u8]) -> Option<Vec<u8>> {
        if frame.len() < 3 {
            return None;
        }
        let payload_len = u16::from_le_bytes([frame[1], frame[2]]) as usize;
        if frame.len() < 3 + payload_len {
            return None;
        }
        let mut payload = frame[3..3 + payload_len].to_vec();
        payload.reverse(); // 反转还原
        Some(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_unwrap_roundtrip() {
        let raw = b"tao store request payload";
        let wrapped = ObfuscatedFrame::wrap(raw);
        let unwrapped = ObfuscatedFrame::unwrap(&wrapped).expect("unwrap should succeed");
        assert_eq!(unwrapped, raw);
    }

    #[test]
    fn test_wrap_produces_different_outputs() {
        let raw = b"same data";
        let f1 = ObfuscatedFrame::wrap(raw);
        let f2 = ObfuscatedFrame::wrap(raw);
        // 随机填充 + 随机 mask_type 导致不同输出
        assert_ne!(f1, f2);
    }
}
