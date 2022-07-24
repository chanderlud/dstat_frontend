use rocket::serde::Serialize;

use crate::schema::{logs, servers};

#[derive(Insertable, Queryable)]
#[table_name = "logs"]
pub struct DstatLog {
    pub time: i32,
    pub server_name: String,
    pub rps: i32
}

#[derive(Insertable, Queryable, Serialize, Clone)]
#[table_name = "servers"]
#[serde(crate = "rocket::serde")]
pub struct DstatServer {
    pub server_id: String,
    pub category: String,
    pub server_name: String,
    pub url: String
}