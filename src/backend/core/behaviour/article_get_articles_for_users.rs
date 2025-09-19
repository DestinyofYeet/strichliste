#![cfg(feature = "ssr")]

use crate::{
    backend::{
        core::Article,
        database::{ArticleDB, DatabaseResponse, DB},
    },
    models::UserId,
};
impl Article {
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
