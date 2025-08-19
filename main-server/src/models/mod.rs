use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub mod account;
pub mod activity_log;
pub mod challenge;
pub mod global_leaderboard;
pub mod solutions;

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct InsertedId(pub i32);

pub trait GetById: Sized {
    async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Self>, sqlx::Error>;
}
