use rusqlite::{Connection, MappedRows, Row};
use crate::{Code};
use anyhow::Result;

pub fn create_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "Create table if not exists codes(
                        id INTEGER PRIMARY KEY autoincrement,
                        code text not null,
                        activations integer,
                        is_valid bool,
                    )
                ",
        (),
    )?;
    Ok(())
}

pub fn get_codes(connection: &Connection) -> Result<Vec<Code>>{
    let mut codes = connection.prepare("
                    select id, code, activations, is_valid from codes
                "
    )?;
    let mut codes_iter = codes.query_map([], |row| {
        Ok(Code {
            id: row.get(0)?,
            code: row.get(1)?,
            activations: row.get(2)?,
            is_valid: row.get(3)?,
        })
    })?;

    let mut codes = Vec::new();
    while let Ok(code) = codes_iter.next().unwrap() {
        codes.push(code);
    }
    Ok(codes)
}

pub fn delete_code_by_id(connection: &Connection, id: i32) -> Result<()>{
    connection.execute("
            DELETE FROM codes WHERE id = ?
        ",
        [id]
    )?;
    Ok(())
}
pub fn create_code(connection: &Connection, code: &String, activations: u8) -> Result<()>{
    connection.execute(
        "INSERT INTO codes (code, activations) VALUES (?1, ?2)",
        (code, activations),
    )?;
    Ok(())
}