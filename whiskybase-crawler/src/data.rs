use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct WeakDate {
    pub year: u16,
    pub month: Option<u8>,
    pub day: Option<u8>,
}
