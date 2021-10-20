use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct User {
    id: Option<(u32, Uuid)>,
    registered: Option<DateTime>,
    name: String,
    email: String,
}

impl User {
    pub fn new(name: String, email: String) -> Option<Self> {
        (name.len() <= 100 && email.len() <= 100).then(|| Self {
            id: None,
            registered: None,
            name,
            email,
        })
    }

    pub fn saved(self, id: (u32, Uuid), registered: DateTime) -> Self {
        Self {
            id: Some(id),
            registered: Some(registered),
            ..self
        }
    }

    pub fn id(&self) -> u32 {
        self.id.unwrap().0
    }

    pub fn pub_id(&self) -> Uuid {
        self.id.unwrap().1
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}
