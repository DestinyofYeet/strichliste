#![cfg(feature = "ssr")]

use chrono::{DateTime, Utc};
use sqlx::{query, Executor};

use crate::{
    backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType},
    models::DatabaseId,
};

impl ArticleDB {
    pub async fn get_effective_cost<T>(
        conn: &mut T,
        article_id: DatabaseId,
        timestamp: DateTime<Utc>,
    ) -> DatabaseResponse<i64>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let result = query!(
            "
                select cost from ArticleCostMap
                where article_id = ? and effective_since < ?
                order by effective_since desc
                limit 1
            ",
            article_id,
            timestamp
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(DBError::new)?;

        Ok(result.cost)
    }
}
