use rocket::http::Status;
use rocket::response::{Redirect, Responder};
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};

use crate::island::ClientResponseIsland;

#[derive(Responder)]
pub enum FullResponse {
    Status(Status),
    Template(Template),
    Redirect(Redirect),
}

// Template structures

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateIsLoggedIn {
    pub is_logged_in: bool,
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
    pub island: ClientResponseIsland,
}
