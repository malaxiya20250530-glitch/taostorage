pub mod error;
pub mod erasure;
pub mod group;
pub mod index;
pub mod metadata;
pub mod qi;
pub mod storage;
pub mod unit;

pub use error::{TaoError, TaoResult};
pub use erasure::{ErasureEncoder, Shard};
pub use group::ErasureGroup;
pub use index::{TagIndex, fuzzy_search, collect_stats, StoreStats, export_backup, import_backup, ImportResult, SearchHit};
pub use metadata::NameIndex;
pub use qi::{decide, QiAction};
pub use storage::LocalStore;
pub use unit::{DataUnit, Hexagram, Qi, Yang, Yin};
