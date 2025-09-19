#![cfg(feature = "ssr")]

use sqlx::Executor;

use crate::{
    backend::{
        core::Article,
        database::{DatabaseResponse, DatabaseType},
    },
    models::ArticleDB,
};
impl Article {
    pub async fn set_name<T>(&mut self, conn: &mut T, name: String) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        ArticleDB::set_name(conn, self.id, name.clone()).await?;

        self.name = name;

        Ok(())
    }
}
