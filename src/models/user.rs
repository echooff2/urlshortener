use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, Serialize, PostgresMapper)]
#[pg_mapper(table = "user")]
pub struct User {
    pub id: i32,
    pub password: String,
}