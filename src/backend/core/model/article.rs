use serde::{Deserialize, Serialize};

use crate::models::{ArticleSound, Barcode, Money};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Article {
    pub id: i64,
    pub name: String,
    pub cost: Money,
    pub sounds: Vec<ArticleSound>,
    pub barcodes: Vec<Barcode>,
}
