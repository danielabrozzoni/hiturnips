use rocket::http::Status;
use rocket::response::{Redirect, Responder};
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};

use crate::island::{ClientResponseIsland, DatabaseIsland};

#[derive(Responder)]
pub enum FullResponse {
    Status(Status),
    Template(Template),
    Redirect(Redirect),
    StringData(String),
    //IntegerData(u32),
}

// Template structures

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateIsLoggedIn {
    pub is_logged_in: bool,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateSeeIslands {
    pub is_logged_in: bool,
    pub islands: Vec<ClientResponseIsland>,
    pub my_islands: Vec<ClientResponseIsland>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateSeeIslandsUuid {
    pub is_logged_in: bool,
    pub name: Option<String>,
    pub island: ClientResponseIsland,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateEditIsland {
    pub island: DatabaseIsland,
}
