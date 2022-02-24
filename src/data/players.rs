use crate::base::db::Db;
use crate::base::game_state::Player;

use rocket::State;
use sqlx::mysql::MySqlQueryResult;

pub async fn get_free_players(db: &State<Db>, label: &String) -> Vec<Player> {
    sqlx::query_as!(
        Player,
        "SELECT players.name, players.elo, players.avatar \
                FROM players \
                JOIN free_players on free_players.player_id = players.id \
                WHERE free_players.match_label=?",
        label
    )
    .fetch_all(&**db)
    .await
    .unwrap()
}

pub async fn insert_players(db: &State<Db>, player: Player) -> MySqlQueryResult {
    sqlx::query!(
        "INSERT INTO players (name, elo, avatar) values (?, ?, ?)",
        player.name,
        player.elo,
        player.avatar
    )
    .execute(&**db)
    .await
    .unwrap()
}

pub async fn insert_free_players(db: &State<Db>, label: String, player_id: u64) {
    sqlx::query!(
        "INSERT INTO free_players (match_label, player_id) values (?, ?)",
        label,
        player_id
    )
    .execute(&**db)
    .await
    .unwrap();
}
