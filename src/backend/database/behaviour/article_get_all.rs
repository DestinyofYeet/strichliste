#![cfg(feature = "ssr")]

use sqlx::{query_as, Executor};

use crate::backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType};

impl ArticleDB {
    pub async fn get_all<T>(conn: &mut T, limit: Option<i64>) -> DatabaseResponse<Vec<Self>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let result = match limit {
            Some(limit) => query_as!(
                Self,
                "
                        select * from Articles
                        limit ?
                    ",
                limit
            )
            .fetch_all(&mut *conn)
            .await
            .map_err(DBError::new),

            None => query_as!(
                Self,
                "
                    select * from Articles
                "
            )
            .fetch_all(&mut *conn)
            .await
            .map_err(DBError::new),
        };

        result
    }
}
