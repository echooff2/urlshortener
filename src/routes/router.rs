use actix_files::NamedFile;
use actix_web::{error, get, http::StatusCode, web, Either, Error, HttpResponse, Responder};
use deadpool_postgres::Pool;
use log::{error, info};
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::models::redirect::Redirect;

#[get("/{route}")]
pub async fn router(
    route: web::Path<String>,
    db_pool: web::Data<Pool>,
) -> Result<impl Responder, Error> {
    let client = db_pool.get().await.map_err(|err| {
        error!("Database Error: {err}");
        error::ErrorInternalServerError("database error")
    })?;

    let query = include_str!("../../sql/query_redirect.sql");
    let query = query.replace("$table_fields", &Redirect::sql_table_fields());
    let query = client.prepare(&query).await.unwrap();

    let result = client.query(&query, &[&route.to_string()]).await;

    let result = result.map_err(|err| {
        error!(
            "database error, code: {}, full error: {err}",
            err.code().map_or("no code", |c| c.code())
        );
        error::ErrorInternalServerError("database error")
    })?;

    let result = result.get(0);

    if result.is_none() {
        info!("no url for name {}", route);
        return Ok(Either::Left(
            NamedFile::open_async("public/404.html")
                .await
                .unwrap()
                .customize()
                .with_status(StatusCode::NOT_FOUND),
        ));
    }

    let result = Redirect::from_row_ref(result.unwrap()).unwrap();

    Ok(Either::Right(
        HttpResponse::TemporaryRedirect()
            .append_header(("Location", result.url))
            .finish(),
    ))
}
