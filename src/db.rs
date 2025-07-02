use include_dir::{Dir, include_dir};
use rusqlite::{Connection, Transaction};
use rusqlite::ffi::sqlite3_auto_extension;
use rusqlite_migration::Migrations;
use sqlite_vec::sqlite3_vec_init;
use std::sync::LazyLock;
use zerocopy::IntoBytes;

use crate::model::Zettel;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

static MIGRATIONS: LazyLock<Migrations<'static>> =
    LazyLock::new(|| Migrations::from_directory(&MIGRATIONS_DIR).unwrap());

pub async fn migrate_to_latest(db_url: &str) -> Result<(), rusqlite::Error> {
    let mut conn = get_db(db_url).await?;
    MIGRATIONS.to_latest(&mut conn).unwrap();
    Ok(())
}

pub async fn get_db(db_url: &str) -> Result<Connection, rusqlite::Error> {
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }
    let conn = Connection::open(db_url)?;
    Ok(conn)
}

pub async fn store_zettel(tx: &Transaction<'_>, content: &str, embedding: Vec<f32>, parent_ids: Vec<i64>) -> Result<(), rusqlite::Error> {
    let zettel: Zettel = tx
        .prepare("insert into zettel (content) values (?) returning id, content, created_at")?
        .query_one((content,), |row| {
            Ok(Zettel {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;

    tx.prepare("insert into zettel_embedding (zettel_id, embedding) values (?, ?)")?
        .execute(rusqlite::params![zettel.id, embedding.as_bytes()])?;

    tx.prepare("insert into edge (node_id) values (?)")?
        .execute(rusqlite::params![zettel.id])?;

    let mut insert_edge_stmt = tx.prepare("insert into edge (node_id, parent_id) values (?, ?)")?;
    for id in parent_ids {
        insert_edge_stmt.execute(rusqlite::params![zettel.id, id])?;
    }

    Ok(())
}


pub async fn find_zettel_by_id(tx: &Transaction<'_>, id: i64) -> Result<Zettel, rusqlite::Error> {
    let mut stmt = tx.prepare(
        "
        select id, content, created_at
        from zettel
        where id = ?
        "
        )?;

    let zettel = stmt.query_one([id], |row| {
        Ok(Zettel {
            id: row.get(0)?,
            content: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;

    Ok(zettel)
}


pub async fn find_zettels_by_embedding(tx: &Transaction<'_>, embedding: Vec<f32>) -> Result<Vec<Zettel>, rusqlite::Error> {
    let mut stmt = tx.prepare(
        "
        select id, content, created_at
        from zettel z
        join zettel_embedding ze on z.id = ze.zettel_id
        where ze.embedding match ?
        and k = 3
        "
        )?;

    let thoughts: Vec<Zettel> = stmt.query_map(
        [embedding.as_bytes()],
        |row| {
            Ok(Zettel {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
            })
        },
    )?
        .collect::<Result<Vec<Zettel>, rusqlite::Error>>()?;

    Ok(thoughts)
}

