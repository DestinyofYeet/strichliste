#[cfg(feature = "ssr")]
use sqlx::Executor;

#[cfg(feature = "ssr")]
use crate::{
    backend::database::{ArticleDB, DatabaseResponse, DatabaseType},
    models::DatabaseId,
};

#[cfg(feature = "ssr")]
impl ArticleDB {
    pub async fn set_price<T>(
        conn: &mut T,
        article_id: DatabaseId,
        cost: i64,
    ) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        use chrono::Utc;
        use sqlx::query;

        use crate::backend::database::DBError;

        let now = Utc::now();
        _ = query!(
            "
                insert into ArticleCostMap
                    (article_id, cost, effective_since)
                values
                    (?, ?, ?)
            ",
            article_id,
            cost,
            now
        )
        .execute(&mut *conn)
        .await
        .map_err(DBError::new)?;

        Ok(())
    }
}
