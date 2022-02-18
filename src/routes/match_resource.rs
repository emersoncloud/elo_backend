use crate::base::db::Db;
use crate::base::errors::ServerError;
use crate::base::game_state::{Match, MatchEgg, MatchLabel, Player};

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
    let row: Option<MatchLabel> =
        sqlx::query_as!(MatchLabel, "SELECT * FROM matches WHERE label=?", label)
            .fetch_optional(&**db)
            .await?;

    if row.is_none() {
        error!("Match: \"{}\" not found", label);
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
