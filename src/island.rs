use rocket::request::{Form, LenientForm};
use serde::{Serialize, Deserialize};
use rocket_contrib::json::Json;
use rocket::data::FromData;

#[derive(Debug, Deserialize, FromForm)]
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
