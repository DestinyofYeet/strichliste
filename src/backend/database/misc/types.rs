use sqlx::Sqlite;

use crate::backend::database::DBError;

pub type DatabaseResponse<T> = Result<T, DBError>;

pub type DatabaseType = Sqlite;
