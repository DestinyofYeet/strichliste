#![cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, sqlx::Type, sqlx::FromRow)]
pub struct BarcodeDB {
    pub article_id: i64,
    pub barcode_content: String,
}
