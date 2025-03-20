use std::env;
use dotenvy::dotenv;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, Pool};

pub fn establish_connection() -> r2d2::Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder().build(manager).expect("Failed to create pool.")
}
