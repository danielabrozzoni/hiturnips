use rocket::http::Status;
//use rocket::Request;

#[derive(Debug)]
pub enum TurnipsError {
    JSON(serde_json::error::Error),
    Redis(redis::RedisError),
    SystemTime(std::time::SystemTimeError),
    Argon2(argon2::Error),
    JWT(jsonwebtoken::errors::Error),
    UUID(uuid::Error),
}

macro_rules! impl_error {
    ( $from:ty, $to:ident ) => {
        impl std::convert::From<$from> for TurnipsError {
            fn from(err: $from) -> Self {
                TurnipsError::$to(err)
            }
        }
    };
}

impl std::convert::From<TurnipsError> for Status {
    fn from(_err: TurnipsError) -> Self {
        Status::InternalServerError
    }
}

impl_error!(serde_json::Error, JSON);
impl_error!(redis::RedisError, Redis);
impl_error!(std::time::SystemTimeError, SystemTime);
impl_error!(argon2::Error, Argon2);
impl_error!(jsonwebtoken::errors::Error, JWT);
impl_error!(uuid::Error, UUID);

/*
#[catch(404)]
fn not_found(req: &Request) -> String {
    String::from("What the fuck")
}

#[catch(500)]
fn server_error(req: &Request) -> String {
    String::from("What the fuck")
}
*/
