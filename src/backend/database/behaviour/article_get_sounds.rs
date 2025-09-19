#![cfg(feature = "ssr")]

use sqlx::{query, query_as, Executor};

use crate::{
    backend::database::{ArticleDB, DBError, DatabaseResponse, DatabaseType},
    models::{ArticleSound, DatabaseId},
};

impl ArticleDB {
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
}
