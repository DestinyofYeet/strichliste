use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Barcode(pub String);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BarcodeDiff {
    Removed(String),
    Added(String),
}
