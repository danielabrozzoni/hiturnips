use crate::rocket::outcome::IntoOutcome;
use argon2::{self, Config};
use jsonwebtoken::{encode, Header};
use rand::rngs::OsRng;
use rand_core::RngCore;
use redis::Commands;
use rocket::http::{Cookie, Cookies};
use rocket::request::{self, Form, FromRequest, Request};
use rocket::response::Flash;
use rocket::response::Redirect;
use rocket::State;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::db::Database;
use crate::errors::TurnipsError;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub psw_hash: String,
    pub salt: Vec<u8>,
    pub roles: Vec<String>,
}

pub struct Admin {}

#[derive(Serialize, Deserialize)]
pub struct UserRolesToken {
    pub iat: u64,
    pub exp: u64,
    pub email: String,
    pub roles: Vec<String>,
}

impl UserRolesToken {
    fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        let db = request.guard::<State<Arc<Database>>>()?;
        request
            .cookies()
            .get("jwt")
            .and_then(|cookie| db.get_user(cookie.value()).ok())
            .or_forward(())
    }
}

pub fn jwt_generate(email: String, roles: Vec<String>) -> Result<String, TurnipsError> {
    let iat = SystemTime::now();
    let exp = iat + Duration::from_secs(60 * 60 * 24 * 7);

    let payload = UserRolesToken {
        iat: iat.duration_since(UNIX_EPOCH)?.as_secs(),
        exp: exp.duration_since(UNIX_EPOCH)?.as_secs(),
        email,
        roles,
    };

    encode(
        &Header::default(),
        &json!(payload),
        "supersupersecret,hopingnoonewillseethis".as_ref(),
    )
    .map_err(|e| e.into())
}

#[derive(FromForm)]
pub struct Login {
    email: String,
    password: String,
}

#[post("/login", data = "<login>")]
pub fn login(
    mut cookies: Cookies,
    login: Form<Login>,
    db: State<Arc<Database>>,
) -> Result<Redirect, Flash<Redirect>> {
    let mut connection = db.connect().unwrap();

    let user: Option<String> = connection.get(format!("user:{}", login.email)).unwrap();

    if let None = user {
        return Err(Flash::error(Redirect::to("/"), "Invalid username."));
    }

    let user: User = serde_json::from_str(&user.unwrap()).unwrap();

    let hash = user.psw_hash;

    // Argon2 password verifier
    if !argon2::verify_encoded(&hash, &login.password.clone().into_bytes()).unwrap() {
        return Err(Flash::error(Redirect::to("/"), "Wrong password"));
    }

    // Add JWT to cookies
    cookies.add(Cookie::new::<String, String>(
        "jwt".into(),
        jwt_generate(user.email, user.roles).unwrap(),
    ));

    Ok(Redirect::to("/"))
}

#[post("/signup", data = "<login>")]
pub fn sign_up(login: Form<Login>, database: State<Arc<Database>>) -> Result<(), TurnipsError> {
    // TODO error if email already exists

    let password = login.password.as_bytes();
    let mut salt = [0u8; 256];
    OsRng.fill_bytes(&mut salt);
    let config = Config::default();
    let psw_hash = argon2::hash_encoded(password, &salt, &config).unwrap();
    let roles = [String::from("USER")];

    let user = User {
        email: login.email.clone(),
        psw_hash,
        salt: salt.to_vec(),
        roles: roles.to_vec(),
    };

    database.add_user(&user)?;

    Ok(())
}
