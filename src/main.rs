#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod authentication;
mod db;
mod errors;
mod island;
mod model;

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
                island::see_island,
                authentication::login_get,
                authentication::login_submit,
                authentication::signup_get,
                authentication::signup_submit,
            ],
        )
        .attach(Template::fairing())
        .launch();
}
