use crate::parse::radicals;
use crate::{models::radical as DbRadical, DbPool};

/// Import radicals
pub async fn import(db: &DbPool, path: String) {
    println!("Clearing old radicals...");
    DbRadical::clear(db).await.unwrap();
    println!("Importing radicals...");
    let db = db.get().unwrap();

    radicals::parse(&path, |radical| {
        DbRadical::insert(&db, radical.into()).unwrap();
    });
}
