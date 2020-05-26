use rocket::http::Status;
use rocket::response::{Redirect, Responder};
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::Databaseable;

#[derive(Responder)]
pub enum FullResponse {
    Status(Status),
    Template(Template),
    Redirect(Redirect),
}
