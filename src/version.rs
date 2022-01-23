use std::cmp::{PartialEq, PartialOrd, Ordering};

use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Version(pub u32, pub u32, pub u32);

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 > other.0 {
            return Some(Ordering::Greater);
        }

        if self.0 < other.0 {
            return Some(Ordering::Less);
        }

        if self.1 > other.1 {
            return Some(Ordering::Greater);
        }

        if self.1 < other.1 {
            return Some(Ordering::Less);
        }

        if self.2 > other.2 {
            return Some(Ordering::Greater);
        }

        if self.2 < other.2 {
            return Some(Ordering::Less);
        }

        Some(Ordering::Equal)
    }
}
