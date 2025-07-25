pub use article::*;
#[cfg(feature = "ssr")]
pub use group::*;
pub use money::*;
pub use transaction::*;
pub use user::*;

mod article;
#[cfg(feature = "ssr")]
mod group;
mod money;
mod transaction;
mod user;

pub type DatabaseId = i64;
