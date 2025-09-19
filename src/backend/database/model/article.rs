#![cfg(feature = "ssr")]

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

use crate::models::DatabaseId;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Type, FromRow)]
pub struct ArticleDB {
    pub id: DatabaseId,
    pub name: String,
}
