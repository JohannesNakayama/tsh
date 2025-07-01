use include_dir::{Dir, include_dir};
use rusqlite::{Connection, Transaction};
use rusqlite::ffi::sqlite3_auto_extension;
use rusqlite_migration::Migrations;
use sqlite_vec::sqlite3_vec_init;
use std::sync::LazyLock;
use zerocopy::IntoBytes;

use crate::model::Thought;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

static MIGRATIONS: LazyLock<Migrations<'static>> =
    LazyLock::new(|| Migrations::from_directory(&MIGRATIONS_DIR).unwrap());

pub async fn migrate_to_latest(db_url: &str) -> Result<(), rusqlite::Error> {
    let mut conn = Connection::open(db_url)?;
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

pub async fn store_atomic_thought(tx: &Transaction<'_>, content: &str, embedding: Vec<f32>) -> Result<Thought, rusqlite::Error> {
    let thought: Thought = tx
        .prepare("insert into thought (content) values (?) returning id, content")?
        .query_one((content,), |row| {
            Ok(Thought {
                id: row.get(0)?,
                content: row.get(1)?,
            })
        })?;

    tx.prepare("insert into thought_embedding (thought_id, embedding) values (?, ?)")?
        .execute(rusqlite::params![thought.id, embedding.as_bytes()])?;

    tx.prepare("insert into edge (node_id) values (?)")?
        .execute((thought.id,))?;

    Ok(thought)
}

pub async fn store_combined_thought(tx: &Transaction<'_>, content: &str, embedding: Vec<f32>, parent_ids: Vec<i64>) -> Result<(), rusqlite::Error> {
    // let db = get_db("my_thoughts.db").await?;

    let thought: Thought = tx
        .prepare("insert into thought (content) values (?) returning id, content")?
        .query_one((content,), |row| {
            Ok(Thought {
                id: row.get(0)?,
                content: row.get(1)?,
            })
        })?;

    tx.prepare("insert into thought_embedding (thought_id, embedding) values (?, ?)")?
        .execute(rusqlite::params![thought.id, embedding.as_bytes()])?;

    let mut insert_edge_stmt = tx.prepare("insert into edge (node_id, parent_id) values (?, ?)")?;

    for id in parent_ids {
        insert_edge_stmt.execute(rusqlite::params![thought.id, id])?;
    }

    Ok(())
}


pub async fn find_thoughts_by_embedding(tx: &Transaction<'_>, embedding: Vec<f32>) -> Result<Vec<Thought>, rusqlite::Error> {
    let mut stmt = tx.prepare(
        "
        select id, content
        from thought t
        join thought_embedding te on t.id = te.thought_id
        where te.embedding match ?
        and k = 3
        "
        )?;

    let thoughts: Vec<Thought> = stmt.query_map(
        [embedding.as_bytes()],
        |row| {
            Ok(Thought {
                id: row.get(0)?,
                content: row.get(1)?,
            })
        },
    )?
        .collect::<Result<Vec<Thought>, rusqlite::Error>>()?;

    Ok(thoughts)
}

