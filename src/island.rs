use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use rocket::http::Status;
use rocket::request::Form;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_contrib::templates::Template;
use uuid::Uuid;

use crate::authentication::User;
use crate::db::{Database, Databaseable};
use crate::errors::TurnipsError;
use crate::model::FullResponse;

#[derive(Debug, Serialize, Deserialize, FromForm)]
pub struct ClientCreateIsland {
    pub turnips_price: u16,
    pub fee_description: Option<String>,
    pub map_description: Option<String>,
    pub host_description: Option<String>,
    pub name: String,
    pub host_name: String,
    pub dodo: String,
    pub max_queue_size: u8,
    pub max_visitors_allowed: u8,
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
    pub max_visitors_allowed: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseIsland {
    pub user_uuid: String,
    pub dodo: String,
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

    fn get_indexes(&self) -> Vec<(&'static str, String)> {
        Vec::new()
    }
}

#[get("/see_islands")]
pub fn see_islands(db: State<Arc<Database>>) -> Result<Template, TurnipsError> {
    let mut context = HashMap::new();
    let islands = DatabaseIsland::get_all(&mut db.connect()?)?
        .into_iter()
        .map(|i| i.client_response_island)
        .collect::<Vec<_>>();
    context.insert("islands", islands);
    Ok(Template::render("see_islands", context))
}

#[get("/see_islands/<uuid>")]
pub fn see_islands_uuid(
    uuid: String,
    db: State<Arc<Database>>,
) -> Result<FullResponse, TurnipsError> {
    let mut context = HashMap::new();
    let island = DatabaseIsland::get(&uuid, &mut db.connect()?)?;
    if let Some(i) = island {
        let island = i.client_response_island;
        context.insert("island", island);
        Ok(FullResponse::Template(Template::render(
            "see_islands_uuid",
            context,
        )))
    } else {
        Ok(FullResponse::Status(Status::NotFound))
    }
}

//pub fn see_island_host(host: IslandHost, uuid: String, db: State<Arc<Database>>) {}

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
pub fn create_island(
    user: User,
    island: Form<ClientCreateIsland>,
    db: State<Arc<Database>>,
) -> Result<(), TurnipsError> {
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
        max_visitors_allowed: island.max_visitors_allowed,
    };

    let database_island = DatabaseIsland {
        dodo: island.dodo,
        user_uuid: user.uuid.to_hyphenated().to_string().clone(),
        client_response_island,
    };

    database_island.add(&mut db.connect()?)?;

    Ok(())
}

#[get("/join_queue/<island_uuid>")]
pub fn join_queue(
    user: User,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<Redirect, TurnipsError> {
    // TODO: check that user has not joined more than *limit* islands

    let _: () = redis::Cmd::new()
        .arg("ZADD")
        .arg(format!("queue:{}", island_uuid))
        .arg("NX")
        .arg(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        )
        .arg(user.get_key())
        .query(&mut db.connect()?)?;

    Ok(Redirect::to(format!("/rank/{}", island_uuid)))
}

#[get("/leave_queue/<island_uuid>")]
pub fn leave_queue(
    user: User,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<(), TurnipsError> {
    let _: () = redis::Cmd::new()
        .arg("ZREM")
        .arg(format!("queue:{}", island_uuid))
        .arg(user.get_key())
        .query(&mut db.connect()?)?;
    Ok(())
}

#[get("/rank/<island_uuid>")]
pub fn get_rank_template(
    user: User,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<Template, TurnipsError> {
    let mut context = HashMap::new();
    context.insert("rank", get_rank(user, island_uuid, db)?);

    Ok(Template::render("users_queue", context))
}

pub fn get_rank(
    user: User,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<Option<u32>, TurnipsError> {
    let rank: Option<u32> = redis::Cmd::new()
        .arg("ZRANK")
        .arg(format!("queue:{}", island_uuid))
        .arg(user.get_key())
        .query(&mut db.connect()?)?;
    Ok(rank)
}

//pub fn get_dodo() {}
