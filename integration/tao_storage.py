#!/usr/bin/env python3
"""
🦀 TaoStorage × 🧠 幻觉检测系统 集成层

将 AI 幻觉检测结果存入去中心化 P2P 存储网络。
检测→存储→审计→追溯，全链路闭环。
"""

import json
import subprocess
import time
import os
from typing import Optional
from datetime import datetime

TAO_BIN = os.path.expanduser("~/taostorage/target/debug/tao")

class TaoStorageClient:
    """TaoStorage Python 客户端 — 封装 CLI 调用"""

    def __init__(self, bin_path: str = TAO_BIN):
        self.bin = bin_path

    def _run(self, *args) -> str:
        """执行 tao 命令"""
        try:
            result = subprocess.run(
                [self.bin] + list(args),
                capture_output=True, text=True, timeout=10
            )
            return result.stdout
        except Exception as e:
            return f"Error: {e}"

    def put(self, key: str, value: str, tags: list[str] = None) -> str:
        """写入一条数据"""
        cmd = ["put", key, value]
        for t in (tags or []):
            cmd.extend(["--tag", t])
        return self._run(*cmd)

    def get(self, key: str) -> Optional[str]:
        """读取数据"""
        return self._run("get", key)

    def search(self, query: str) -> str:
        """搜索"""
        return self._run("search", query)

    def by_tag(self, tag: str) -> str:
        """按标签查询"""
        return self._run("by-tag", tag)

    def stats(self) -> str:
        """统计信息"""
        return self._run("stats")

    def invite_generate(self, node_id: str) -> str:
        """生成邀请码"""
        return self._run("invite", "generate", node_id)

    def reputation(self, node_id: str = None) -> str:
        """查看信誉/排行榜"""
        if node_id:
            return self._run("reputation", node_id)
        return self._run("reputation")


class AuditLogger:
    """审计日志 — 把检测结果写入 TaoStorage"""

    def __init__(self, client: TaoStorageClient = None):
        self.client = client or TaoStorageClient()
        self.session_id = f"session-{int(time.time())}"

    def log_detection(self, query: str, response: str, report: dict):
        """
        记录一次幻觉检测结果

        参数:
            query:   用户提问
            response: AI 回答
            report:  检测报告 (HallucinationReport 结构)
        """
        ts = datetime.utcnow().isoformat()
        key = f"audit:{ts[:19]}"

        entry = {
            "timestamp": ts,
            "session": self.session_id,
            "query": query[:500],
            "response_length": len(response),
            "hallucination_score": report.get("overall_score", 1.0),
            "hallucination_ratio": report.get("hallucination_ratio", 0.0),
            "claims_count": len(report.get("claims", [])),
            "verdicts": [
                {
                    "claim": r.get("claim", "")[:200],
                    "verdict": r.get("verdict", "unknown"),
                    "confidence": r.get("confidence", 0.0)
                }
                for r in report.get("results", [])
            ],
            "warnings": report.get("warnings", []),
        }

        tags = ["audit", "hallucination"]
        score = report.get("overall_score", 1.0)
        if score < 0.5:
            tags.append("critical")
        elif score < 0.8:
            tags.append("suspicious")
        else:
            tags.append("clean")

        # 存完整 JSON
        result = self.client.put(
            key,
            json.dumps(entry, ensure_ascii=False),
            tags=tags
        )
        return key, result

    def log_claim(self, claim_text: str, verdict: str, confidence: float):
        """记录单条声明的验证结果"""
        ts = datetime.utcnow().isoformat()
        key = f"claim:{int(time.time())}"

        entry = {
            "timestamp": ts,
            "claim": claim_text[:500],
            "verdict": verdict,
            "confidence": confidence,
            "session": self.session_id,
        }

        tags = ["claim", verdict]
        self.client.put(key, json.dumps(entry, ensure_ascii=False), tags=tags)
        return key

    def get_recent_audits(self, limit: int = 20) -> list:
        """获取最近的审计记录"""
        raw = self.client.search("audit:")
        # 解析搜索结果
        lines = raw.strip().split("\n")
        results = []
        for line in lines:
            if "audit:" in line:
                parts = line.split()
                if parts:
                    results.append(parts[0])
        return results[:limit]

    def get_critical_alerts(self) -> list:
        """获取严重警告"""
        raw = self.client.by_tag("critical")
        return raw.strip().split("\n") if raw.strip() else []


# ============================================================
# 快速测试
# ============================================================
if __name__ == "__main__":
    print("🦀 TaoStorage × 🧠 幻觉检测 集成测试")
    print("=" * 40)

    tao = TaoStorageClient()
    logger = AuditLogger(tao)

    # 模拟一次检测
    report = {
        "overall_score": 0.35,
        "hallucination_ratio": 0.65,
        "claims": [{"text": "上海人口有5000万"}],
        "results": [
            {"claim": "上海人口有5000万", "verdict": "contradicted", "confidence": 0.92}
        ],
        "warnings": ["数值严重偏离事实: 上海实际人口约2500万"]
    }

    key, result = logger.log_detection(
        query="上海有多少人口？",
        response="上海有5000万人口。",
        report=report
    )
    print(f"✅ 已记录: {key}")
    print(f"\n📊 查看审计记录:")
    print(f"   tao get {key}")
    print(f"   tao by-tag critical")
    print(f"   tao reputation")
