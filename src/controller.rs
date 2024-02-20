use crate::{Code, config::*};
use anyhow::{Context, Result};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};
use rusqlite::{named_params, Connection, Row};
use rusqlite_migration::{Migrations, M};
use std::convert::TryFrom;
use std::path::Path;
use std::str::FromStr;
use std::string::ToString;

const MIGRATIONS: &[&str] = &[
    "CREATE TABLE codes(id INTEGER PRIMARY KEY AUTOINCREMENT, code TEXT NOT NULL, activations INTEGER);",
];
#[derive(Debug)]
pub struct Controller {
    pub conn: Connection,
}

impl Controller {
    pub fn open(config: &Config) -> Result<Self> {
        let conn = Self::get_conn(&config.db_path).context("error connecting to database")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(Controller { conn })
    }

    fn get_conn(db_path: &Path) -> Result<Connection, rusqlite::Error> {
        Connection::open(db_path)
    }
    pub fn migrate(&mut self) -> Result<(), rusqlite_migration::Error> {
        let migrations = MIGRATIONS.iter().map(|e| M::up(e)).collect();
        Migrations::new(migrations).to_latest(&mut self.conn)
    }

    pub fn get_codes(&self) -> Result<Vec<Code>> {
        let mut codes = self.conn.prepare("
                      select * from codes;
        "
        )?;
        let mut codes_iter = codes.query_map([], |row| {
            Ok(Code {
                id: row.get(0)?,
                code: row.get(1)?,
                activations: row.get(2)?,
            })
        })?;
        let mut codes = Vec::new();
        for i in codes_iter{
            let code = i?;
            codes.push(code);
        }
        Ok(codes)
    }

    pub fn delete_code_by_id(&self, id: i32) -> Result<()> {
        self.conn.execute("
                DELETE FROM codes WHERE id = ?;
            ",
                           [id]
        )?;
        Ok(())
    }
    pub fn create_code(&self, code: &String, activations: u8) -> Result<()> {
        self.conn.execute(
            "INSERT INTO codes (code, activations) VALUES (?1, ?2);",
            (code, activations),
        )?;
        Ok(())
    }
    pub fn update_code(&self, code_id: i32) -> Result<()>{
        self.conn.execute(
            "UPDATE codes SET activations = activations - 1 where id = ?;",
            [code_id],
        )?;
        Ok(())
    }
}