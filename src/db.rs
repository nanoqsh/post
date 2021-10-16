use rocket::{fairing::AdHoc, Build, Rocket};
use rocket_sync_db_pools::database;

#[database("post")]
struct Db(postgres::Client);

async fn init_db(rocket: Rocket<Build>) -> Rocket<Build> {
    const CREATE: &str = r#"
    CREATE TABLE IF NOT EXISTS posts (
        id INTEGER PRIMARY KEY,
        title TEXT NOT NULL,
        text TEXT NOT NULL
    )"#;

    Db::get_one(&rocket)
        .await
        .expect("database mounted")
        .run(|conn| conn.execute(CREATE, &[]))
        .await
        .expect("DB initialized");

    rocket
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Postgres DB Stage", |rocket| async {
        rocket
            .attach(Db::fairing())
            .attach(AdHoc::on_ignite("Init DB", init_db))
    })
}
