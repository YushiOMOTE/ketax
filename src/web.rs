use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use anyhow::Result;
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler};
use log::*;

use crate::db::Db;
use crate::graphql::{schema, Context, Image, Schema};

async fn graphiql_route() -> Result<HttpResponse, Error> {
    graphiql_handler("/graphql", None).await
}

async fn playground_route() -> Result<HttpResponse, Error> {
    playground_handler("/graphql", None).await
}

async fn graphql_route(
    req: web::HttpRequest,
    payload: web::Payload,
    db: web::Data<Db>,
    schema: web::Data<Schema>,
) -> Result<HttpResponse, Error> {
    let context = Context::new(Db::clone(&db));
    graphql_handler(&schema, &context, req, payload).await
}

async fn export(db: web::Data<Db>) -> Result<String, Error> {
    let imgs: Vec<Image> = db.list().map_err(|_| HttpResponse::InternalServerError())?;
    Ok(serde_json::to_string(&imgs)?)
}

async fn import(db: web::Data<Db>, imgs: web::Json<Vec<Image>>) -> Result<String, Error> {
    for img in &*imgs {
        db.set(&img.id, &img)
            .map_err(|_| HttpResponse::InternalServerError())?;
    }
    Ok("OK".into())
}

fn json_cfg(limit_kb: usize) -> web::JsonConfig {
    web::JsonConfig::default()
        .limit(limit_kb * 1024)
        .error_handler(|e, _| {
            error!("{:?}", e);
            e.into()
        })
}

pub async fn run(addr: &str, db: Db, limit_kb: usize) -> Result<()> {
    let server = HttpServer::new(move || {
        App::new()
            .data(db.clone())
            .data(json_cfg(limit_kb))
            .data(schema())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql_route))
                    .route(web::get().to(graphql_route)),
            )
            .service(web::resource("/playground").route(web::get().to(playground_route)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql_route)))
            .service(web::resource("/export").route(web::get().to(export)))
            .service(web::resource("/import").route(web::post().to(import)))
    });
    server.bind(addr).unwrap().run().await?;
    Ok(())
}
