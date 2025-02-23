pub mod schema;
pub mod currencies;
pub mod expenses;
pub mod pots;
pub mod users;
pub mod splits;

pub mod setup {
    use std::error::Error;
    use diesel::prelude::*;
    use diesel::pg::PgConnection;
    use diesel_async::AsyncPgConnection;
    use diesel_async::pooled_connection::{AsyncDieselConnectionManager};
    use diesel_async::pooled_connection::bb8::Pool;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    /// The exact type of the DbPool in this application.
    // pub type DbPool = deadpool_diesel::postgres::Pool;

    /// The exact type of the connection pool that is used in this application.
    pub type DbPool = Pool<AsyncPgConnection>;
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

    /// Sets up the db for the application.
    pub async fn setup_db() -> Result<DbPool, Box<dyn Error>> {
        // TODO: load from config
        let db_string = "postgres://admin:localpassword@localhost/postgres";

        let manager = AsyncDieselConnectionManager
            ::<AsyncPgConnection>::new(db_string);

        let pool = Pool::builder()
            .build(manager)
            .await?;

        run_migrations(db_string);

        Ok(pool)
    }

    /// Runs the migrations on server startup.
    fn run_migrations(connection_string : &str)
    {
        let mut conn = PgConnection::establish(connection_string)
            .expect("connection error");
        conn.run_pending_migrations(MIGRATIONS).expect("Failed to run migrations");
    }
}
