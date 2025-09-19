#![cfg(feature = "ssr")]

use sqlx::{query, Executor};

use crate::{
    backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType},
    models::DatabaseId,
};

impl ArticleDB {
    pub async fn remove_barcode<T>(
        conn: &mut T,
        article_id: DatabaseId,
        barcode: String,
    ) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        _ = query!(
            "
                delete from ArticleBarcodes
                where article_id = ? and barcode_content = ?
            ",
            article_id,
            barcode
        )
        .execute(&mut *conn)
        .await
        .map_err(DBError::new)?;

        Ok(())
    }
}
