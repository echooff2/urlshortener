use actix_web::{get, Responder};
use actix_files::NamedFile;

#[get("/")]
pub async fn index() -> impl Responder {
    NamedFile::open_async("public/index.html").await.unwrap()
}