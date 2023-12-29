use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpRequest};
use actix_web::error::ErrorUnauthorized;
use futures::future::{Ready, ok, err};
use serde_derive::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use chrono::{Utc};
use tokio_postgres::{Client, Row, Statement};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwToken {
    pub username: String,
    pub exp: usize,
    pub is_admin: bool,
}

impl JwToken {
    pub fn get_key() -> String {
        let key: String = std::env::var("JWT_KEY").unwrap_or("secret".parse().unwrap());
        return key;
    }
    pub fn encode(self) -> String {
        let key = EncodingKey::from_secret(JwToken::get_key().as_ref());
        let token = encode(&Header::default(), &self, &key).unwrap();
        return token;
    }
    pub async fn new(username: String, client: &Client) -> Self {
        let s: Statement = client.prepare("SELECT admin from users where user_name=$1").await.expect("Wrong admin finding query");
        let mut admin: bool = false;
        let r: Result<Vec<Row>, tokio_postgres::error::Error> = client.query(&s, &[&username]).await;
        match r {
            Ok(ro) => {
                let b: bool = ro[0].get("admin");
                if b==true {
                    admin=true;
                } else {
                    admin=false;
                }
            },
            _ => {admin = false}
        }
        let timestamp = Utc::now().checked_add_signed(chrono::Duration::minutes(360)).expect("valid Timestamp").timestamp();
        return JwToken {username, exp: timestamp as usize, is_admin: admin};
    }
    pub fn from_token(token: String) -> Result<Self, String>{
        let key = DecodingKey::from_secret(
            JwToken::get_key().as_ref()
        );
        let token_result = decode::<JwToken>(
            &token, &key, &Validation::default());
        match token_result {
            Ok(data) => {
                Ok(data.claims)
            },
            Err(error) => {
                let message = format!("{}", error);
                return Err(message);
            }
        }
    }
}
impl FromRequest for JwToken {
    type Error = Error;
    type Future = Ready<Result<JwToken, Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        match req.headers().get("token") {
            Some(data) => {
                let raw_token = data.to_str().unwrap().to_string();
                let token_result = JwToken::from_token(raw_token);
                match token_result {
                    Ok(token) => {
                        ok(token)
                    },
                    Err(message) => {
                        if message == "ExpiredSignature".to_owned() {
                            return err(ErrorUnauthorized("Token expired"))
                        }
                        return err(ErrorUnauthorized("Token can't be decoded"))
                    }
                }
            },
            None => {
                return err (
                    ErrorUnauthorized("token not in header under key 'token'")
                );
            }
        }
    }
}