use crate::unit::Yang;


/// 名实分离中的"名"层：元数据索引
///
/// 用逻辑名称查找内容哈希，支持多标签检索。
/// 当前默认给出一个基础实现。
pub struct NameIndex {
    entries: Vec<Yang>,
}

impl NameIndex {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, yang: Yang) {
        self.entries.push(yang);
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Yang> {
        self.entries.iter().find(|y| y.name == name)
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&Yang> {
        self.entries.iter().filter(|y| y.tags.contains(&tag.to_string())).collect()
    }

    pub fn hottest(&self, n: usize) -> Vec<&Yang> {
        let mut sorted: Vec<&Yang> = self.entries.iter().collect();
        sorted.sort_by_key(|y| std::cmp::Reverse(y.heat));
        sorted.truncate(n);
        sorted
    }
}

impl Default for NameIndex {
    fn default() -> Self {
        Self::new()
    }
}
