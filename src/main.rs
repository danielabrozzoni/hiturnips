#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod authentication;
mod db;
mod errors;
mod island;
mod model;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::sync::Arc;

#[get("/")]
fn index() -> Template {
    let mut context = HashMap::new();
    context.insert(0, 0);
    Template::render("index", context)
}

fn main() {
    rocket::ignite()
        .manage(Arc::new(db::Database::new_local().unwrap()))
        .mount(
            "/",
            routes![
                index,
                island::create_island,
                island::get_create_island_authorized,
                island::get_create_island,
                island::see_islands,
                island::see_islands_uuid,
                island::join_queue,
                island::leave_queue,
                island::get_rank_template,
                authentication::login_get,
                authentication::login_get_redirect,
                authentication::login_submit,
                authentication::login_submit_redirect,
                authentication::signup_get,
                authentication::signup_get_redirect,
                authentication::signup_submit,
                authentication::signup_submit_redirect,
                authentication::logout,
            ],
        )
        .mount("/static", StaticFiles::from("static/"))
        .attach(Template::fairing())
        .launch();
}
