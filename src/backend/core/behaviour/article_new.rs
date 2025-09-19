#![cfg(feature = "ssr")]

use crate::{
    backend::{
        core::Article,
        database::{ArticleDB, DBError, DatabaseResponse, DB},
    },
    models::Money,
};

impl Article {
    pub async fn new(db: &DB, name: String, cost: Money) -> DatabaseResponse<Self> {
        let mut transaction = db.get_conn_transaction().await?;

        let id = ArticleDB::create(&mut transaction, name, cost.value).await?;

        transaction.commit().await.map_err(DBError::new)?;

        let article = Article::get(db, id).await?;

        Ok(article.expect("Newly created article should exist!"))
    }
}
