#!/usr/bin/env python3
"""
🔌 网关钩子 — 将 awareness_gateway.py 的检测结果自动存入 TaoStorage

用法:
    from integration.gateway_hook import tao_audit_hook
    # 在网关处理完请求后调用:
    # tao_audit_hook(query, response, report)
"""

import json
import os
import sys
from typing import Optional

# 添加项目路径
sys.path.insert(0, os.path.expanduser("~/taostorage"))

from integration.tao_storage import TaoStorageClient, AuditLogger

# 全局单例
_client: Optional[TaoStorageClient] = None
_logger: Optional[AuditLogger] = None

def get_logger() -> AuditLogger:
    global _client, _logger
    if _logger is None:
        _client = TaoStorageClient()
        _logger = AuditLogger(_client)
    return _logger

def tao_audit_hook(query: str, response: str, report: dict):
    """
    网关处理钩子 — 每次检测完成自动调用

    在 awareness_gateway.py 中这样用:
    ```python
    from integration.gateway_hook import tao_audit_hook
    # 在得到 result 后:
    tao_audit_hook(query, response_text, report)
    ```
    """
    try:
        logger = get_logger()
        key, _ = logger.log_detection(query, response, report)
        print(f"  🦀 审计日志已存入 TaoStorage: tao get {key}")
    except Exception as e:
        print(f"  ⚠️ TaoStorage 审计失败: {e}")

def tao_record_claim(claim_text: str, verdict: str, confidence: float):
    """记录单条声明验证"""
    try:
        logger = get_logger()
        logger.log_claim(claim_text, verdict, confidence)
    except Exception as e:
        print(f"  ⚠️ 记录声明失败: {e}")


# ============================================================
# 一键修补 awareness_gateway.py
# ============================================================
def patch_gateway(gateway_path: str = None):
    """
    自动修补 awareness_gateway.py，在检测完成后插入 TaoStorage 存储

    用法: python3 -c "from integration.gateway_hook import patch_gateway; patch_gateway()"
    """
    if gateway_path is None:
        gateway_path = os.path.expanduser(
            "~/hallucination_detector/awareness_gateway.py")

    if not os.path.exists(gateway_path):
        print(f"❌ 找不到网关文件: {gateway_path}")
        return False

    with open(gateway_path, 'r') as f:
        source = f.read()

    # 检查是否已修补
    if "tao_audit_hook" in source:
        print("✅ 网关已修补，无需重复操作")
        return True

    # 在文件开头添加导入
    import_line = 'from integration.gateway_hook import tao_audit_hook\n'
    source = import_line + source

    # 在 _forward 方法返回前插入审计调用
    # 找到 result = call_upstream(...) 之后
    old = "            # 记录对话"
    new = """            # 🦀 TaoStorage 审计日志
            try:
                report = getattr(result, 'report', {}) if hasattr(result, 'report') else {}
                if report:
                    tao_audit_hook(user_msg, str(result), report)
            except Exception:
                pass

            # 记录对话"""
    source = source.replace(old, new)

    # 保存修改
    backup_path = gateway_path + ".bak"
    with open(backup_path, 'w') as f:
        f.write(open(gateway_path).read())
    print(f"📦 备份原文件: {backup_path}")

    with open(gateway_path, 'w') as f:
        f.write(source)

    print(f"✅ 已修补: {gateway_path}")
    print(f"   检测结果将自动存入 TaoStorage")
    print(f"   查看: tao by-tag audit")
    return True


if __name__ == "__main__":
    # 测试集成
    print("🧪 TaoStorage ↔ 幻觉检测 集成测试")
    print("=" * 50)
    
    # 测试存储
    report = {
        "overall_score": 0.92,
        "hallucination_ratio": 0.08,
        "claims": [],
        "results": [],
        "warnings": []
    }
    
    hook = tao_audit_hook("测试查询", "这是测试响应", report)
    print()
    
    # 查询
    c = TaoStorageClient()
    print(c.by_tag("audit"))
    print()
    
    # 修补网关
    print("=" * 50)
    print("修补网关:")
    patch_gateway()
