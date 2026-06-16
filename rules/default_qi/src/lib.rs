#![allow(dead_code)]
/// 默认气机决策模块 — 编译为 WASM
///
/// 导出 execute_qi 函数，供 wasm_host 调用。
///
/// 参数：
///   hexagram: 当前卦象（0=屯, 1=既济, 2=泰, 3=否, 4=剥, 5=坤）
///   replicas: 当前副本数
///   target: 目标副本数
///
/// 返回值映射：
///   0 = Noop
///   1 = Replicate
///   2 = EmergencyRebuild
///   3 = PromoteToHot
///   4 = DemoteToCold
///   5 = Archive
///   6 = Stabilized

const HEXAGRAM_ZHUN: i32 = 0;
const HEXAGRAM_JIJI: i32 = 1;
const HEXAGRAM_TAI: i32 = 2;
const HEXAGRAM_PI: i32 = 3;
const HEXAGRAM_BO: i32 = 4;

const ACTION_NOOP: i32 = 0;
const ACTION_REPLICATE: i32 = 1;
const ACTION_EMERGENCY_REBUILD: i32 = 2;
const ACTION_PROMOTE_TO_HOT: i32 = 3;
const ACTION_DEMOTE_TO_COLD: i32 = 4;
const ACTION_ARCHIVE: i32 = 5;
const ACTION_STABILIZED: i32 = 6;

#[no_mangle]
pub extern "C" fn execute_qi(hexagram: i32, replicas: i32, target: i32) -> i32 {
    // 剥卦 → 紧急修复
    if hexagram == HEXAGRAM_BO {
        return ACTION_EMERGENCY_REBUILD;
    }

    // 副本不足 → 复制
    if replicas < target {
        return ACTION_REPLICATE;
    }

    // 根据卦象决定
    match hexagram {
        HEXAGRAM_ZHUN if replicas >= target => ACTION_STABILIZED,
        HEXAGRAM_TAI => {
            if replicas < target {
                ACTION_REPLICATE
            } else {
                ACTION_NOOP
            }
        }
        _ => ACTION_NOOP,
    }
}
