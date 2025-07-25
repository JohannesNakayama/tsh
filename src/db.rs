use include_dir::{Dir, include_dir};
use rusqlite::ffi::sqlite3_auto_extension;
use rusqlite::{Connection, Transaction};
use rusqlite_migration::Migrations;
use sqlite_vec::sqlite3_vec_init;
use std::sync::LazyLock;
use zerocopy::IntoBytes;

use crate::model::{Article, Zettel};

// TODO: move migrations dir to canonical location or specify in config.toml
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

pub async fn store_zettel(
    tx: &Transaction<'_>,
    content: &str,
    embedding: Vec<f32>,
    parent_ids: Vec<i64>,
) -> Result<(), rusqlite::Error> {
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

    tx.prepare("insert into zettel_edge (node_id) values (?)")?
        .execute(rusqlite::params![zettel.id])?;

    let mut insert_zettel_edge_stmt =
        tx.prepare("insert into zettel_edge (node_id, parent_id) values (?, ?)")?;
    for id in parent_ids {
        insert_zettel_edge_stmt.execute(rusqlite::params![zettel.id, id])?;
    }

    Ok(())
}

pub async fn find_zettel_by_id(tx: &Transaction<'_>, id: i64) -> Result<Zettel, rusqlite::Error> {
    let mut stmt = tx.prepare(
        "
        select id, content, created_at
        from zettel
        where id = ?
        ",
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

pub async fn find_zettels_by_embedding(
    tx: &Transaction<'_>,
    embedding: Vec<f32>,
) -> Result<Vec<Zettel>, rusqlite::Error> {
    let mut stmt = tx.prepare(
        "
        select id, content, created_at
        from zettel z
        join zettel_embedding ze on z.id = ze.zettel_id
        where ze.embedding match ?
        and k = 15
        ",
    )?;

    let thoughts: Vec<Zettel> = stmt
        .query_map([embedding.as_bytes()], |row| {
            Ok(Zettel {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<Zettel>, rusqlite::Error>>()?;

    Ok(thoughts)
}

pub async fn find_n_recent_leaf_zettels(
    tx: &Transaction<'_>,
    n: i64,
) -> Result<Vec<Zettel>, rusqlite::Error> {
    let mut stmt = tx.prepare(
        "
        with leaf_nodes as (
            select node_id
            from zettel_edge
            where node_id not in (
                select parent_id
                from zettel_edge
                where parent_id is not null
            )
            group by node_id
        )
        select z.id, z.content, z.created_at
        from zettel z
        inner join leaf_nodes ln on z.id = ln.node_id
        order by z.created_at desc
        limit ?
        ",
    )?;

    let n_recent_zettels: Vec<Zettel> = stmt
        .query_map([n], |row| {
            Ok(Zettel {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<Zettel>, rusqlite::Error>>()?;

    Ok(n_recent_zettels)
}

pub async fn store_article(
    tx: &Transaction<'_>,
    zettel_id: i64,
    title: &str,
    content: &str,
) -> Result<Article, rusqlite::Error> {
    let article: Article = tx
        .prepare(
            "
            insert into article (zettel_id , title , content)
            values (?, ?, ?)
            returning id, zettel_id, title, content, created_at
            ",
        )?
        .query_one((zettel_id, title, content), |row| {
            Ok(Article {
                id: row.get(0)?,
                zettel_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
    Ok(article)
}
