#![cfg(feature = "ssr")]

use tracing::debug;

use crate::backend::{
    core::{Article, Barcode},
    database::{ArticleDB, DatabaseResponse, DB},
};
impl Article {
    pub async fn get_all(db: &DB, limit: Option<i64>) -> DatabaseResponse<Vec<Self>> {
        let mut conn = db.get_conn().await?;

        let articles = ArticleDB::get_all(&mut *conn, limit).await?;

        let mut article_no_db = Vec::new();
        for article in articles {
            let ArticleDB { id, name } = article;
            let article_sounds = ArticleDB::get_sounds(&mut *conn, id).await?;
            debug!("Fetched sounds");
            let article_barcodes = ArticleDB::get_barcodes(&mut *conn, id)
                .await?
                .into_iter()
                .map(|elem| Barcode(elem.barcode_content))
                .collect();
            debug!("Fetched barcodes");
            let cost = ArticleDB::get_latest_cost(&mut *conn, id).await?;
            debug!("Fetched cost");

            article_no_db.push(Article {
                id,
                name,
                cost: cost.into(),
                sounds: article_sounds,
                barcodes: article_barcodes,
            });
        }
        Ok(article_no_db)
    }
}
