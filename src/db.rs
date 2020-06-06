use std::collections::HashMap;

use redis::{Client, Connection};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;

use crate::errors::TurnipsError;

pub struct Database {
    client: Client,
}

pub trait Databaseable: Serialize + DeserializeOwned {
    fn get_key(&self) -> String;

    fn get_table() -> &'static str;

    fn get_indexes(&self) -> Vec<(&'static str, String)>;

    fn add(&self, connection: &mut Connection) -> Result<(), TurnipsError> {
        let _: () = redis::Cmd::new()
            .arg("HSET")
            .arg(Self::get_table())
            .arg(self.get_key())
            .arg(json!(&self).to_string())
            .query(connection)?;

        for (key, value) in self.get_indexes() {
            let _: () = redis::Cmd::new()
                .arg("HSET")
                .arg(format!("{}:{}", Self::get_table(), key))
                .arg(value)
                .arg(self.get_key())
                .query(connection)?;
        }

        Ok(())
    }

    fn get(key: &str, connection: &mut Connection) -> Result<Option<Self>, TurnipsError> {
        let obj: Option<String> = redis::Cmd::new()
            .arg("HGET")
            .arg(Self::get_table())
            .arg(key)
            .query(connection)?;

        match obj {
            Some(i) => Ok(serde_json::from_str(&i)?),
            None => Ok(None),
        }
    }

    fn get_all(connection: &mut Connection) -> Result<Vec<Self>, TurnipsError> {
        Ok(redis::Cmd::new()
            .arg("HGETALL")
            .arg(Self::get_table())
            .query::<HashMap<String, String>>(connection)?
            .into_iter()
            .map(|(_, v)| serde_json::from_str(&v))
            .collect::<Result<_, _>>()?)
    }

    fn get_by_index(
        index: (&'static str, &str),
        connection: &mut Connection,
    ) -> Result<Option<Self>, TurnipsError> {
        let index: Option<String> = redis::Cmd::new()
            .arg("HGET")
            .arg(format!("{}:{}", Self::get_table(), index.0))
            .arg(index.1)
            .query(connection)?;

        match index {
            Some(i) => Ok(Self::get(&i, connection)?),
            None => Ok(None),
        }
    }

    fn delete(self, connection: &mut Connection) -> Result<(), TurnipsError> {
        let _: () = redis::Cmd::new()
            .arg("HDEL")
            .arg(Self::get_table())
            .arg(self.get_key())
            .query(connection)?;
        Ok(())
    }
}

impl Database {
    pub fn new_local() -> Result<Database, TurnipsError> {
        Database::new(String::from("redis://127.0.0.1/"))
    }

    pub fn new(url: String) -> Result<Database, TurnipsError> {
        let client = redis::Client::open(url)?;
        Ok(Database { client })
    }

    pub fn connect(&self) -> Connection {
        self.client.get_connection().expect("Db connection failure")
    }
}
