#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

mod island;
use crate::island::{static_rocket_route_info_for_create_island};

#[get("/hello/<name>/<age>")]
fn hello(name: String, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

fn main() {
    rocket::ignite().mount("/", routes![hello, create_island]).launch();
}

