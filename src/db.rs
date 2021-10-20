use crate::prelude::*;
use rocket::{fairing::AdHoc, Build, Rocket};
use sqlx::{Error, PgPool, Row};

#[derive(Deserialize)]
struct Config {
    user: String,
    pass: String,
    host: String,
    port: u16,
    database: String,
}

struct Db {
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
}

enum GetId {
    Id(u32),
    PubId(Uuid),
}

impl GetId {
    fn get(self) -> (Option<u32>, Option<Uuid>) {
        match self {
            GetId::Id(id) => (Some(id), None),
            GetId::PubId(id) => (None, Some(id)),
        }
    }
}

struct Model<'a> {
    db: &'a Db,
}

impl<'a> Model<'a> {
    fn new(db: &'a Db) -> Self {
        Self { db }
    }

    async fn create_extension(&self) {
        sqlx::query(r#"CREATE EXTENSION IF NOT EXISTS "uuid-ossp""#)
            .execute(&self.db.pool)
            .await
            .expect("Extension initialized");
    }

    async fn create_table_users(&self) {
        const QUERY: &str = r#"
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            pub_id UUID NOT NULL DEFAULT uuid_generate_v4(),
            registered TIMESTAMP NOT NULL DEFAULT now(),
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            name TEXT NOT NULL CHECK(LENGTH(name) <= 100),
            email TEXT NOT NULL CHECK(LENGTH(name) <= 100)
        )"#;

        sqlx::query(QUERY)
            .execute(&self.db.pool)
            .await
            .expect("DB initialized");
    }

    async fn insert_user(&self, user: User) -> User {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            pub_id: Uuid,
            registered: chrono::NaiveDateTime,
        }

        const QUERY: &str = r#"
        INSERT INTO users (name, email) VALUES ($1, $2)
        RETURNING id, pub_id, registered
        "#;

        let item = user.clone();
        let Row {
            id,
            pub_id,
            registered,
        } = sqlx::query_as(QUERY)
            .bind(item.name())
            .bind(item.email())
            .fetch_one(&self.db.pool)
            .await
            .expect("Inserted");

        user.saved(
            (id as u32, pub_id),
            DateTime::from_utc(registered, chrono::Utc),
        )
    }

    async fn get_user(&self, id: GetId) -> Option<User> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            pub_id: Uuid,
            registered: chrono::NaiveDateTime,
            name: String,
            email: String,
        }

        const QUERY: &str = r#"
        SELECT id, pub_id, registered, name, email FROM users
        WHERE enabled
        AND (id=$1 OR $1 IS NULL)
        AND (pub_id=$2 OR $2 IS NULL)
        "#;

        let (id, pub_id) = id.get();
        let Row {
            id,
            pub_id,
            registered,
            name,
            email,
        } = sqlx::query_as(QUERY)
            .bind(id)
            .bind(pub_id)
            .fetch_optional(&self.db.pool)
            .await
            .expect("Selected")?;

        Some(User::new(name, email).unwrap().saved(
            (id as u32, pub_id),
            DateTime::from_utc(registered, chrono::Utc),
        ))
    }
}

async fn init_db(rocket: Rocket<Build>) -> Rocket<Build> {
    let config = rocket.figment().extract_inner("postgres").expect("Config");
    let db = Db::new(config).await.expect("Database mounted");
    let model = Model::new(&db);
    model.create_extension().await;
    model.create_table_users().await;

    let user = User::new("Ivan".into(), "ivan@mail.ru".into()).unwrap();
    let user = model.insert_user(user).await;
    dbg!(&user);

    let id = user.pub_id();
    let user = model.get_user(GetId::PubId(id)).await.unwrap();
    dbg!(&user);

    rocket.manage(db)
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Postgres DB Stage", |rocket| async {
        rocket.attach(AdHoc::on_ignite("Init DB", init_db))
    })
}
