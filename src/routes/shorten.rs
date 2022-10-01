use actix_web::post;

use crate::{
    models::{redirect::Redirect, shorten_payload::ShortenPayload, user::User},
    LETTER_COUNT,
};
use actix_web::{error, web, Error, HttpResponse};
use deadpool_postgres::{Client, Pool};
use log::{error, info};
use tera::Tera;
use tokio_pg_mapper::FromTokioPostgresRow;

#[post("/shorten")]
pub async fn shorten(
    shorten_payload: web::Form<ShortenPayload>,
    tmpl: web::Data<Tera>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let client = db_pool.get().await.map_err(|err| {
        error!("Database Error: {err}");
        error::ErrorInternalServerError("database error")
    })?;

    if !check_password(&shorten_payload.password, &client).await? {
        info!(
            "forbidden for password: {}, url: {}",
            shorten_payload.password, shorten_payload.url
        );
        return Err(error::ErrorForbidden("403 FORBIDDEN"));
    }

    info!(
        "validated password {} for creating url: {}",
        shorten_payload.password, shorten_payload.url
    );

    let query = include_str!("../../sql/add_redirect.sql");
    let query = query.replace("$table_fields", &Redirect::sql_table_fields());
    let query = client.prepare(&query).await.unwrap();

    let result = client
        .query(&query, &[&LETTER_COUNT, &shorten_payload.url])
        .await
        .map_err(|err| {
            error!(
                "database error, code: {}, full error: {err}",
                err.code().map_or("no code", |c| c.code())
            );
            error::ErrorInternalServerError("database error")
        })?;

    let result = result.get(0);

    if let None = result.as_ref() {
        error!(
            "database insert error, database didn't return error but didn't insert value either"
        );
        return Err(error::ErrorInternalServerError("database error"));
    }

    let result = Redirect::from_row_ref(result.unwrap()).unwrap();

    let mut ctx = tera::Context::new();
    ctx.insert("url", &format!("http://localhost:3030/{}", result.name));

    let body = tmpl.render("success.html", &ctx).map_err(|err| {
        error!("template error: {:?}", err);
        error::ErrorInternalServerError("Template Error")
    })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

async fn check_password(password: &str, client: &Client) -> Result<bool, Error> {
    let query = include_str!("../../sql/check_password.sql");
    let query = query.replace("$table_fields", &User::sql_table_fields());
    let query = client.prepare(&query).await.map_err(|err| {
        error!("database error: error while preparing query, err: {err}");
        error::ErrorInternalServerError("Database Error")
    })?;

    let result = client.query_one(&query, &[&password]).await;

    match result {
        Ok(_) => Ok(true),
        Err(err) => {
            if err.as_db_error().is_some() {
                error!("Database error: error while quering one, err: {err}");
            }

            Ok(false)
        }
    }
}
