#![cfg(feature = "ssr")]

use sqlx::{query, Executor};

use crate::{
    backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType},
    models::DatabaseId,
};

impl ArticleDB {
    pub async fn set_name<T>(
        conn: &mut T,
        article_id: DatabaseId,
        name: String,
    ) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        _ = query!(
            "
                update Articles
                    set name = ?
                where id = ?
            ",
            name,
            article_id,
        )
        .execute(&mut *conn)
        .await
        .map_err(DBError::new)?;

        Ok(())
    }
}
