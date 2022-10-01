use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, Serialize, PostgresMapper)]
#[pg_mapper(table = "redirect")]
pub struct Redirect {
    pub id: i32,
    pub name: String,
    pub url: String,
}