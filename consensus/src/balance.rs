use crate::reputation::ReputationTable;
use rand::Rng;

/// 冲气共识算法
pub struct QiConsensus {
    reputation: ReputationTable,
    turbulence: u8,
}

impl QiConsensus {
    pub fn new(turbulence: u8) -> Self {
        Self { reputation: ReputationTable::new(), turbulence }
    }

    pub fn with_reputation(reputation: ReputationTable, turbulence: u8) -> Self {
        Self { reputation, turbulence }
    }

    pub fn select_node(&self, candidates: &[String]) -> Option<String> {
        if candidates.is_empty() { return None; }
        let mut rng = rand::thread_rng();

        let scores: Vec<(String, f64)> = candidates.iter().map(|id| {
            let base = self.reputation.score(id) as f64;
            let normalized = base + 100.0;
            let noise: f64 = rng.gen_range(-(self.turbulence as f64)..(self.turbulence as f64));
            (id.clone(), (normalized + noise).max(0.0))
        }).collect();

        let total: f64 = scores.iter().map(|(_, w)| w).sum();
        if total <= 0.0 { return Some(candidates[rng.gen_range(0..candidates.len())].clone()); }

        let mut threshold: f64 = rng.gen_range(0.0..total);
        for (id, weight) in &scores {
            threshold -= weight;
            if threshold <= 0.0 { return Some(id.clone()); }
        }
        Some(candidates.last().unwrap().clone())
    }

    pub fn reputation_mut(&mut self) -> &mut ReputationTable { &mut self.reputation }
    pub fn reputation(&self) -> &ReputationTable { &self.reputation }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_node_bias() {
        let mut c = QiConsensus::new(5);
        for _ in 0..10 { c.reputation_mut().reward("alpha"); }
        for _ in 0..5 { c.reputation_mut().penalize("gamma"); }

        let ids: Vec<String> = ["alpha", "beta", "gamma"].iter().map(|s| s.to_string()).collect();
        let mut counts = std::collections::HashMap::new();
        for _ in 0..500 {
            if let Some(n) = c.select_node(&ids) {
                *counts.entry(n).or_insert(0) += 1;
            }
        }
        assert!(
            counts.get("alpha").unwrap_or(&0) > counts.get("gamma").unwrap_or(&0),
            "alpha={:?} gamma={:?}", counts.get("alpha"), counts.get("gamma")
        );
    }
}
