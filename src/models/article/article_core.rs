#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};

use crate::backend::core::Article;
#[cfg(feature = "ssr")]
use crate::models::Money;

#[cfg(feature = "ssr")]
use super::{ArticleSound, Barcode};

#[cfg(feature = "ssr")]
use {
    super::{BarcodeDB, BarcodeDiff},
    crate::backend::database::{DBError, DB},
    crate::backend::database::{DatabaseResponse, DatabaseType},
    crate::models::{DatabaseId, UserId},
    chrono::{DateTime, Utc},
    sqlx::query,
    sqlx::query_as,
    sqlx::Executor,
    sqlx::Transaction,
};

impl Article {
    //TODO move this to the relevant component directly or to a constants file in frontend
    pub const DEFAULT_ARTICLE_AMOUNT: usize = 9;
}

#[cfg(feature = "ssr")]
impl Article {
    pub async fn get(db: &DB, id: i64) -> DatabaseResponse<Option<Self>> {
        let mut conn = db.get_conn().await?;

        match ArticleDB::get_single(&mut *conn, id).await? {
            Some(article) => {
                let article_sounds = ArticleDB::get_sounds(&mut *conn, article.id).await?;
                let article_barcodes = ArticleDB::get_barcodes(&mut *conn, article.id)
                    .await?
                    .into_iter()
                    .map(|elem| Barcode(elem.barcode_content))
                    .collect();

                let cost = ArticleDB::get_latest_cost(&mut *conn, article.id).await?;

                let ArticleDB { id, name } = article;
                Ok(Some(Article {
                    id,
                    name,
                    cost: cost.into(),
                    sounds: article_sounds,
                    barcodes: article_barcodes,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn get_by_barcode(db: &DB, barcode: String) -> DatabaseResponse<Option<Article>> {
        let mut conn = db.get_conn().await?;

        let result = ArticleDB::get_article_id_by_barcode(&mut *conn, barcode).await?;

        match result {
            None => Ok(None),
            Some(value) => {
                let article = Article::get(db, value).await?;
                Ok(article)
            }
        }
    }

    pub async fn set_name<T>(&mut self, conn: &mut T, name: String) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        ArticleDB::set_name(conn, self.id, name.clone()).await?;

        self.name = name;

        Ok(())
    }

    pub async fn set_cost<T>(&mut self, conn: &mut T, cost: Money) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        ArticleDB::set_price(conn, self.id, cost.value).await?;

        self.cost = cost;

        Ok(())
    }

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

    pub async fn get_articles_for_user(db: &DB, user_id: UserId) -> DatabaseResponse<Vec<Self>> {
        let mut conn = db.get_conn().await?;

        let mut articles_amount_bought =
            ArticleDB::get_articles_for_user(&mut *conn, user_id).await?;

        //sort by most bought
        articles_amount_bought.sort_by(|a, b| b.1.cmp(&a.1));

        let mut full_articles = Vec::<Article>::new();

        for (article_id, _amount_bought) in articles_amount_bought.iter() {
            full_articles.push(
                Article::get(db, *article_id)
                    .await?
                    .expect("fetched article should exist!"),
            );
        }

        let mut articles = Self::get_all(db, None).await?;

        for article in full_articles.iter() {
            articles.retain(|value| value.id != article.id);
        }

        full_articles.reverse();

        for article in full_articles.into_iter() {
            articles.insert(0, article);
        }

        Ok(articles)
    }
}

#[cfg(feature = "ssr")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, sqlx::Type, sqlx::FromRow)]
pub struct ArticleDB {
    pub id: DatabaseId,
    pub name: String,
}

#[cfg(feature = "ssr")]
impl ArticleDB {
    pub async fn get_single<T>(
        conn: &mut T,
        article_id: DatabaseId,
    ) -> DatabaseResponse<Option<Self>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let result = query_as!(
            Self,
            "
                select * from Articles
                where id = ?
            ",
            article_id
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(DBError::new)?;

        Ok(result)
    }
    pub async fn create<'a>(
        conn: &mut Transaction<'a, DatabaseType>,
        name: String,
        cost: i64,
    ) -> DatabaseResponse<DatabaseId> {
        let id = Self::_insert_name(conn, name).await?;
        Self::set_price(&mut **conn, id, cost).await?;

        Ok(id)
    }

    pub async fn set_price<T>(
        conn: &mut T,
        article_id: DatabaseId,
        cost: i64,
    ) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
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

    pub async fn get_sounds<T>(
        conn: &mut T,
        article_id: DatabaseId,
    ) -> DatabaseResponse<Vec<ArticleSound>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let sound_ids = query!(
            "
            select sound_id from ArticleSoundMap
            where article_id = ?
                
            ",
            article_id
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(DBError::new)?;

        let mut sounds = Vec::new();
        for sound_id in sound_ids {
            let sound = query_as!(
                ArticleSound,
                "
                    select * from ArticleSounds
                    where id = ?
                ",
                sound_id.sound_id
            )
            .fetch_one(&mut *conn)
            .await
            .map_err(DBError::new)?;
            sounds.push(sound);
        }
        Ok(sounds)
    }

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

    pub async fn get_barcodes<T>(
        conn: &mut T,
        article_id: DatabaseId,
    ) -> DatabaseResponse<Vec<BarcodeDB>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        query_as!(
            BarcodeDB,
            "
                select * from ArticleBarcodes
                where article_id = ?
            ",
            article_id
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(DBError::new)
    }

    pub async fn set_name<T>(
        conn: &mut T,
        article_id: DatabaseId,
        name: String,
    ) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        _ = query!(
            "
                update Articles
                    set name = ?
                where id = ?
            ",
            name,
            article_id,
        )
        .execute(&mut *conn)
        .await
        .map_err(DBError::new)?;

        Ok(())
    }

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
