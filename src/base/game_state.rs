use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Match {
    pub label: String,
    pub players: Vec<Player>,
    pub team_1: Vec<Player>,
    pub team_2: Vec<Player>,
}

pub struct MatchLabel {
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MatchEgg {
    pub players: Vec<Player>,
    pub team_1: Vec<Player>,
    pub team_2: Vec<Player>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Player {
    pub name: String,
    pub elo: i32,
    pub avatar: Option<String>,
}
