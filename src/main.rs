#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod authentication;
mod db;
mod errors;
mod island;

use crate::authentication::*;
use crate::island::*;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use std::sync::Arc;

fn main() {
    rocket::ignite()
        .manage(Arc::new(db::Database::new_local().unwrap()))
        .mount(
            "/",
            routes![
                island::create_island,
                island::get_create_island_authorized,
                island::get_create_island,
                island::see_islands,
                authentication::login,
                authentication::sign_up,
            ],
        )
        .mount(
            "/static",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .attach(Template::fairing())
        .launch();
}
