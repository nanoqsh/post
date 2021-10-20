mod code;
mod model;
mod user;

mod prelude {
    pub use crate::user::User;
    pub use serde::{Deserialize, Serialize};
    pub use uuid::Uuid;

    pub type DateTime = chrono::DateTime<chrono::Utc>;
}

use crate::{
    model::{Db, Identifier, UserModel},
    prelude::*,
};
use rocket::{get, routes, State};

#[get("/find/<id>")]
async fn find(db: &State<Db>, id: &str) -> Option<String> {
    let id = code::decode(id)?;
    let model = UserModel::new(db);
    let user = model.find(Identifier::PubId(id)).await?;
    let id = user.pub_id();
    let report = format!("{}: {}", code::encode(&id), user.name());

    Some(report)
}

#[get("/insert/<name>/<email>")]
async fn insert(db: &State<Db>, name: String, email: String) -> Option<String> {
    let model = UserModel::new(db);
    let user = model.insert(User::new(name, email)?).await;
    let id = user.pub_id();
    let report = format!("{}: {}", code::encode(&id), user.name());

    Some(report)
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .attach(model::stage())
        .mount("/", routes![find, insert])
        .launch()
        .await;
}
