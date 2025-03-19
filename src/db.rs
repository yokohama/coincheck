use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, Pool};
use std::env;
use dotenvy::dotenv;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager).expect("Failed to create pool.")
}

