pub mod behaviour;
pub mod misc;
pub mod model;

pub use model::*;

use std::str::FromStr;

use sqlx::{
    pool::PoolConnection,
    query,
    sqlite::{SqliteConnectOptions, SqlitePool},
    Sqlite, Transaction,
};
use tracing::{debug, info};

use crate::models::{GroupDB, GroupId, UserId};


pub type DatabaseResponse<T> = Result<T, DBError>;

pub type DatabaseType = Sqlite;

pub const DBGROUP_SNACKBAR_ID: GroupId = GroupId(0);
pub const DBGROUP_AUFLADUNG_ID: GroupId = GroupId(1);
pub const DBUSER_SNACKBAR_ID: UserId = UserId(0);
pub const DBUSER_AUFLADUNG_ID: UserId = UserId(1);

pub struct DB {
    pool: SqlitePool,
}

impl DB {
    pub async fn new(path: &str) -> Result<Self, DBError> {
        let path = String::new() + "sqlite://" + path;
        let opts = SqliteConnectOptions::from_str(&path)
            .map_err(|e| DBError::new(e.to_string()))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        info!("Connecting to database: {}", path);
        let pool = SqlitePool::connect_with(opts).await;

        let db = pool.map_err(|e| DBError::new(e.to_string()))?;

        let db = DB { pool: db };

        db.setup().await?;

        Ok(db)
    }

    pub async fn get_conn(&self) -> Result<PoolConnection<Sqlite>, DBError> {
        self.pool
            .acquire()
            .await
            .map_err(|e| DBError::new(e.to_string()))
    }

    pub async fn get_conn_transaction(&'_ self) -> Result<Transaction<'_, Sqlite>, DBError> {
        self.pool.begin().await.map_err(DBError::new)
    }

    async fn setup(&self) -> Result<(), DBError> {
        let mut transaction = self.get_conn_transaction().await.unwrap();

        sqlx::migrate!("./migrations")
            .run(&mut *transaction)
            .await
            .map_err(DBError::new)?;

        info!("Applied database migrations (if necessary)");

        _ = query!(
            "
                insert or ignore into Users
                    (id, nickname, money, is_system_user)
                values
                    (?, ?, ?, ?)
            ",
            DBGROUP_SNACKBAR_ID.0,
            "kasse",
            0,
            true,
        )
        .execute(&mut *transaction)
        .await
        .map_err(DBError::new)?;

        debug!("Created DBUSER_KASSE user");

        _ = query!(
            "
                insert or ignore into Users
                    (id, nickname, money, is_system_user)
                values
                    (?, ?, ?, ?)
            ",
            DBGROUP_AUFLADUNG_ID.0,
            "aufladung",
            0,
            true
        )
        .execute(&mut *transaction)
        .await
        .map_err(DBError::new)?;

        debug!("Created DBUSER_AUFLADUNG user");

        let group_k = GroupDB::_create(&mut *transaction, DBGROUP_SNACKBAR_ID.0).await?;
        let group_a = GroupDB::_create(&mut *transaction, DBGROUP_AUFLADUNG_ID.0).await?;

        match group_k
            .link_user(&mut *transaction, DBUSER_SNACKBAR_ID)
            .await
        {
            Ok(_) => {}
            Err(_) => {
                debug!("Failed to link DBUSER_KASSE with group. (Hopefully) Already linked")
            }
        };
        debug!("Linked group to user: DBUSER_KASSE");
        match group_a
            .link_user(&mut *transaction, DBUSER_AUFLADUNG_ID)
            .await
        {
            Ok(_) => {}
            Err(_) => {
                debug!("Failed to link DBUSER_AUFLADUNG with group. (Hopefully) Already linked")
            }
        };
        debug!("Linked group to user: DBUSER_AUFLADUNG");

        // no need ?

        transaction.commit().await.map_err(From::from)
    }
}
