//! Actix Web Diesel integration example
//!
//! Diesel v2 is not an async library, so we have to execute queries in `web::block` closures which
//! offload blocking code (like Diesel's) to a thread-pool in order to not block the server.

#[macro_use]
extern crate diesel;

use actix_web::{error, get, middleware, post, web, App, HttpResponse, HttpServer, Responder, put};
use diesel::{prelude::*, r2d2};
use diesel::query_dsl::methods::OffsetDsl;
use uuid::Uuid;
use crate::actions::update_plant_humidity;

mod actions;
mod models;
mod schema;

/// Short-hand for the database pool type to use throughout the app.
type DbPool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

#[get("/")]
async fn get_hello_world() -> impl Responder {
    HttpResponse::Ok().body("Hello bridgefield!")
}

/// Finds plant by UID.
///
/// Extracts:
/// - the database pool handle from application data
/// - a plant UID from the request path
#[get("/plant/{plant_id}")]
async fn get_plant(
    pool: web::Data<DbPool>,
    plant_uid: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let plant_uid = plant_uid.into_inner();

    // use web::block to offload blocking Diesel queries without blocking server thread
    let plant = web::block(move || {
        // note that obtaining a connection from the pool is also potentially blocking
        let mut conn = pool.get()?;

        actions::find_plant_by_uid(&mut conn, plant_uid)
    })
    .await?
    // map diesel query errors to a 500 error response
    .map_err(error::ErrorInternalServerError)?;

    Ok(match plant {
        // plant was found; return 200 response with JSON formatted plant object
        Some(plant) => HttpResponse::Ok().json(plant),

        // plant was not found; return 404 response with error message
        None => HttpResponse::NotFound().body(format!("No plant found with UID: {plant_uid}")),
    })
}

/// Creates new plant.
///
/// Extracts:
/// - the database pool handle from application data
/// - a JSON form containing new plant info from the request body
#[post("/plant")]
async fn add_plant(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewPlant>,
) -> actix_web::Result<impl Responder> {
    // use web::block to offload blocking Diesel queries without blocking server thread
    let plant = web::block(move || {
        // note that obtaining a connection from the pool is also potentially blocking
        let mut conn = pool.get()?;

        actions::insert_new_plant(&mut conn, &form.name)
    })
    .await?
    // map diesel query errors to a 500 error response
    .map_err(error::ErrorInternalServerError)?;

    // plant was added successfully; return 201 response with new plant info
    Ok(HttpResponse::Created().json(plant))
}

#[post("/plant/{plant_id}/humidity")]
async fn set_humidity(
    pool: web::Data<DbPool>,
    plant_id: web::Path<Uuid>,
    data: web::Json<models::Humidity>,
) -> actix_web::Result<impl Responder> {
    let plant_id = plant_id.into_inner();
    println!("{}", plant_id);
    web::block(move || {
        let mut conn = pool.get()?;
        actions::update_plant_humidity(&mut conn, plant_id, data.humidity)
    })
        .await?
        .map_err(error::ErrorInternalServerError)?;


    Ok(HttpResponse::Ok())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // initialize DB pool outside of `HttpServer::new` so that it is shared across all workers
    let pool = initialize_db_pool();

    log::info!("starting HTTP server at http://localhost:8081");

    HttpServer::new(move || {
        App::new()
            // add DB pool handle to app data; enables use of `web::Data<DbPool>` extractor
            .app_data(web::Data::new(pool.clone()))
            // add request logger middleware
            .wrap(middleware::Logger::default())
            // add route handlers
            .service(get_plant)
            .service(add_plant)
            .service(get_hello_world)
            .service(set_humidity)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}

/// Initialize database connection pool based on `DATABASE_URL` environment variable.
///
/// See more: <https://docs.rs/diesel/latest/diesel/r2d2/index.html>.
fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<SqliteConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file")
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test};

    use super::*;

    #[actix_web::test]
    async fn plant_routes() {
        dotenv::dotenv().ok();
        env_logger::try_init_from_env(env_logger::Env::new().default_filter_or("info")).ok();

        let pool = initialize_db_pool();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(middleware::Logger::default())
                .service(get_plant)
                .service(add_plant),
        )
        .await;

        // send something that isn't a UUID to `get_plant`
        let req = test::TestRequest::get().uri("/plant/123").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        let body = test::read_body(res).await;
        assert!(
            body.starts_with(b"UUID parsing failed"),
            "unexpected body: {body:?}",
        );

        // try to find a non-existent plant
        let req = test::TestRequest::get()
            .uri(&format!("/plant/{}", Uuid::nil()))
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        let body = test::read_body(res).await;
        assert!(
            body.starts_with(b"No plant found"),
            "unexpected body: {body:?}",
        );

        // create new plant
        let req = test::TestRequest::post()
            .uri("/plant")
            .set_json(models::NewPlant::new("Test plant"))
            .to_request();
        let res: models::Plant = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.name, "Test plant");

        // get a plant
        let req = test::TestRequest::get()
            .uri(&format!("/plant/{}", res.id))
            .to_request();
        let res: models::Plant = test::call_and_read_body_json(&app, req).await;
        assert_eq!(res.name, "Test plant");

        // delete new plant from table
        use crate::schema::plants::dsl::*;
        diesel::delete(plants.filter(id.eq(res.id)))
            .execute(&mut pool.get().expect("couldn't get db connection from pool"))
            .expect("couldn't delete test plant from table");
    }
}
