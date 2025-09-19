#![cfg(feature = "ssr")]
use sqlx::{query, Transaction};

use crate::{
    backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType},
    models::DatabaseId,
};

impl ArticleDB {
    pub async fn create<'a>(
        conn: &mut Transaction<'a, DatabaseType>,
        name: String,
        cost: i64,
    ) -> DatabaseResponse<DatabaseId> {
        let id = Self::_insert_name(conn, name).await?;
        Self::set_price(&mut **conn, id, cost).await?;

        Ok(id)
    }

    async fn _insert_name<'a>(
        conn: &mut Transaction<'a, DatabaseType>,
        name: String,
    ) -> DatabaseResponse<DatabaseId> {
        let result = query!(
            "
                insert into Articles
                    (name)
                values
                    (?)
                returning id
            ",
            name,
        )
        .fetch_one(&mut **conn)
        .await
        .map_err(DBError::new)?
        .id;

        Ok(result)
    }
}
