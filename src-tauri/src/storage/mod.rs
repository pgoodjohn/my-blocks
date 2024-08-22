use log;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub fn setup_database(
    configuration: &super::configuration::Configuration,
) -> Result<Pool<SqliteConnectionManager>, String> {
    log::debug!("Initializing db {:?}", &configuration.db_path);
    let manager = SqliteConnectionManager::file(std::path::PathBuf::from(&configuration.db_path));
    log::debug!("DB Was initialized");

    match r2d2::Pool::new(manager) {
        Ok(pool) => {
            setup_structure(&pool, configuration).unwrap();
            log::debug!("Pool Was initialized");
            Ok(pool)
        }
        Err(e) => {
            log::error!("Could not initialize db: {:?}", e);
            Err(String::from("Could not initialize database"))
        }
    }
}

// TODO: Manage db versions
pub fn setup_structure(
    pool: &Pool<SqliteConnectionManager>,
    configuration: &super::configuration::Configuration,
) -> rusqlite::Result<()> {
    if configuration.development_mode {
        log::debug!("Run with --run-migrations to run migrations");
        // @TODO: Set up with --run-migrations flag and uncomment this return.
        // return Ok(());
    }

    log::info!("Running Migrations");

    pool.get()
        .unwrap()
        .execute(
            "
            CREATE TABLE IF NOT EXISTS blocks (
                id TEXT PRIMARY KEY,
                parent_id TEXT NOT NULL,
                block_type VARCHAR(255) NOT NULL,
                data TEXT NOT NULL,
                block_order INTEGER NOT NULL DEFAULT 0,
                favorite BOOLEAN NOT NULL DEFAULT 0,
                created_at_utc DATETIME NOT NULL,
                updated_at_utc DATETIME NOT NULL
            );",
            [],
        )
        .unwrap();

    // TODO: Add indexes
    // CREATE INDEX idx_parent_id ON blocks(parent_id);
    // CREATE INDEX idx_type ON blocks(type);
    // CREATE INDEX idx_created_at_utc ON blocks(created_at_utc);
    // CREATE INDEX idx_updated_at_utc ON blocks(updated_at_utc);

    Ok(())
}
