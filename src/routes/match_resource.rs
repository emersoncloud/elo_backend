use crate::base::db::Db;
use crate::base::errors::ServerError;
use crate::base::game_state::{Match, MatchEgg, MatchLabel, Player};
use crate::data::{matches, players, teams};

use rocket::fairing::AdHoc;
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::State;

use rand::rngs::StdRng;
use rand::{FromEntropy, Rng};
use sqlx::mysql::MySqlQueryResult;

const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
type EndpointResult<T> = Result<T, ServerError>;

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("match", |rocket| async {
        rocket.mount("/", routes![save_state, get_match])
    })
}

#[get("/<label>")]
pub async fn get_match(db: &State<Db>, label: String) -> EndpointResult<Json<Match>> {
    let row: Option<MatchLabel> = matches::get_match(db, &label).await;
    if row.is_none() {
        error!("Match: \"{}\" not found", label);
        return Err(ServerError::MyError(sqlx::Error::RowNotFound));
    }

    let free_players: Vec<Player> = players::get_free_players(db, &label).await;
    let team_1: Vec<Player> = teams::get_team_one(db, &label).await;
    let team_2: Vec<Player> = teams::get_team_two(db, &label).await;

    let created_match = Match {
        label,
        players: free_players,
        team_1,
        team_2,
    };

    Ok(Json::from(created_match))
}

#[post("/", format = "json", data = "<match_egg>")]
pub async fn save_state(
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

    matches::insert_match(db, label.clone()).await;

    for player in match_egg.players.iter() {
        let players_result: MySqlQueryResult = players::insert_players(db, player.clone()).await;
        players::insert_free_players(db, label.clone(), players_result.last_insert_id()).await;
    }

    for player in match_egg.team_1.iter() {
        let players_result: MySqlQueryResult = players::insert_players(db, player.clone()).await;
        teams::insert_team_one(db, label.clone(), players_result.last_insert_id()).await;
    }

    for player in match_egg.team_2.iter() {
        let players_result: MySqlQueryResult = players::insert_players(db, player.clone()).await;
        teams::insert_team_two(db, label.clone(), players_result.last_insert_id()).await;
    }

    let created_match = Match {
        label,
        players: match_egg.players.clone(),
        team_1: match_egg.team_1.clone(),
        team_2: match_egg.team_2.clone(),
    };

    Ok(Created::new("/").body(Json::from(created_match)))
}
