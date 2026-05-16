//! Wire-side serialization for the `Category` domain enum.
//!
//! `Category` itself doesn't derive `Serialize` — placing the impl here
//! keeps the wire format (the snake_case label) defined in one place and
//! lets a CI grep enforce "no `Serialize` derives outside `src/wire/`".
//! The on-wire string is identical to `Category::label()` so existing
//! consumers (UI, JSON / NDJSON output, agent integrations) see no
//! change.

use serde::{Serialize, Serializer};

use crate::category::Category;

impl Serialize for Category {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_to_snake_case_label() {
        // Spot-check both core (`cache`) and extended (`vm_image`,
        // `model_cache`) variants so a future label() rename would fail
        // here as well as in category.rs.
        assert_eq!(
            serde_json::to_string(&Category::Cache).unwrap(),
            "\"cache\""
        );
        assert_eq!(
            serde_json::to_string(&Category::VmImage).unwrap(),
            "\"vm_image\""
        );
        assert_eq!(
            serde_json::to_string(&Category::ModelCache).unwrap(),
            "\"model_cache\""
        );
    }

    #[test]
    fn every_variant_serializes_to_its_label() {
        for &c in Category::ALL {
            let json = serde_json::to_string(&c).unwrap();
            assert_eq!(json, format!("\"{}\"", c.label()));
        }
    }
}
