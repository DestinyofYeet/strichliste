#![cfg(feature = "ssr")]

use crate::backend::{core::Barcode, database::BarcodeDB};

impl From<BarcodeDB> for Barcode {
    fn from(value: BarcodeDB) -> Self {
        Self(value.barcode_content)
    }
}
