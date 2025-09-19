#![cfg(feature = "ssr")]

use sqlx::{query, Executor};

use crate::{
    backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType, DBGROUP_SNACKBAR_ID},
    models::UserId,
};

impl ArticleDB {
    pub async fn get_articles_for_user<T>(
        conn: &mut T,
        user_id: UserId,
    ) -> DatabaseResponse<Vec<(i64, i64)>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let result = query!(
            "
                select
                    t_type_data as article_id, count(id) as amount
                from
                    Transactions
                where
                    sender = ? and is_undone = 0 and receiver = ?
                group by t_type_data
                order by timestamp desc
                limit 50
            ",
            user_id.0,
            DBGROUP_SNACKBAR_ID.0
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(DBError::new)
        .map(|elem| {
            elem.into_iter()
                .map(|value| (value.article_id.unwrap(), value.amount))
                .collect::<Vec<(i64, i64)>>()
        })?;

        Ok(result)
    }
}
