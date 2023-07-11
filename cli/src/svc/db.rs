//! SQlite DB

use std::{fs, path::PathBuf};

use anyhow::{Error, Ok};
use rusqlite::Connection;

use super::config::Config;

/// Database client
pub struct DbClient {
    /// Connection
    conn: Connection,
}

impl DbClient {
    /// Instantiates a new DB client
    pub fn new() -> Result<Self, Error> {
        let file = Self::db_file();
        let conn = Connection::open(file)?;
        Ok(Self { conn })
    }

    /// Returns the path to the DB file
    fn db_file() -> PathBuf {
        dirs::data_dir().unwrap().join("Newsie/data.sqlite")
    }

    /// Inititializes the DB file
    ///
    /// # Notes
    ///
    /// The file is not recreated if it exists already
    pub fn init_db_file() -> Result<PathBuf, Error> {
        let db_file = Self::db_file();
        if db_file.exists() {
            return Ok(db_file);
        }
        if let Some(folder) = db_file.parent() {
            if !folder.exists() {
                fs::create_dir(folder)?;
            }
        }
        fs::write(&db_file, vec![])?;
        Ok(db_file)
    }

    /// Destroys the DB file
    pub fn destroy_db_file() -> Result<(), Error> {
        let db_file = Self::db_file();
        if db_file.exists() {
            fs::remove_file(&db_file)?;
        }
        Ok(())
    }

    /// Checks if the db schema is initialized
    pub fn is_db_schema_init(&self) -> Result<bool, Error> {
        let query = "
            SELECT name 
            FROM sqlite_master 
            WHERE type='table' 
            AND name='config';
        ";
        Ok(self.conn.prepare(query)?.exists([])?)
    }

    /// Inititializes the SQLite schema
    pub fn init_db_schema(&self) -> Result<(), Error> {
        let query =
            "CREATE TABLE config (id INTEGER PRIMARY KEY, api_url TEXT NOT NULL, token TEXT);";
        let _n = self.conn.execute(query, ())?;
        Ok(())
    }
}

impl DbClient {
    /// Reads the configuration
    pub fn read_config(&self) -> Result<Option<Config>, Error> {
        let mut stmt = self.conn.prepare("SELECT * FROM config WHERE id=1")?;
        let mut rows = stmt.query([])?;
        let mut configs = vec![];
        while let Some(row) = rows.next()? {
            configs.push(Config {
                api_url: row.get(1)?,
                token: row.get(2)?,
            })
        }
        Ok(configs.into_iter().next())
    }

    /// Creates the config entry
    pub fn create_config(&self, config: Config) -> Result<Config, Error> {
        let _n_inserted = self.conn.execute(
            "INSERT INTO config (id, api_url, token) VALUES (1, ?1, ?2)",
            (&config.api_url, &config.token),
        )?;
        Ok(config)
    }

    /// Updates the configuration
    pub fn update_config(&self, config: Config) -> Result<Config, Error> {
        let _n_updated = self.conn.execute(
            "UPDATE config SET api_url = ?1, token = ?2 WHERE id = 1",
            (&config.api_url, &config.token),
        )?;
        Ok(config)
    }
}
