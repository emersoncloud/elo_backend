use rocket::{fairing, Build, Rocket};

use dotenv::dotenv;
use std::env;

pub async fn init_db(rocket: Rocket<Build>) -> fairing::Result {
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
