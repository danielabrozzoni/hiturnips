use crate::errors::TurnipsError;
use jsonwebtoken::{decode, Validation};
use redis::{Client, Connection};
use serde_json::json;

use crate::authentication::{User, UserRolesToken};

pub struct Database {
    client: Client,
}

impl Database {
    pub fn new_local() -> Result<Database, TurnipsError> {
        Database::new(String::from("redis://127.0.0.1/"))
    }

    pub fn new(url: String) -> Result<Database, TurnipsError> {
        let client = redis::Client::open(url)?;
        Ok(Database { client })
    }

    pub fn connect(&self) -> Result<Connection, TurnipsError> {
        let con = self.client.get_connection()?;
        Ok(con)
    }

    pub fn add_user(&self, user: &User) -> Result<(), TurnipsError> {
        let mut connection = self.client.get_connection()?;
        // FIXME: this sucks
        let _: () = redis::Cmd::new()
            .arg("SET")
            .arg(format!("user:{}", &user.email))
            .arg(json!(user).to_string())
            .query(&mut connection)?;
        Ok(())
    }

    pub fn get_user(&self, token: &str) -> Result<User, TurnipsError> {
        let mut connection = self.connect()?;
        let token = decode::<UserRolesToken>(
            &token,
            "supersupersecret,hopingnoonewillseethis".as_ref(),
            &Validation::default(),
        )?;
        let claims = token.claims;
        let user: String = redis::Cmd::new()
            .arg("GET")
            .arg(format!("user:{}", &claims.email))
            .query(&mut connection)?;

        serde_json::from_str(&user).map_err(|e| e.into())
    }
}
