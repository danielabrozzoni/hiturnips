#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod authentication;
mod db;
mod errors;
mod island;
mod model;

use crate::authentication::User;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use std::sync::Arc;

#[get("/")]
fn index(user: Option<User>) -> Template {
    Template::render(
        "index",
        model::TemplateIsLoggedIn {
            is_logged_in: user.is_some(),
            name: user.map(|u| u.name),
        },
    )
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
                island::see_islands_uuid_host,
                island::join_queue,
                island::leave_queue,
                //island::get_rank,
                island::delete_island,
                island::get_edit_island,
                island::edit_island,
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
