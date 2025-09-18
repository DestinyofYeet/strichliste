pub mod behaviour;
pub mod misc;
pub mod model;

pub use model::*;

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
