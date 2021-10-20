use crate::prelude::*;
use rocket::{fairing::AdHoc, Build, Rocket};
use sqlx::{Error, PgPool};

#[derive(Deserialize)]
struct Config {
    user: String,
    pass: String,
    host: String,
    port: u16,
    database: String,
}

pub struct Db {
    pool: PgPool,
}

impl Db {
    async fn new(config: Config) -> Result<Self, Error> {
        let Config {
            user,
            pass,
            host,
            port,
            database,
        } = config;

        let url = format!(
            "postgresql://{}:{}@{}:{}/{}",
            user, pass, host, port, database
        );

        Ok(Self {
            pool: PgPool::connect(&url).await?,
        })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn init(&self) {
        sqlx::query(r#"CREATE EXTENSION IF NOT EXISTS "uuid-ossp""#)
            .execute(self.pool())
            .await
            .expect("Extension initialized");
    }
}

async fn init_db(rocket: Rocket<Build>) -> Rocket<Build> {
    let config = rocket.figment().extract_inner("postgres").expect("Config");
    let db = Db::new(config).await.expect("Database mounted");
    db.init().await;

    super::init_tables(&db).await;

    rocket.manage(db)
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Postgres DB Stage", |rocket| async {
        rocket.attach(AdHoc::on_ignite("Init DB", init_db))
    })
}
