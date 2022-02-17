#[macro_use]
extern crate rocket;

use rocket::fairing::{self, AdHoc};
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Request, Rocket, State};

use rand::rngs::StdRng;
use rand::{FromEntropy, Rng};
use sqlx::mysql::MySqlQueryResult;

use dotenv::dotenv;
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use std::env;

use log::error;

const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

pub type Db = sqlx::MySqlPool;

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Match {
    label: String,
    players: Vec<Player>,
    team_1: Vec<Player>,
    team_2: Vec<Player>,
}

struct MatchLabel {
    label: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct MatchEgg {
    players: Vec<Player>,
    team_1: Vec<Player>,
    team_2: Vec<Player>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Player {
    name: String,
    elo: i32,
    avatar: Option<String>,
}

#[derive(thiserror::Error, Debug)]
enum ServerError {
    #[error("sqlx error")]
    MyError(#[from] sqlx::Error),
}

type EndpointResult<T> = Result<T, ServerError>;

impl<'r> Responder<'r, 'static> for ServerError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Self::MyError(_) => Err(Status::NotFound),
        }
    }
}

#[options("/")]
async fn yes_option() -> &'static str {
    "test"
}

#[get("/panic")]
fn get_panic() -> &'static str {
    panic!("abd news!")
}

#[get("/<label>")]
async fn get_match(db: &State<Db>, label: String) -> EndpointResult<Json<Match>> {
    let row: Option<MatchLabel> =
        sqlx::query_as!(MatchLabel, "SELECT * FROM matches WHERE label=?", label)
            .fetch_optional(&**db)
            .await?;

    if row.is_none() {
        error!("Match {} not found", label);
        return Err(ServerError::MyError(sqlx::Error::RowNotFound));
    }

    let free_players: Vec<Player> = sqlx::query_as!(
        Player,
        "SELECT players.name, players.elo, players.avatar \
                FROM players \
                JOIN free_players on free_players.player_id = players.id \
                WHERE free_players.match_label=?",
        label
    )
    .fetch_all(&**db)
    .await?;

    let team_1: Vec<Player> = sqlx::query_as!(
        Player,
        "SELECT players.name, players.elo, players.avatar \
                FROM players \
                JOIN team_1 on team_1.player_id = players.id \
                WHERE team_1.match_label=?",
        label
    )
    .fetch_all(&**db)
    .await?;

    let team_2: Vec<Player> = sqlx::query_as!(
        Player,
        "SELECT players.name, players.elo, players.avatar \
                FROM players \
                JOIN team_2 on team_2.player_id = players.id \
                WHERE team_2.match_label=?",
        label
    )
    .fetch_all(&**db)
    .await?;

    let created_match = Match {
        label,
        players: free_players,
        team_1,
        team_2,
    };

    Ok(Json::from(created_match))
}

#[post("/", format = "json", data = "<match_egg>")]
async fn save_state(
    db: &State<Db>,
    match_egg: Json<MatchEgg>,
) -> EndpointResult<Created<Json<Match>>> {
    let mut std_rng = StdRng::from_entropy();

    let label: String = (0..4)
        .map(|_| {
            let idx = std_rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    sqlx::query!("INSERT INTO matches (label) values (?)", label)
        .execute(&**db)
        .await?;

    for player in match_egg.players.iter() {
        let players_result: MySqlQueryResult = sqlx::query!(
            "INSERT INTO players (name, elo, avatar) values (?, ?, ?)",
            player.name,
            player.elo,
            player.avatar
        )
        .execute(&**db)
        .await?;

        sqlx::query!(
            "INSERT INTO free_players (match_label, player_id) values (?, ?)",
            label,
            players_result.last_insert_id()
        )
        .execute(&**db)
        .await?;
    }
    for player in match_egg.team_1.iter() {
        let players_result: MySqlQueryResult = sqlx::query!(
            "INSERT INTO players (name, elo, avatar) values (?, ?, ?)",
            player.name,
            player.elo,
            player.avatar
        )
        .execute(&**db)
        .await?;

        sqlx::query!(
            "INSERT INTO team_1 (match_label, player_id) values (?, ?)",
            label,
            players_result.last_insert_id()
        )
        .execute(&**db)
        .await?;
    }

    for player in match_egg.team_2.iter() {
        let players_result: MySqlQueryResult = sqlx::query!(
            "INSERT INTO  players (name, elo, avatar) values (?, ?, ?)",
            player.name,
            player.elo,
            player.avatar
        )
        .execute(&**db)
        .await?;

        sqlx::query!(
            "INSERT INTO team_2 (match_label, player_id) values (?, ?)",
            label,
            players_result.last_insert_id()
        )
        .execute(&**db)
        .await?;
    }

    let created_match = Match {
        label,
        players: match_egg.players.clone(),
        team_1: match_egg.team_1.clone(),
        team_2: match_egg.team_2.clone(),
    };

    Ok(Created::new("/").body(Json::from(created_match)))
}

#[get("/")]
async fn default_get() -> &'static str {
    "default get"
}

#[catch(500)]
fn internal_error(_: &Request) -> (ContentType, &'static str) {
    (ContentType::JSON, "{\"status\": \"internal error\"}")
}

#[catch(404)]
fn not_found(_: &Request) -> (ContentType, &'static str) {
    (ContentType::JSON, "{\"status\": \"not found\"}")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::try_on_ignite("SQLx database", init_db))
        .mount(
            "/",
            routes![save_state, get_match, yes_option, default_get, get_panic],
        )
        .register("/", catchers![internal_error, not_found])
}

async fn init_db(rocket: Rocket<Build>) -> fairing::Result {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = match sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(5)
        .connect(database_url.as_str())
        .await
    {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to connect to SQLx database: {:?}", e);
            return Err(rocket);
        }
    };

    if let Err(e) = sqlx::migrate!().run(&db).await {
        error!("Failed to init db: {}", e);
        return Err(rocket);
    };

    Ok(rocket.manage(db))
}
