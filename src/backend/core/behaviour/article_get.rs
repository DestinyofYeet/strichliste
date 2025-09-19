#![cfg(feature = "ssr")]

use crate::{
    backend::{
        core::Article,
        database::{ArticleDB, DatabaseResponse, DB},
    },
    models::Barcode,
};
impl Article {
    pub async fn get(db: &DB, id: i64) -> DatabaseResponse<Option<Self>> {
        let mut conn = db.get_conn().await?;

        match ArticleDB::get_single(&mut *conn, id).await? {
            Some(article) => {
                let article_sounds = ArticleDB::get_sounds(&mut *conn, article.id).await?;
                let article_barcodes = ArticleDB::get_barcodes(&mut *conn, article.id)
                    .await?
                    .into_iter()
                    .map(|elem| Barcode(elem.barcode_content))
                    .collect();

                let cost = ArticleDB::get_latest_cost(&mut *conn, article.id).await?;

                let ArticleDB { id, name } = article;
                Ok(Some(Article {
                    id,
                    name,
                    cost: cost.into(),
                    sounds: article_sounds,
                    barcodes: article_barcodes,
                }))
            }
            None => Ok(None),
        }
    }
}
