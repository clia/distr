use std::{env, io};

use anyhow::Result;
use bytes::Bytes;
use ntex::channel::mpsc;
use ntex::http::{header, Method, StatusCode};
use ntex::web::{self, error, guard, middleware, App, Error, HttpRequest, HttpResponse};
use ntex_files as fs;
use ntex_session::{CookieSession, Session};

mod config;

/// favicon handler
#[web::get("/favicon")]
async fn favicon() -> Result<fs::NamedFile, Error> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

/// simple index handler
#[web::get("/welcome")]
async fn welcome(session: Session, req: HttpRequest) -> Result<HttpResponse, Error> {
    println!("{:?}", req);

    // session
    let mut counter = 1;
    if let Some(count) = session.get::<i32>("counter")? {
        println!("SESSION value: {}", count);
        counter = count + 1;
    }

    // set counter to session
    session.set("counter", counter)?;

    // response
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/welcome.html")))
}

/// 404 handler
async fn p404() -> Result<fs::NamedFile, Error> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

/// response body
async fn response_body(path: web::types::Path<String>) -> HttpResponse {
    let text = format!("Hello {}!", *path);

    let (tx, rx_body) = mpsc::channel();
    let _ = tx.send(Ok::<_, Error>(Bytes::from(text)));

    HttpResponse::Ok().streaming(rx_body)
}

/// handler with path parameters like `/user/{name}/`
async fn with_param(req: HttpRequest, path: web::types::Path<(String,)>) -> HttpResponse {
    println!("{:?}", req);

    HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("Hello {}!", path.0))
}

#[ntex::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "ntex=info");
    env_logger::init();

    let server_config = config::ServerConfig::load_config()?;
    println!("{:?}", server_config);

    web::server(|| {
        App::new()
            // cookie session middleware
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            .service(
                // static files
                fs::Files::new("/", "./static/").index_file("welcome.html"),
            )

            // .service((
            //     // register favicon
            //     favicon,
            //     // register simple route, handle all methods
            //     welcome,
            //     // with path parameters
            //     web::resource("/user/{name}").route(web::get().to(with_param)),
            //     // async response body
            //     web::resource("/async-body/{name}").route(web::get().to(response_body)),
            //     web::resource("/test").to(|req: HttpRequest| async move {
            //         match *req.method() {
            //             Method::GET => HttpResponse::Ok(),
            //             Method::POST => HttpResponse::MethodNotAllowed(),
            //             _ => HttpResponse::NotFound(),
            //         }
            //     }),
            //     web::resource("/error").to(|| async {
            //         error::InternalError::new(
            //             io::Error::new(io::ErrorKind::Other, "test"),
            //             StatusCode::INTERNAL_SERVER_ERROR,
            //         )
            //     }),
            //     // static files
            //     fs::Files::new("/static", "static").show_files_listing(),
            //     // redirect
            //     web::resource("/").route(web::get().to(|req: HttpRequest| async move {
            //         println!("{:?}", req);
            //         HttpResponse::Found()
            //             .header(header::LOCATION, "static/welcome.html")
            //             .finish()
            //     })),
            // ))
            // // default
            // .default_service(
            //     // 404 for GET request
            //     web::resource("")
            //         .route(web::get().to(p404))
            //         // all requests that are not `GET`
            //         .route(
            //             web::route()
            //                 .guard(guard::Not(guard::Get()))
            //                 .to(|| async { HttpResponse::MethodNotAllowed() }),
            //         ),
            // )
    })
    .bind(format!("0.0.0.0:{}", server_config.http.outer_port))?
    .bind(format!("0.0.0.0:{}", server_config.http.outer_ssl_port))?
    .run()
    .await?;

    Ok(())
}
