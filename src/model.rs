use uuid::Uuid;
use serde::{Serialize, Deserialize};
use rocket::response::{Responder, Redirect};
use rocket_contrib::templates::Template;
use rocket::http::Status;

use crate::db::Databaseable;

#[derive(Debug, Serialize, Deserialize, FromForm)]
pub struct ClientCreateIsland {
    pub turnips_price: u16,
    pub fee_description: Option<String>,
    pub map_description: Option<String>,
    pub host_description: Option<String>,
    pub name: String,
    pub host_name: String,
    pub DODO: String,
    pub max_queue_size: u8,
    //rating?
    //queue?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientResponseIsland {
    pub uuid: Uuid,
    pub turnips_price: u16,
    pub fee_description: Option<String>,
    pub map_description: Option<String>,
    pub host_description: Option<String>,
    pub name: String,
    pub host_name: String,
    pub max_queue_size: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseIsland {
    pub user_email: String,
    pub DODO: String,
    pub client_response_island: ClientResponseIsland,
    //rating?
    //queue?
}

impl Databaseable for DatabaseIsland {
    fn get_table() -> &'static str {
        "island"
    }

    fn get_key(&self) -> String {
        self.client_response_island.uuid.to_hyphenated().to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
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
        self.email.clone()
    }
}

#[derive(Responder)]
pub enum FullResponse {
    Status(Status),
    Template(Template),
    Redirect(Redirect),
}
