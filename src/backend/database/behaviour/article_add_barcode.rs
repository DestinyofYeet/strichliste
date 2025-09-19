#![cfg(feature = "ssr")]

use sqlx::{query, Error, Executor};

use crate::{
    backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType},
    models::DatabaseId,
};

impl ArticleDB {
    pub async fn add_barcode<T>(
        conn: &mut T,
        article_id: DatabaseId,
        barcode: String,
    ) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        _ = query!(
            "
                insert into ArticleBarcodes
                    (article_id, barcode_content)
                values
                    (?, ?)
            ",
            article_id,
            barcode
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| match e {
            Error::Database(e) => {
                if e.is_unique_violation() {
                    DBError::new(format!(
                        "The barcode '{}' is already used elsewhere!",
                        barcode
                    ))
                } else {
                    DBError::new(e)
                }
            }

            _ => DBError::new(e),
        })?;

        Ok(())
    }
}
