use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;

use rocket::http::Status;
use rocket::request::Form;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_contrib::templates::Template;
use uuid::Uuid;

use crate::authentication::{IslandHost, User};
use crate::db::{Database, Databaseable};
use crate::errors::TurnipsError;
use crate::model::{self, FullResponse};

#[derive(Debug, Serialize, Deserialize, FromForm)]
pub struct ClientCreateIsland {
    pub turnips_price: u16,
    pub fee_required: bool,
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
    pub fee_required: bool,
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
pub fn see_islands(user: Option<User>, db: State<Arc<Database>>) -> Result<Template, TurnipsError> {
    let is_logged_in = user.is_some();
    let (my_islands, islands): (Vec<DatabaseIsland>, Vec<DatabaseIsland>) =
        DatabaseIsland::get_all(&mut db.connect())?
            .into_iter()
            .partition(|i| {
                is_logged_in
                    && i.user_uuid == user.as_ref().unwrap().uuid.to_hyphenated().to_string()
            });

    Ok(Template::render(
        "see_islands",
        model::TemplateSeeIslands {
            is_logged_in,
            islands: islands
                .into_iter()
                .map(|i| i.client_response_island)
                .collect(),
            my_islands: my_islands
                .into_iter()
                .map(|i| i.client_response_island)
                .collect(),
        },
    ))
}

#[get("/see_islands/<uuid>", rank = 3)]
pub fn see_islands_uuid(
    user: Option<User>,
    uuid: String,
    db: State<Arc<Database>>,
) -> Result<FullResponse, TurnipsError> {
    let is_logged_in = user.is_some();

    let island = DatabaseIsland::get(&uuid, &mut db.connect())?;
    if let Some(i) = island {
        let island = i.client_response_island;
        Ok(FullResponse::Template(Template::render(
            "see_islands_uuid",
            model::TemplateSeeIslandsUuid {
                is_logged_in,
                island,
                name: user.map(|u| u.name),
            },
        )))
    } else {
        Ok(FullResponse::Status(Status::NotFound))
    }
}

#[get("/see_islands/<uuid>", rank = 2)]
pub fn see_islands_uuid_host(
    _host: IslandHost,
    uuid: String,
    db: State<Arc<Database>>,
) -> Result<FullResponse, TurnipsError> {
    let island = DatabaseIsland::get(&uuid, &mut db.connect())?;
    if let Some(i) = island {
        let island = i.client_response_island;
        Ok(FullResponse::Template(Template::render(
            "see_islands_uuid_host",
            model::TemplateSeeIslandsUuid {
                is_logged_in: true,
                island,
                name: None,
            },
        )))
    } else {
        Ok(FullResponse::Status(Status::NotFound))
    }
}

#[get("/create_island", rank = 2)]
pub fn get_create_island_authorized(user: User) -> Template {
    Template::render(
        "create_edit_island",
        model::TemplateIsLoggedIn {
            is_logged_in: true,
            name: Some(user.name),
        },
    )
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
) -> Result<Redirect, TurnipsError> {
    let uuid = Uuid::new_v4();
    let island = island.into_inner();
    let client_response_island = ClientResponseIsland {
        uuid,
        turnips_price: island.turnips_price,
        fee_required: island.fee_required,
        fee_description: island.fee_description,
        map_description: island.map_description,
        host_description: island.host_description,
        name: island.name,
        host_name: island.host_name,
        max_queue_size: island.max_queue_size,
        max_visitors_allowed: island.max_visitors_allowed,
    };

    let database_island = DatabaseIsland {
        user_uuid: user.uuid.to_hyphenated().to_string().clone(),
        dodo: island.dodo,
        client_response_island,
    };

    database_island.add(&mut db.connect())?;

    Ok(Redirect::to(format!("/see_islands/{}", uuid)))
}

#[post("/join_queue/<island_uuid>", data="<user_name>")]
pub fn join_queue(
    user: User,
    user_name: String,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<(), TurnipsError> {
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
        .query(&mut db.connect())?;

    let _: () = redis::Cmd::new()
        .arg("HSET")
        .arg(format!("queue_names:{}", island_uuid))
        .arg(user.get_key())
        .arg(user_name)
        .query(&mut db.connect())?;

    Ok(())
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
        .query(&mut db.connect())?;

    let _: () = redis::Cmd::new()
        .arg("HDEL")
        .arg(format!("queue_names:{}", island_uuid))
        .arg(user.get_key())
        .query(&mut db.connect())?;

    Ok(())
}

/*
// TODO remove this template
pub fn get_rank_template(
    user: User,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<Template, TurnipsError> {
    let mut context = HashMap::new();
    context.insert("rank", get_rank(user, island_uuid, db)?);

    Ok(Template::render("users_queue", context))
}
*/

//#[get("/rank/<island_uuid>")]
pub fn get_rank(
    user: User,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<Option<u32>, TurnipsError> {
    let rank: Option<u32> = redis::Cmd::new()
        .arg("ZRANK")
        .arg(format!("queue:{}", island_uuid))
        .arg(user.get_key())
        .query(&mut db.connect())?;
    Ok(rank)
}

#[get("/delete/<island_uuid>")]
pub fn delete_island(
    _island_host: IslandHost,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<(), Status> {
    let mut connection = db.connect();
    let island = DatabaseIsland::get(&island_uuid, &mut connection)?;
    if let Some(i) = island {
        i.delete(&mut connection)?;
        Ok(())
    } else {
        Err(Status::NotFound)
    }
}

#[get("/dodo/<island_uuid>")]
pub fn get_dodo(
    user: User,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<FullResponse, TurnipsError> {
    let island = DatabaseIsland::get(&island_uuid, &mut db.connect())?;
    if let Some(i) = island {
        let rank = get_rank(user, island_uuid, db)?;
        match rank {
            Some(r) if r < i.client_response_island.max_visitors_allowed as u32 => Ok(FullResponse::StringData(i.dodo)),
            _ => Ok(FullResponse::Status(Status::NotFound)),
        }
    } else {
        Ok(FullResponse::Status(Status::NotFound))
    }
}

#[get("/edit_island/<island_uuid>")]
pub fn get_edit_island(
    _island_host: IslandHost,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<FullResponse, TurnipsError> {
    let island = DatabaseIsland::get(&island_uuid, &mut db.connect())?;
    if let Some(i) = island {
        Ok(FullResponse::Template(Template::render(
            "create_edit_island",
            model::TemplateEditIsland {
                island: i,
            },
        )))
    } else {
        Ok(FullResponse::Status(Status::NotFound))
    }
}

#[post("/edit_island/<island_uuid>", data = "<island>")]
pub fn edit_island(
    user: User,
    island: Form<ClientCreateIsland>,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<Redirect, TurnipsError> {
    let island = island.into_inner();
    let client_response_island = ClientResponseIsland {
        uuid: Uuid::parse_str(&island_uuid)?,
        turnips_price: island.turnips_price,
        fee_required: island.fee_required,
        fee_description: island.fee_description,
        map_description: island.map_description,
        host_description: island.host_description,
        name: island.name,
        host_name: island.host_name,
        max_queue_size: island.max_queue_size,
        max_visitors_allowed: island.max_visitors_allowed,
    };

    let database_island = DatabaseIsland {
        user_uuid: user.uuid.to_hyphenated().to_string().clone(),
        dodo: island.dodo,
        client_response_island,
    };

    database_island.add(&mut db.connect())?;

    Ok(Redirect::to(format!("/see_islands/{}", island_uuid)))
}
