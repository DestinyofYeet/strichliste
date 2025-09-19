#![cfg(feature = "ssr")]

use sqlx::{query, Executor};

use crate::{
    backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType},
    models::DatabaseId,
};

impl ArticleDB {
    pub async fn get_article_id_by_barcode<T>(
        conn: &mut T,
        barcode: String,
    ) -> DatabaseResponse<Option<DatabaseId>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let result = query!(
            "
                select article_id from ArticleBarcodes
                where barcode_content = ?
            ",
            barcode
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(DBError::new)?
        .map(|elem| elem.article_id);

        Ok(result)
    }
}
