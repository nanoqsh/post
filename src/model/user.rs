use crate::{
    model::{Db, Identifier},
    prelude::*,
};

pub struct Model<'a>(&'a Db);

impl<'a> Model<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self(db)
    }

    pub async fn create_table(&self) {
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
            .execute(self.0.pool())
            .await
            .expect("Create");
    }

    pub async fn insert(&self, user: User) -> User {
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
            .fetch_one(self.0.pool())
            .await
            .expect("Insert");

        user.saved(
            (id as u32, pub_id),
            DateTime::from_utc(registered, chrono::Utc),
        )
    }

    pub async fn find(&self, id: Identifier) -> Option<User> {
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
            .fetch_optional(self.0.pool())
            .await
            .expect("Select")?;

        Some(User::new(name, email).unwrap().saved(
            (id as u32, pub_id),
            DateTime::from_utc(registered, chrono::Utc),
        ))
    }

    pub async fn update(&self, user: &User) {
        const QUERY: &str = r#"
        UPDATE users
        SET name=$1, email=$2
        WHERE enabled
        AND id=$3
        "#;

        sqlx::query(QUERY)
            .bind(user.name())
            .bind(user.email())
            .bind(user.id())
            .execute(self.0.pool())
            .await
            .expect("Update");
    }

    pub async fn delete(&self, id: Identifier, hard_delete: bool) {
        const QUERY_SOFT: &str = r#"
        UPDATE users
        SET enabled=FALSE
        WHERE (id=$1 OR $1 IS NULL)
        AND (pub_id=$2 OR $2 IS NULL)
        "#;

        const QUERY_HARD: &str = r#"
        DELETE FROM users
        WHERE (id=$1 OR $1 IS NULL)
        AND (pub_id=$2 OR $2 IS NULL)
        "#;

        let (id, pub_id) = id.get();
        sqlx::query(if hard_delete { QUERY_HARD } else { QUERY_SOFT })
            .bind(id)
            .bind(pub_id)
            .execute(self.0.pool())
            .await
            .expect("Delete");
    }
}
