#[macro_use]
extern crate rocket;

use rocket::fairing::{self, AdHoc, Fairing, Info, Kind};
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket, State, Request, Response};
use rocket::http::Header;

use rand::rngs::StdRng;
use rand::{FromEntropy, Rng};
use sqlx::mysql::MySqlQueryResult;

use dotenv::dotenv;
use std::env;

const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Match {
    label: String,
    players: Vec<Player>,
    team_1: Vec<Player>,
    team_2: Vec<Player>,
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

pub type Db = sqlx::MySqlPool;

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[options("/")]
async fn yes_option() -> &'static str {
    "test"
}

#[post("/", format = "json", data = "<match_egg>")]
async fn save_state(db: &State<Db>, match_egg: Json<MatchEgg>) -> Result<Created<Json<Match>>> {
    let mut std_rng = StdRng::from_entropy();

    let label: String = (0..4)
        .map(|_| {
            let idx = std_rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    let match_result: MySqlQueryResult =
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

        let free_players_result: MySqlQueryResult = sqlx::query!(
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

        let team_1_result: MySqlQueryResult = sqlx::query!(
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

        let team_2_result: MySqlQueryResult = sqlx::query!(
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

#[get("/<label>")]
async fn get_match(db: &State<Db>, label: String) -> Result<Json<Match>> {
    let row = sqlx::query!("SELECT * FROM matches WHERE label=?", label)
        .fetch_one(&**db)
        .await?;

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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(AdHoc::try_on_ignite("SQLx database", init_db))
        .attach(CORS)
        .mount("/", routes![save_state, get_match, yes_option])
}

async fn init_db(rocket: Rocket<Build>) -> fairing::Result {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

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
