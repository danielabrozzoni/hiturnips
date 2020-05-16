#[derive(Debug)]
pub enum TurnipsError {
    JSON(serde_json::error::Error),
    Redis(redis::RedisError),
    SystemTime(std::time::SystemTimeError),
    Argon2(argon2::Error),
    JWT(jsonwebtoken::errors::Error),
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

impl_error!(serde_json::Error, JSON);
impl_error!(redis::RedisError, Redis);
impl_error!(std::time::SystemTimeError, SystemTime);
impl_error!(argon2::Error, Argon2);
impl_error!(jsonwebtoken::errors::Error, JWT);
