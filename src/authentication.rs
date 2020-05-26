use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{decode, encode, Header, Validation};
use rand::rngs::OsRng;
use rand_core::RngCore;
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, Form, FromRequest, Request};
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::db::{Database, Databaseable};
use crate::errors::TurnipsError;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub uuid: Uuid,
    pub email: String,
    pub psw_hash: String,
    pub salt: Vec<u8>,
    pub roles: Vec<String>,
}

impl Databaseable for User {
    fn get_table() -> &'static str {
        "user"
    }

    fn get_key(&self) -> String {
        self.uuid.to_hyphenated().to_string()
    }

    fn get_indexes(&self) -> Vec<(&'static str, String)> {
        let mut vec = Vec::new();
        vec.push(("email", self.email.clone()));
        vec
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        let db = request.guard::<State<Arc<Database>>>()?;
        println!("{:?}", request.get_param::<'a, String>(1));
        request
            .cookies()
            .get("jwt")
            .and_then(|cookie| {
                decode::<UserRolesToken>(
                    &cookie.value(),
                    "supersupersecret,hopingnoonewillseethis".as_ref(),
                    &Validation::default(),
                )
                .ok()
            })
            .and_then(|token| {
                let claims = token.claims;
                User::get(&claims.uuid, &mut db.connect().unwrap())
                    .ok()
                    .flatten()
            })
            .or_forward(())
    }
}

pub struct IslandHost {
    pub user: User,
    pub island_uuid: String,
}

/*
impl<'a, 'r> FromRequest<'a, 'r> for IslandHost {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<IslandHost, ()> {
        let db = request.guard::<State<Arc<Database>>>()?;
        let user = request.guard::<User>()?;
        println!("{:?}", request.get_param(2));

        request
            .cookies()
            .get("jwt")
            .and_then(|cookie| {
                decode::<UserRolesToken>(
                    &cookie.value(),
                    "supersupersecret,hopingnoonewillseethis".as_ref(),
                    &Validation::default(),
                )
                .ok()
            })
            .and_then(|token| {
                let claims = token.claims;
                User::get(&claims.uuid, &mut db.connect().unwrap())
                    .ok()
                    .flatten()
            })
            .or_forward(())
    }
}
*/

#[derive(Serialize, Deserialize)]
pub struct UserRolesToken {
    pub iat: u64,
    pub exp: u64,
    pub uuid: String,
    pub roles: Vec<String>,
}

impl UserRolesToken {
    fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

pub fn jwt_generate(user: &User) -> Result<String, TurnipsError> {
    let iat = SystemTime::now();
    let exp = iat + Duration::from_secs(60 * 60 * 24 * 7);

    let payload = UserRolesToken {
        iat: iat.duration_since(UNIX_EPOCH)?.as_secs(),
        exp: exp.duration_since(UNIX_EPOCH)?.as_secs(),
        uuid: user.uuid.to_hyphenated().to_string(),
        roles: user.roles.clone(),
    };

    Ok(encode(
        &Header::default(),
        &json!(payload),
        "supersupersecret,hopingnoonewillseethis".as_ref(),
    )?)
}

#[derive(FromForm)]
pub struct LoginForm {
    email: String,
    password: String,
}

#[get("/login", rank = 2)]
pub fn login_get_redirect(_user: User) -> Redirect {
    Redirect::to("/")
}

#[get("/login", rank = 3)]
pub fn login_get() -> Template {
    let mut context = HashMap::new();
    context.insert(0, 0);
    Template::render("login", context)
}

#[post("/login", rank = 2)]
pub fn login_submit_redirect(_user: User) -> Redirect {
    Redirect::to("/")
}

#[post("/login", data = "<login>", rank = 3)]
pub fn login_submit(
    mut cookies: Cookies,
    login: Form<LoginForm>,
    db: State<Arc<Database>>,
) -> Result<Redirect, Flash<Redirect>> {
    let mut connection = db.connect().unwrap();

    let user: Option<User> = User::get_by_index(("email", &login.email), &mut connection).unwrap();

    if let None = user {
        return Err(Flash::error(Redirect::to("/"), "Invalid username."));
    }

    let user = user.unwrap();

    // Argon2 password verifier
    if !argon2::verify_encoded(&user.psw_hash, &login.password.clone().into_bytes()).unwrap() {
        return Err(Flash::error(Redirect::to("/"), "Wrong password"));
    }

    // Add JWT to cookies
    cookies.add(Cookie::new::<String, String>(
        "jwt".into(),
        jwt_generate(&user).unwrap(),
    ));

    Ok(Redirect::to("/"))
}

#[get("/signup", rank = 2)]
pub fn signup_get_redirect(_user: User) -> Redirect {
    Redirect::to("/")
}

#[get("/signup", rank = 3)]
pub fn signup_get() -> Template {
    let mut context = HashMap::new();
    context.insert(0, 0);
    Template::render("signup", context)
}

#[post("/signup", rank = 2)]
pub fn signup_submit_redirect(_user: User) -> Redirect {
    Redirect::to("/")
}

#[post("/signup", data = "<login>", rank = 3)]
pub fn signup_submit(
    login: Form<LoginForm>,
    database: State<Arc<Database>>,
) -> Result<(), TurnipsError> {
    // TODO error if email already exists

    let password = login.password.as_bytes();
    let mut salt = [0u8; 256];
    OsRng.fill_bytes(&mut salt);
    let config = argon2::Config::default();
    let psw_hash = argon2::hash_encoded(password, &salt, &config).unwrap();
    let roles = [String::from("USER")];

    let user = User {
        uuid: Uuid::new_v4(),
        email: login.email.clone(),
        psw_hash,
        salt: salt.to_vec(),
        roles: roles.to_vec(),
    };

    user.add(&mut database.connect()?)?;

    Ok(())
}
