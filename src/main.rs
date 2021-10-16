mod db;

use rocket::{get, routes};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .attach(db::stage())
        .mount("/", routes![index])
        .launch()
        .await;
}
