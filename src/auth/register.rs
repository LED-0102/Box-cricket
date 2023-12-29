use bcrypt::{DEFAULT_COST, hash, verify};
use tokio_postgres::{Client, Row};
use serde_derive::Deserialize;
use actix_web::{web, HttpResponse, Responder};
use std::error::Error;
use crate::Database;

#[derive(Clone, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String
}
pub struct User {
    pub username: String,
    pub password: String,
    pub email: String
}

impl NewUser {
    pub fn new (username: String, password: String, email: String) -> NewUser {
        let hashed_password: String = hash (password.as_str(), DEFAULT_COST).unwrap();
        return NewUser {
            username,
            password: hashed_password,
            email
        }
    }
}
impl User {
    pub fn verify(&self, password: String) -> bool {
        verify(password.as_str(), &self.password).unwrap()
    }
}
impl From<Row> for User {
    fn from(value: Row) -> Self {
        User{
            username: value.get("user_name"),
            password: value.get("password"),
            email: value.get("email")
        }
    }
}
pub async fn insert( st: NewUser, client: &Client) -> Result<(), Box<dyn Error>> {
    client.execute("INSERT INTO users (user_name, password, admin) VALUES ($1, $2, false)", &[&st.username, &st.password ]).await?;
    println!("Inserted");
    Ok(())
}
pub async fn register (new_user: web::Json<NewUser>, db: web::Data<Database>) -> impl Responder {
    let new_user = NewUser::new(
        new_user.username.clone(),
        new_user.password.clone(),
        new_user.email.clone()
    );
    println!("{} {}", new_user.password, new_user.username);
    match insert(new_user, &db.client).await {
        Ok(_) => {
            println!("Created");
            HttpResponse::Created()
        },
        Err(_) => {
            println!("Conflict");
            HttpResponse::Conflict()
        }
    }
}