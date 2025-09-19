#![cfg(feature = "ssr")]

use sqlx::{query_as, Executor};

use crate::{
    backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType},
    models::{BarcodeDB, DatabaseId},
};

impl ArticleDB {
    pub async fn get_barcodes<T>(
        conn: &mut T,
        article_id: DatabaseId,
    ) -> DatabaseResponse<Vec<BarcodeDB>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        query_as!(
            BarcodeDB,
            "
                select * from ArticleBarcodes
                where article_id = ?
            ",
            article_id
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(DBError::new)
    }
}
