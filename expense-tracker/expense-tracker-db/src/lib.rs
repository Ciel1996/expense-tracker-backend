pub mod schema;
pub mod currencies;
pub mod expenses;
pub mod pots;
pub mod users;
pub mod splits;

pub mod setup {
    use deadpool_diesel::postgres::{Manager, Object};
    use deadpool_diesel::Pool;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    /// The exact type of the DbPool in this application.
    pub type DbPool = deadpool_diesel::postgres::Pool;

    /// The exact type of the connection pool that is used in this application.
    pub type DbConnectionPool = Pool<Manager, Object>;

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

    /// Sets up the db for the application.
    pub async fn setup_db() -> DbConnectionPool {
        // TODO: load from config
        let db_string: String = String::from("postgres://admin:localpassword@localhost/postgres");

        let manager = Manager::new(db_string, deadpool_diesel::Runtime::Tokio1);

        let pool = Pool::builder(manager).build().unwrap();

        run_migrations(&pool).await;

        pool
    }

    /// Runs the migrations on server startup.
    async fn run_migrations(pool: &Pool<Manager, Object>) {
        let conn = pool.get().await.unwrap();
        conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
            .await
            .unwrap()
            .unwrap();
    }
}
