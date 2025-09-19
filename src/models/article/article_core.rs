use crate::backend::core::Article;
#[cfg(feature = "ssr")]
use crate::backend::database::ArticleDB;

#[cfg(feature = "ssr")]
use {
    crate::backend::database::DBError,
    crate::backend::database::{DatabaseResponse, DatabaseType},
    crate::models::{DatabaseId, UserId},
    sqlx::query,
    sqlx::Executor,
};

impl Article {
    //TODO move this to the relevant component directly or to a constants file in frontend
    pub const DEFAULT_ARTICLE_AMOUNT: usize = 9;
}

#[cfg(feature = "ssr")]
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
            sqlx::Error::Database(e) => {
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

    /// returns the article_id and amount of items bought for the user
    pub async fn get_articles_for_user<T>(
        conn: &mut T,
        user_id: UserId,
    ) -> DatabaseResponse<Vec<(i64, i64)>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        use crate::backend::database::DBGROUP_SNACKBAR_ID;

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
