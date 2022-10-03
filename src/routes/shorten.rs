use actix_web::{post, HttpRequest};

use crate::{
    models::{redirect::Redirect, shorten_payload::ShortenPayload, user::User},
    LETTER_COUNT,
};
use actix_web::{error, web, Error, HttpResponse};
use deadpool_postgres::{Client, Pool};
use log::{error, info, trace};
use tera::Tera;
use tokio_pg_mapper::FromTokioPostgresRow;

#[post("/shorten")]
pub async fn shorten(
    request: HttpRequest,
    shorten_payload: web::Form<ShortenPayload>,
    tmpl: web::Data<Tera>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let client = db_pool.get().await.map_err(|err| {
        error!("Database Error: {err}");
        error::ErrorInternalServerError("database error")
    })?;

    let mut url = shorten_payload
        .url
        .parse::<url::Url>()
        .or_else(|_| format!("http://{}", shorten_payload.url).parse::<url::Url>())
        .map_err(|_err| {
            trace!("{}, is not a valid URL", shorten_payload.url);
            error::ErrorBadRequest("url malformated")
        })?;

    if url.cannot_be_a_base() {
        trace!("{} cannot be a base", url);
        return Err(error::ErrorBadGateway("url malformated"));
    }

    if url.scheme().is_empty() {
        url.set_scheme("http").map_err(|_err| {
            trace!("setting schema \"http\" unsuccessful for url: {}", url);
            error::ErrorBadRequest("url malformated")
        })?;
    }

    if !check_password(&shorten_payload.password, &client).await? {
        info!(
            "forbidden for password: {}, url: {}",
            shorten_payload.password, url.to_string()
        );
        return Err(error::ErrorForbidden("403 FORBIDDEN"));
    }

    info!(
        "validated password {} for creating url: {}",
        shorten_payload.password, url.to_string()
    );

    let query = include_str!("../../sql/add_redirect.sql");
    let query = query.replace("$table_fields", &Redirect::sql_table_fields());
    let query = client.prepare(&query).await.unwrap();

    let result = client
        .query(&query, &[&LETTER_COUNT, &url.to_string()])
        .await
        .map_err(|err| {
            error!(
                "database error, code: {}, full error: {err}",
                err.code().map_or("no code", |c| c.code())
            );
            error::ErrorInternalServerError("database error")
        })?;

    let result = result.get(0);

    if result.is_none() {
        error!(
            "database insert error, database didn't return error but didn't insert value either"
        );
        return Err(error::ErrorInternalServerError("database error"));
    }

    let result = Redirect::from_row_ref(result.unwrap()).unwrap();

    let conn_info = request.connection_info();
    let mut ctx = tera::Context::new();
    ctx.insert(
        "url",
        &format!(
            "{}://{}/{}",
            conn_info.scheme(),
            conn_info.host(),
            result.name
        ),
    );

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
