use actix_files::NamedFile;
use actix_web::{
    http::{Method, StatusCode},
    Either, HttpResponse, Responder,
};

pub async fn default_handler(req_method: Method) -> impl Responder {
    match req_method {
        Method::GET => Either::Left(
            NamedFile::open_async("public/404.html")
                .await
                .unwrap()
                .customize()
                .with_status(StatusCode::NOT_FOUND),
        ),
        _ => {
            println!("method not allowed");
            Either::Right(HttpResponse::MethodNotAllowed())
        }
    }
}
