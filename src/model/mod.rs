mod db;
mod user;

pub use db::{stage, Db};
pub use user::Model as UserModel;

async fn init_tables(db: &Db) {
    UserModel::new(db).create_table().await
}

pub enum Identifier {
    Id(u32),
    PubId(uuid::Uuid),
}

impl Identifier {
    fn get(self) -> (Option<u32>, Option<uuid::Uuid>) {
        match self {
            Self::Id(id) => (Some(id), None),
            Self::PubId(id) => (None, Some(id)),
        }
    }
}
