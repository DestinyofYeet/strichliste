use crate::backend::core::Barcode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type, sqlx::FromRow))]
pub struct BarcodeDB {
    pub article_id: i64,
    pub barcode_content: String,
}

impl From<BarcodeDB> for Barcode {
    fn from(value: BarcodeDB) -> Self {
        Self(value.barcode_content)
    }
}
