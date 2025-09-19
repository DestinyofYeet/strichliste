#![cfg(feature = "ssr")]

use sqlx::Executor;

use crate::{
    backend::{
        core::Article,
        database::{ArticleDB, DatabaseResponse, DatabaseType},
    },
    models::Money,
};

impl Article {
    pub async fn set_cost<T>(&mut self, conn: &mut T, cost: Money) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        ArticleDB::set_price(conn, self.id, cost.value).await?;

        self.cost = cost;

        Ok(())
    }
}
