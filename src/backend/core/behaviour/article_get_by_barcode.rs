#![cfg(feature = "ssr")]

use crate::backend::{
    core::Article,
    database::{ArticleDB, DatabaseResponse, DB},
};
impl Article {
    pub async fn get_by_barcode(db: &DB, barcode: String) -> DatabaseResponse<Option<Article>> {
        let mut conn = db.get_conn().await?;

        let result = ArticleDB::get_article_id_by_barcode(&mut *conn, barcode).await?;

        match result {
            None => Ok(None),
            Some(value) => {
                let article = Article::get(db, value).await?;
                Ok(article)
            }
        }
    }
}
