#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;

use std::time::SystemTime;

use diesel::prelude::*;
use diesel::RunQueryDsl;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::serde::{Deserialize, json::Json};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use rocket_sync_db_pools::diesel::SqliteConnection;

use crate::models::{DstatLog, DstatServer};

mod schema;
mod models;

#[database("dstat")]
pub struct DbConn(SqliteConnection);

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ReportData<'r> {
    name: &'r str,
    rps: i32,
    secret: &'r str
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    shared_secret: String
}

#[post("/reports", data = "<data>")]
pub async fn report_api(data: Json<ReportData<'_>>, config: &State<Config>, conn: DbConn) -> Status {
    if data.secret != config.shared_secret {
        Status::Unauthorized
    } else {
        let log = DstatLog {
            time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i32,
            server_name: data.name.to_owned(),
            rps: data.rps
        };

        let _ = conn
            .run({
                use schema::logs::dsl::*;

                move |conn| diesel::insert_into(logs)
                    .values(log)
                    .execute(conn).unwrap()
            })
            .await;

        Status::Ok
    }
}

#[get("/data?<name>")]
pub async fn data_api(name: &str, conn: DbConn) -> String {
    let logs: Vec<DstatLog> = conn
        .run({
            use schema::logs::dsl::*;

            let other_name = name.to_string();

            move |conn| logs
                .filter(server_name.eq(other_name))
                .order(time.desc())
                .limit(60)
                .load::<DstatLog>(conn).unwrap()
        })
        .await;

    if logs.len() == 0 {
        0.to_string()
    } else {
        let last_log = &logs[0];
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i32;

        if (now - last_log.time) > 2 {
            0.to_string() // Dstat server has not reported data so it is probably offline
        } else {
            last_log.rps.to_string()
        }
    }
}

#[get("/?<server>")]
pub async fn dstat_page(mut server: Option<String>, conn: DbConn) -> Result<Template, Redirect> {
    let server_data = if server.is_none() {
        let servers: Vec<DstatServer> = conn
            .run({
                use schema::servers::dsl::*;

                move |conn| servers
                    .load::<DstatServer>(conn).unwrap()
            })
            .await;

        server = Some(servers[0].server_name.clone());
        servers[0].clone()
    } else {
        let data: Vec<DstatServer> = conn
            .run({
                use schema::servers::dsl::*;

                let other_server = server.as_ref().unwrap().to_string();

                move |conn| servers
                    .filter(server_name.eq(other_server))
                    .load::<DstatServer>(conn).unwrap()
            })
            .await;

        if data.len() == 0 {
            return Err(Redirect::to("/"))
        }

        data[0].clone()
    };

    Ok(Template::render("dstat", context! { id: server, url: server_data.url.to_string() }))
}

#[get("/server-status")]
pub async fn server_status(conn: DbConn) -> Template {
    let mut statuses: Vec<(String, bool)> = vec![];

    let servers: Vec<DstatServer> = conn
        .run({
            use schema::servers::dsl::*;

            move |conn| servers
                .load::<DstatServer>(conn).unwrap()
        })
        .await;

    for server in servers {
        let logs: Vec<DstatLog> = conn
            .run({
                use schema::logs::dsl::*;

                let other_name = server.server_name.to_string();

                move |conn| logs
                    .filter(server_name.eq(other_name))
                    .order(time.desc())
                    .limit(60)
                    .load::<DstatLog>(conn).unwrap()
            })
            .await;

        let last_log = &logs[0];
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i32;

        let offline = (now - last_log.time) > 2;

        statuses.push((server.server_name, !offline))
    }

    Template::render("server_status", context! { statuses })
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/api/v1", routes![report_api, data_api])
        .mount("/", routes![dstat_page])
        .mount("/fonts", FileServer::from("fonts"))
        .mount("/static", FileServer::from("static"))
        .mount("/app-assets", FileServer::from("assets"))
        .attach(Template::fairing())
        .attach(AdHoc::config::<Config>())
        .attach(DbConn::fairing())
        .ignite().await?
        .launch().await?;

    Ok(())
}