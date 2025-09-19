#![cfg(feature = "ssr")]

use sqlx::{query_as, Executor};

use crate::{
    backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType},
    models::DatabaseId,
};
impl ArticleDB {
    pub async fn get_single<T>(
        conn: &mut T,
        article_id: DatabaseId,
    ) -> DatabaseResponse<Option<Self>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let result = query_as!(
            Self,
            "
                select * from Articles
                where id = ?
            ",
            article_id
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(DBError::new)?;

        Ok(result)
    }
}
