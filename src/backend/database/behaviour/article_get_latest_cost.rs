#![cfg(feature = "ssr")]

use sqlx::{query, Executor};

use crate::{
    backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType},
    models::DatabaseId,
};

impl ArticleDB {
    pub async fn get_latest_cost<T>(conn: &mut T, article_id: DatabaseId) -> DatabaseResponse<i64>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let result = query!(
            "
                select cost from ArticleCostMap
                where article_id = ?
                order by effective_since desc
            ",
            article_id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(DBError::new)?;

        Ok(result.cost)
    }
}
