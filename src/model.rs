use rocket::http::Status;
use rocket::response::{Redirect, Responder};
use rocket_contrib::templates::Template;

#[derive(Responder)]
pub enum FullResponse {
    Status(Status),
    Template(Template),
    Redirect(Redirect),
}
