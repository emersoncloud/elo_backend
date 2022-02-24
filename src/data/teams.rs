use crate::base::db::Db;
use crate::base::game_state::Player;

use rocket::State;

pub async fn get_team_one(db: &State<Db>, label: &String) -> Vec<Player> {
    sqlx::query_as!(
        Player,
        "SELECT players.name, players.elo, players.avatar \
        FROM players \
        JOIN team_1 on team_1.player_id = players.id \
    WHERE team_1.match_label=?",
        label
    )
    .fetch_all(&**db)
    .await
    .unwrap()
}

pub async fn get_team_two(db: &State<Db>, label: &String) -> Vec<Player> {
    sqlx::query_as!(
        Player,
        "SELECT players.name, players.elo, players.avatar \
        FROM players \
        JOIN team_2 on team_2.player_id = players.id \
    WHERE team_2.match_label=?",
        label
    )
    .fetch_all(&**db)
    .await
    .unwrap()
}

pub async fn insert_team_one(db: &State<Db>, label: String, player_id: u64) {
    sqlx::query!(
        "INSERT INTO team_1 (match_label, player_id) values (?, ?)",
        label,
        player_id
    )
    .execute(&**db)
    .await
    .unwrap();
}

pub async fn insert_team_two(db: &State<Db>, label: String, player_id: u64) {
    sqlx::query!(
        "INSERT INTO team_2 (match_label, player_id) values (?, ?)",
        label,
        player_id
    )
    .execute(&**db)
    .await
    .unwrap();
}
