#![cfg(feature = "ssr")]
use sqlx::Executor;

use crate::{backend::database::ArticleDB, models::Barcode};
use crate::{
    backend::{
        core::Article,
        database::{DatabaseResponse, DatabaseType},
    },
    models::BarcodeDiff,
};

impl Article {
    pub async fn set_barcodes<T>(
        &mut self,
        conn: &mut T,
        barcode_diff: Vec<BarcodeDiff>,
    ) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        for diff in barcode_diff.into_iter() {
            match diff {
                BarcodeDiff::Removed(barcode) => {
                    ArticleDB::remove_barcode(&mut *conn, self.id, barcode).await?;
                }

                BarcodeDiff::Added(barcode) => {
                    ArticleDB::add_barcode(&mut *conn, self.id, barcode).await?;
                }
            }
        }

        self.barcodes = ArticleDB::get_barcodes(&mut *conn, self.id)
            .await?
            .into_iter()
            .map(|e| Barcode(e.barcode_content))
            .collect();

        Ok(())
    }
}
