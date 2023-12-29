use serde_derive::{Deserialize, Serialize};
use bcrypt::{ verify};
use serde_json;
use crate::auth::register::User;
#[derive(Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String
}
#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String
}

use actix_web::{web, HttpResponse};
use crate::auth::jwt::JwToken;
use crate::Database;

pub async fn login (credentials: web::Json<Login>, db: web::Data<Database>) -> HttpResponse {
    let client = &db.client;
    let password = &credentials.password;
    let mut rows = client.query("SELECT * FROM users WHERE user_name = $1", &[&credentials.username]).await.expect("Can't execute this query");
    let mut rows: Vec<User> = rows.into_iter().map(User::from).collect();
    if rows.len() == 0 {
        return HttpResponse::NotFound().await.unwrap();
    } else if rows.len()>1 {
        return HttpResponse::Conflict().await.unwrap();
    }
    let rows = rows.pop().unwrap();
    println!("Useer found");
    match verify(password, &rows.password).unwrap() {
        true =>{
            let token = JwToken::new(rows.username, client);
            let raw_token = token.await.encode();
            let response = LoginResponse{token: raw_token.clone()};
            let body = serde_json::to_string(&response).unwrap();
            HttpResponse::Ok().append_header(("token", raw_token)).json(&body)
        },
        false => {
            HttpResponse::Unauthorized().into()
        }
    }
}