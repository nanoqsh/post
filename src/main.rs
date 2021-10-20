mod db;
mod user;
mod prelude {
    pub use crate::user::User;
    pub use uuid::Uuid;

    pub type DateTime = chrono::DateTime<chrono::Utc>;
}

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
