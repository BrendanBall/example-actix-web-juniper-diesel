//! Actix web diesel example
//!
//! Diesel does not support tokio, so we have to run it in separate threads.
//! Actix supports sync actors by default, so we going to create sync actor
//! that use diesel. Technically sync actors are worker style actors, multiple
//! of them can run in parallel and process messages from same queue.
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate r2d2;
extern crate uuid;

use actix::prelude::*;
use actix_web::{
    http,
    middleware,
    middleware::cors::Cors,
    server, App, AsyncResponder, Error, FutureResponse, HttpRequest, HttpResponse, Json, State,
};

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use futures::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;



mod db;
mod graphql;
mod models;
mod schema;

use db::{DbExecutor};
use graphql::{create_schema, Schema, Context};

struct AppState {
    executor: Addr<GraphQLExecutor>,
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLData(GraphQLRequest);

impl Message for GraphQLData {
    type Result = Result<String, Error>;
}

pub struct GraphQLExecutor {
    schema: std::sync::Arc<Schema>,
    context: Context,
}

impl GraphQLExecutor {
    fn new(schema: std::sync::Arc<Schema>, context: Context) -> GraphQLExecutor {
        GraphQLExecutor { schema, context }
    }
}

impl Actor for GraphQLExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<GraphQLData> for GraphQLExecutor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: GraphQLData, _: &mut Self::Context) -> Self::Result {
        let res = msg.0.execute(&self.schema, &self.context);
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}

fn graphiql(_req: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    let html = graphiql_source("http://127.0.0.1:8080/graphql");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn graphql(
    (st, data): (State<AppState>, Json<GraphQLData>),
) -> FutureResponse<HttpResponse> {
    st.executor
        .send(data.0)
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("juniper-example");

    let manager = ConnectionManager::<SqliteConnection>::new("test.db");
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let db_addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));

    let schema_context = Context { db: db_addr.clone() };
    let schema = std::sync::Arc::new(create_schema());
    let schema_addr = SyncArbiter::start(3, move || GraphQLExecutor::new(schema.clone(), schema_context.clone()));

    server::new(move || {
        App::with_state(AppState {
            executor: schema_addr.clone(),
        })
        .middleware(middleware::Logger::default())
        .configure(|app| {
            Cors::for_app(app)
                .allowed_origin("http://localhost:8080")
                .allowed_methods(vec!["POST"])
                // .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                // .allowed_header(header::CONTENT_TYPE)
                .supports_credentials()
                .max_age(3600)
                .resource("/graphql", |r| r.method(http::Method::POST).with(graphql))
                .resource("/graphiql", |r| r.method(http::Method::GET).h(graphiql))
                .register()

        })
    }).bind("127.0.0.1:8080")
    .unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
