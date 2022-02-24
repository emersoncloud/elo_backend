use crate::base::db::Db;
use crate::base::game_state::MatchLabel;

use rocket::State;

pub async fn get_match(db: &State<Db>, label: &String) -> Option<MatchLabel> {
    sqlx::query_as!(MatchLabel, "SELECT * FROM matches WHERE label=?", label)
        .fetch_optional(&**db)
        .await
        .unwrap()
}

pub async fn insert_match(db: &State<Db>, label: String) {
    sqlx::query!("INSERT INTO matches (label) values (?)", label)
        .execute(&**db)
        .await
        .unwrap();
}
