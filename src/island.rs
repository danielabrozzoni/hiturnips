use rocket::response::Flash;
use rocket::response::{Responder, Redirect};
use rocket::http::Status;
use rocket::State;
use rocket::request::{Form};
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::{Database, Databaseable};
use crate::errors::TurnipsError;
use crate::model::{ClientCreateIsland, ClientResponseIsland, DatabaseIsland, FullResponse, User};

#[get("/see_islands")]
pub fn see_islands(db: State<Arc<Database>>) -> Result<Template, TurnipsError> {
    let mut context = HashMap::new();
    let mut islands = DatabaseIsland::get_all(&mut db.connect()?)?.into_iter()
        .map(|i| i.client_response_island).collect::<Vec<_>>();
    context.insert("islands", islands);
    Ok(Template::render("join", context))
}

#[get("/see_island/<uuid>")]
pub fn see_island(uuid: String, db: State<Arc<Database>>) -> Result<FullResponse, TurnipsError> {
    let mut context = HashMap::new();
    let island = DatabaseIsland::get(&uuid, &mut db.connect()?)?;
    if let Some(i) = island {
        let island = i.client_response_island;
        context.insert("island", island);
        Ok(FullResponse::Template(Template::render("see_island", context)))
    } else {
        Ok(FullResponse::Status(Status::NotFound))
    }
}

#[get("/create_island", rank = 2)]
pub fn get_create_island_authorized(_user: User) -> Template {
    let mut context = HashMap::new();
    context.insert(0, 0);
    Template::render("create_island", context)
}

#[get("/create_island", rank = 3)]
pub fn get_create_island() -> rocket::response::Flash<Redirect> {
    Flash::error(
        Redirect::to("/login"),
        "You must be logged in for creating an island",
    )
}

#[post("/create_island", data = "<island>")]
pub fn create_island(user: User, island: Form<ClientCreateIsland>,
                     db: State<Arc<Database>>) -> Result<(), TurnipsError> {
    let uuid = Uuid::new_v4();
    let island = island.into_inner();
    let client_response_island = ClientResponseIsland {
        uuid,
        turnips_price: island.turnips_price,
        fee_description: island.fee_description,
        map_description: island.map_description,
        host_description: island.host_description,
        name: island.name,
        host_name: island.host_name,
        max_queue_size: island.max_queue_size,
    };

    let database_island = DatabaseIsland {
        DODO: island.DODO,
        user_email: user.email.clone(),
        client_response_island,
    };

    database_island.add(&mut db.connect()?)?;

    Ok(())
}

// get my islands
// get island <id>
// show dodo code
// join queue
// leave queue
// remove visitor from queue
