//! Classification facade. Re-exports the [`Category`] enum (+ `Tier`)
//! and the rule-driven `explain_*` / `classify_*` lookups so callers can
//! `use crate::classify::{Category, classify_dir, ...}` without caring
//! whether each item lives in [`category`] or [`rules`].
//!
//! Wire-format serialization for the types in this module lives in
//! `wire::category` (`Category`) and `wire::explain` (`Classification`,
//! `ClassificationReason`). The domain types here stay derive-free so
//! schema changes are an explicit edit in `wire/`.

pub mod category;
pub mod rules;

pub use category::{Category, Tier};
pub use rules::{
    classify_dir, classify_file, explain_dir, explain_file, Classification, ClassificationReason,
};
