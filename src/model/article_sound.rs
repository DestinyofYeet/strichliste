use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type, sqlx::FromRow))]
pub struct ArticleSound {
    pub id: i64,
    pub name: String,
    pub path: String,
}
