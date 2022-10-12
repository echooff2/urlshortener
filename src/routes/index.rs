use actix_files::NamedFile;
use actix_web::{error as web_error, get};

#[get("/")]
pub async fn index() -> web_error::Result<NamedFile> {
    Ok(NamedFile::open_async("public/index.html")
        .await
        .map_err(|err| {
            log::error!("error while getting file at path 'public/index.html', err: {err}");
            web_error::ErrorInternalServerError("Internal Server Error!")
        })?)
}
