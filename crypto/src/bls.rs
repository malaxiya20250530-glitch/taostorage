#[allow(dead_code)]
/// BLS 聚合签名 — 预留模块
///
/// 用于"长生久视"中的社群多重签名契约。
/// 当前为占位，后续可对接 blst 或 arkworks。
///
/// 功能规划：
/// - sign(msg, sk) -> Signature
/// - aggregate(sigs) -> AggregatedSignature
/// - verify_aggregate(msgs, agg_sig, pks) -> bool
pub struct BlsSignature([u8; 96]);

impl BlsSignature {
    pub fn placeholder() -> Self {
        Self([0u8; 96])
    }
}
