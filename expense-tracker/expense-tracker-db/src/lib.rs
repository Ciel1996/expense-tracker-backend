pub mod schema;
pub mod currencies;
pub mod expenses;
pub mod pots;
pub mod users;
pub mod splits;

pub mod setup {
    use diesel_async::pooled_connection::AsyncDieselConnectionManager;
    use diesel_async::pooled_connection::deadpool::Pool;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    /// The exact type of the DbPool in this application.
    pub type DbPool = Pool<AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>;

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

    /// Sets up the db for the application.
    pub async fn setup_db() -> DbPool {
        // TODO: load from config
        let db_string: String = String::from("postgres://admin:localpassword@localhost/postgres");

        let manager = AsyncDieselConnectionManager::new(db_string);

        let pool = Pool::builder(manager).build().unwrap();

        run_migrations(&pool).await;

        pool
    }

    /// Runs the migrations on server startup.
    async fn run_migrations(pool: &DbPool) {
        let conn = pool.get().await?;

        conn.run_pending_migrations(MIGRATIONS)?;
    }
}
