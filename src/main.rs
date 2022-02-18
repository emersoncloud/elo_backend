#[macro_use]
extern crate rocket;

mod base;
mod db_connection;
mod routes;

use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::Request;

#[options("/")]
async fn option() -> &'static str {
    "test"
}

#[get("/")]
async fn default_get() -> &'static str {
    "default get"
}

#[catch(500)]
fn internal_error(_: &Request) -> (ContentType, &'static str) {
    (ContentType::JSON, "{\"status\": \"internal error\"}")
}

#[catch(404)]
fn not_found(_: &Request) -> (ContentType, &'static str) {
    (ContentType::JSON, "{\"status\": \"not found\"}")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::try_on_ignite(
            "SQLx database",
            db_connection::init_db,
        ))
        .attach(routes::match_resource::stage())
        .mount("/", routes![default_get, option])
        .register("/", catchers![internal_error, not_found])
}
