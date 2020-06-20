use rocket::http::Status;
use rocket::response::{Redirect, Responder};
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};

use crate::island;

#[derive(Responder)]
pub enum FullResponse {
    Status(Status),
    Template(Template),
    Redirect(Redirect),
    StringData(String),
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
    pub islands: Vec<island::ClientSeeIsland>,
    pub my_islands: Vec<island::ClientSeeIsland>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateSeeIslandsUuid {
    pub is_logged_in: bool,
    pub name: Option<String>,
    pub island: island::ClientSeeIsland,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateEditIsland {
    pub island: island::ClientCreateEditIsland,
    pub island_uuid: String,
}

// JSON responses

#[derive(Debug, Serialize, Deserialize)]
pub struct GetRank {
    pub rank: u8,
}
