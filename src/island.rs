use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;

use rocket::http::Status;
use rocket::request::Form;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_contrib::templates::Template;
use uuid::Uuid;
use serde_json::json;

use crate::authentication::{IslandHost, User};
use crate::db::{Database, Databaseable};
use crate::errors::TurnipsError;
use crate::model::{self, FullResponse};

/// Holds data which can be modified by the owner of the island.
#[derive(Debug, Serialize, Deserialize, FromForm)]
pub struct ClientCreateEditIsland {
    pub turnips_price: u16,
    pub fee_required: bool,
    pub fee_description: Option<String>,
    pub map_description: Option<String>,
    pub host_description: Option<String>,
    pub name: String,
    pub host_name: String,
    pub dodo: String,
    pub max_line_size: u8,
    pub max_visitors_allowed: u8,
}

/// Holds general info regarding the island, which can be seen by everyone.
#[derive(Debug, Serialize, Deserialize)]
// TODO change name
pub struct PublicInfoIsland {
    pub uuid: Uuid,
    pub turnips_price: u16,
    pub fee_required: bool,
    pub fee_description: Option<String>,
    pub map_description: Option<String>,
    pub host_description: Option<String>,
    pub name: String,
    pub host_name: String,
    pub max_line_size: u8,
    pub max_visitors_allowed: u8,
}

/// Holds general info regarding the island and current state of the line.
/// Used in see_islands/ and see_islands/<uuid> calls
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientSeeIsland {
    pub public_info_island: PublicInfoIsland,
    pub people_in_line: u8,
    pub eta_mins: u16,
    pub rank: Option<u8>,
}

/// Holds both general and private info regarding the island.
/// Used for storing the island in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateInfoIsland {
    pub user_uuid: String,
    pub dodo: String,
    pub public_info_island: PublicInfoIsland,
    //rating?
}

impl Databaseable for PrivateInfoIsland {
    fn get_table() -> &'static str {
        "island"
    }

    fn get_key(&self) -> String {
        self.public_info_island.uuid.to_hyphenated().to_string()
    }

    fn get_indexes(&self) -> Vec<(&'static str, String)> {
        Vec::new()
    }
}

#[get("/see_islands")]
pub fn see_islands(user: Option<User>, db: State<Arc<Database>>) -> Result<Template, TurnipsError> {
    let is_logged_in = user.is_some();
    let (my_islands, islands): (Vec<PrivateInfoIsland>, Vec<PrivateInfoIsland>) =
        PrivateInfoIsland::get_all(&mut db.connect())?
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
                .map(|i| ClientSeeIsland {
                    people_in_line: people_in_line(i.public_info_island.uuid.to_hyphenated().to_string(), &db).unwrap(),
                    eta_mins: 10,
                    rank: if user.is_some() { rank(user.as_ref().unwrap(), &i.public_info_island.uuid.to_hyphenated().to_string(), &db).unwrap() } else { None },
                    public_info_island: i.public_info_island,
                })
                .collect(),
            my_islands: my_islands
                .into_iter()
                .map(|i| ClientSeeIsland {
                    people_in_line: people_in_line(i.public_info_island.uuid.to_hyphenated().to_string(), &db).unwrap(),
                    eta_mins: 10,
                    rank: if user.is_some() { rank(user.as_ref().unwrap(), &i.public_info_island.uuid.to_hyphenated().to_string(), &db).unwrap() } else { None },
                    public_info_island: i.public_info_island,
                })
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

    let island = PrivateInfoIsland::get(&uuid, &mut db.connect())?;
    if let Some(i) = island {
        let island = ClientSeeIsland {
            people_in_line: people_in_line(i.public_info_island.uuid.to_hyphenated().to_string(), &db).unwrap(),
            eta_mins: 15,
            rank: if user.is_some() { rank(user.as_ref().unwrap(), &i.public_info_island.uuid.to_hyphenated().to_string(), &db).unwrap() } else { None },
            public_info_island: i.public_info_island,
        };
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
    let island = PrivateInfoIsland::get(&uuid, &mut db.connect())?;
    if let Some(i) = island {
        let island = ClientSeeIsland {
            people_in_line: people_in_line(i.public_info_island.uuid.to_hyphenated().to_string(), &db).unwrap(),
            eta_mins: 15,
            rank: None,
            public_info_island: i.public_info_island,
        };
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
    island: Form<ClientCreateEditIsland>,
    db: State<Arc<Database>>,
) -> Result<Redirect, TurnipsError> {
    let uuid = Uuid::new_v4();
    let island = island.into_inner();
    let public_info_island = PublicInfoIsland {
        uuid,
        turnips_price: island.turnips_price,
        fee_required: island.fee_required,
        fee_description: island.fee_description,
        map_description: island.map_description,
        host_description: island.host_description,
        name: island.name,
        host_name: island.host_name,
        max_line_size: island.max_line_size,
        max_visitors_allowed: island.max_visitors_allowed,
    };

    let database_island = PrivateInfoIsland {
        user_uuid: user.uuid.to_hyphenated().to_string(),
        dodo: island.dodo,
        public_info_island,
    };

    database_island.add(&mut db.connect())?;

    Ok(Redirect::to(format!("/see_islands/{}", uuid)))
}

#[post("/join_line/<island_uuid>", data="<user_name>")]
pub fn join_line(
    user: User,
    user_name: String,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<FullResponse, TurnipsError> {
    // TODO: check that user has not joined more than *limit* islands

    let _: () = redis::Cmd::new()
        .arg("ZADD")
        .arg(format!("line:{}", island_uuid))
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
        .arg(format!("line_names:{}", island_uuid))
        .arg(user.get_key())
        .arg(user_name)
        .query(&mut db.connect())?;

    get_rank(user, island_uuid, db)
}

#[get("/leave_line/<island_uuid>")]
pub fn leave_line(
    user: User,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<(), TurnipsError> {
    let _: () = redis::Cmd::new()
        .arg("ZREM")
        .arg(format!("line:{}", island_uuid))
        .arg(user.get_key())
        .query(&mut db.connect())?;

    let _: () = redis::Cmd::new()
        .arg("HDEL")
        .arg(format!("line_names:{}", island_uuid))
        .arg(user.get_key())
        .query(&mut db.connect())?;

    Ok(())
}

pub fn people_in_line(
    island_uuid: String,
    db: &State<Arc<Database>>,
) -> Result<u8, TurnipsError> {
    let people_in_line: u8 = redis::Cmd::new()
        .arg("ZCOUNT")
        .arg(format!("line:{}", island_uuid))
        .arg("-inf")
        .arg("+inf")
        .query(&mut db.connect())?;
    Ok(people_in_line)
}

pub fn rank(
    user: &User,
    island_uuid: &String,
    db: &State<Arc<Database>>,
) -> Result<Option<u8>, TurnipsError> {
    let rank: Option<u8> = redis::Cmd::new()
        .arg("ZRANK")
        .arg(format!("line:{}", island_uuid))
        .arg(user.get_key())
        .query(&mut db.connect())?;
    match rank {
        Some(r) => Ok(Some(r + 1)),
        None => Ok(None)
    }
}

#[get("/rank/<island_uuid>")]
pub fn get_rank(
    user: User,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<FullResponse, TurnipsError> {
    let rank = rank(&user, &island_uuid, &db)?;
    match rank {
        Some(r) => Ok(FullResponse::StringData(json!(model::GetRank{rank: r}).to_string())),
        None => Ok(FullResponse::Status(Status::NotFound)),
    }
}

#[get("/delete/<island_uuid>")]
pub fn delete_island(
    _island_host: IslandHost,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<(), Status> {
    let mut connection = db.connect();
    let island = PrivateInfoIsland::get(&island_uuid, &mut connection)?;
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
    let island = PrivateInfoIsland::get(&island_uuid, &mut db.connect())?;
    if let Some(i) = island {
        let rank = rank(&user, &island_uuid, &db)?;
        match rank {
            Some(r) if r < i.public_info_island.max_visitors_allowed => Ok(FullResponse::StringData(i.dodo)),
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
    let island = PrivateInfoIsland::get(&island_uuid, &mut db.connect())?;
    if let Some(i) = island {
        Ok(FullResponse::Template(Template::render(
            "create_edit_island",
            model::TemplateEditIsland {
                island: ClientCreateEditIsland {
                    turnips_price: i.public_info_island.turnips_price,
                    fee_required: i.public_info_island.fee_required,
                    fee_description: i.public_info_island.fee_description,
                    map_description: i.public_info_island.map_description,
                    host_description: i.public_info_island.host_description,
                    name: i.public_info_island.name,
                    host_name: i.public_info_island.host_name,
                    dodo: i.dodo,
                    max_line_size: i.public_info_island.max_line_size,
                    max_visitors_allowed: i.public_info_island.max_visitors_allowed,
                },
                island_uuid,
            },
        )))
    } else {
        Ok(FullResponse::Status(Status::NotFound))
    }
}

#[post("/edit_island/<island_uuid>", data = "<island>")]
pub fn edit_island(
    user: User,
    island: Form<ClientCreateEditIsland>,
    island_uuid: String,
    db: State<Arc<Database>>,
) -> Result<Redirect, TurnipsError> {
    let island = island.into_inner();
    let public_info_island = PublicInfoIsland {
        uuid: Uuid::parse_str(&island_uuid)?,
        turnips_price: island.turnips_price,
        fee_required: island.fee_required,
        fee_description: island.fee_description,
        map_description: island.map_description,
        host_description: island.host_description,
        name: island.name,
        host_name: island.host_name,
        max_line_size: island.max_line_size,
        max_visitors_allowed: island.max_visitors_allowed,
    };

    let private_info_island = PrivateInfoIsland {
        user_uuid: user.uuid.to_hyphenated().to_string(),
        dodo: island.dodo,
        public_info_island,
    };

    private_info_island.add(&mut db.connect())?;

    Ok(Redirect::to(format!("/see_islands/{}", island_uuid)))
}
