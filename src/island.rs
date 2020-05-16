use rocket::response::Flash;
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::authentication::User;

#[derive(Debug, Serialize, Deserialize, FromForm)]
pub struct Island {
    turnips_price: u16,
    fee_description: Option<String>,
    map_description: Option<String>,
    host_description: Option<String>,
    name: String,
    host_name: String,
    DODO: String,
    max_queue_size: u8,
    //rating?
    //queue?
}

#[get("/see_islands")]
pub fn see_islands() -> Template {
    let mut context = HashMap::new();
    let mut islands = Vec::new();
    islands.push(Island {
        turnips_price: 100,
        fee_description: None,
        map_description: None,
        host_description: None,
        name: String::from("Island name"),
        host_name: String::from("Host name"),
        DODO: String::from("DoDo"),
        max_queue_size: 200,
    });

    islands.push(Island {
        turnips_price: 500,
        fee_description: None,
        map_description: None,
        host_description: None,
        name: String::from("Island2 name"),
        host_name: String::from("Host2 name"),
        DODO: String::from("DoDo"),
        max_queue_size: 100,
    });

    context.insert("islands", islands);
    Template::render("join", context)
}

#[get("/create_island", rank = 2)]
pub fn get_create_island_authorized(_user: User) {}

#[get("/create_island", rank = 3)]
pub fn get_create_island() -> rocket::response::Flash<Redirect> {
    Flash::error(
        Redirect::to("/login"),
        "You must be logged in for creating an island",
    )
}

#[post("/create_island", format = "json", data = "<island>")]
pub fn create_island(island: Json<Island>) {
    println!("{:?}", island);
}

// get my islands
// get all islands
// get island <id>
// show dodo code
// join queue
// leave queue
// remove visitor from queue
