use async_trait::async_trait;
use deadpool_postgres::{
    tokio_postgres::{types::ToSql, Row},
    Pool,
};
use error::Error;

pub trait SQL {
    /// Gives the type the related table name in the schema
    fn get_tablename() -> &'static str;

    /// Returns a default SELECT part of a SQL which can be converted using `from_row`
    fn get_select() -> String {
        format!("SELECT * FROM {}", Self::get_tablename())
    }

    /// Returns a default SQL query with a `where_part`
    fn select_where(where_part: &str) -> String {
        format!("{} WHERE {}", Self::get_select(), where_part)
    }

    /// Returns a default SQL query with a `where_part`
    fn select_where_limit(where_part: &str, limit: i64) -> String {
        format!("{} LIMIT {}", Self::select_where(where_part), limit)
    }

    fn select_where_order(where_part: &str, order_part: &str) -> String {
        format!("{} ORDER BY {}", Self::select_where(where_part), order_part)
    }

    fn select_where_order_limit(where_part: &str, order_part: &str, limit: i64) -> String {
        let order = Self::select_where_order(where_part, order_part);
        format!("{} LIMIT {}", order, limit)
    }

    fn get_delete() -> String {
        format!("DELETE FROM {}", Self::get_tablename())
    }

    /// Returns a default SQL query with a `where_part`
    fn delete_where(where_part: &str) -> String {
        format!("{} WHERE {}", Self::get_delete(), where_part)
    }

    fn get_exists() -> String {
        format!("SELECT EXISTS (SELECT 1 FROM {})", Self::get_tablename(),)
    }

    fn get_exists_where(where_part: &str) -> String {
        let table = Self::get_tablename();
        format!(
            "SELECT EXISTS (SELECT 1 FROM {} WHERE {})",
            table, where_part
        )
    }
}

/// Allow converting a row to `Self`
pub trait FromRow {
    /// give the offset of the index within the query
    fn from_row(row: &Row, offset: usize) -> Self
    where
        Self: Sized;
}

pub trait FromRows {
    fn from_rows(rows: Vec<Row>, offset: usize) -> Vec<Self>
    where
        Self: Sized;
}

#[async_trait]
pub trait Queryable: FromRows {
    async fn query<S: AsRef<str> + Send>(
        db: &Pool,
        query: S,
        params: &[&(dyn ToSql + Sync)],
        offset: usize,
    ) -> Result<Vec<Self>, Error>
    where
        Self: Sized,
    {
        let db = db.get().await?;
        let query = query.as_ref();
        let prepared = db.prepare_cached(query).await?;
        let res = db.query(&prepared, params).await?;
        Ok(Self::from_rows(res, offset))
    }
}

#[async_trait]
pub trait OptQueryable: FromRow {
    async fn query_opt<S: AsRef<str> + Send>(
        db: &Pool,
        query: S,
        params: &[&(dyn ToSql + Sync)],
        offset: usize,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized,
    {
        let db = db.get().await?;
        let query = query.as_ref();
        let prepared = db.prepare_cached(query).await?;
        Ok(db
            .query_opt(&prepared, params)
            .await?
            .map(|i| Self::from_row(&i, offset)))
    }
}

#[async_trait]
pub trait OneQueryable: FromRow {
    async fn query_one<S: AsRef<str> + Send>(
        db: &Pool,
        query: S,
        params: &[&(dyn ToSql + Sync)],
        offset: usize,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let db = db.get().await?;
        let query = query.as_ref();
        let prepared = db.prepare_cached(query).await?;
        let res = db.query_one(&prepared, params).await?;
        Ok(Self::from_row(&res, offset))
    }
}

#[async_trait]
pub trait CheckAvailable: SQL {
    /// Delete all rows by a condition in a table
    async fn exists_where(
        db: &Pool,
        condition: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<bool, Error> {
        let db = db.get().await?;
        let prepared = db
            .prepare_cached(&Self::get_exists_where(condition))
            .await?;
        Ok(db.query_one(&prepared, params).await?.get(0))
    }

    /// Delete all rows in a table
    async fn exists(db: &Pool) -> Result<bool, Error> {
        let db = db.get().await?;
        let prepared = db.prepare_cached(&Self::get_exists()).await?;
        Ok(db.query_one(&prepared, &[]).await?.get(0))
    }
}

#[async_trait]
pub trait Deletable: SQL {
    /// Delete all rows by a condition in a table
    async fn delete(db: &Pool, condition: &str) -> Result<u64, Error> {
        let db = db.get().await?;
        let prepared = db.prepare_cached(&Self::delete_where(condition)).await?;
        Ok(db.execute(&prepared, &[]).await?)
    }

    /// Delete all rows in a table
    async fn delete_all(db: &Pool) -> Result<u64, Error> {
        let db = db.get().await?;
        let prepared = db.prepare_cached(&Self::get_delete()).await?;
        Ok(db.execute(&prepared, &[]).await?)
    }
}

impl<T: SQL> Deletable for T {}
impl<T: SQL> CheckAvailable for T {}
impl<T: FromRows> Queryable for T {}
impl<T: FromRow> OneQueryable for T {}
impl<T: FromRow> OptQueryable for T {}

// Auto implement `FromRows` for all types implementing `FromRow`
impl<T: FromRow> FromRows for T {
    fn from_rows(rows: Vec<Row>, offset: usize) -> Vec<Self>
    where
        Self: Sized,
    {
        rows.into_iter()
            .map(|i| Self::from_row(&i, offset))
            .collect()
    }
}

/// Executes a prepared query, using a statement-cache
pub async fn prepared_query<S: AsRef<str>, T: FromRow>(
    db: &Pool,
    query: S,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Vec<T>, Error> {
    let db = db.get().await?;
    let prepared = db.prepare_cached(query.as_ref()).await?;
    Ok(db
        .query(&prepared, params)
        .await
        .map_err::<Error, _>(|i| i.into())?
        .into_iter()
        .map(|i| T::from_row(&i, 0))
        .collect())
}

impl FromRow for Option<i32> {
    fn from_row(row: &Row, offset: usize) -> Self
    where
        Self: Sized,
    {
        row.get(offset + 0)
    }
}

impl FromRow for Option<String> {
    fn from_row(row: &Row, offset: usize) -> Self
    where
        Self: Sized,
    {
        row.get(offset + 0)
    }
}

impl FromRow for String {
    fn from_row(row: &Row, offset: usize) -> String
    where
        Self: Sized,
    {
        row.get(offset + 0)
    }
}

impl FromRow for i32 {
    fn from_row(row: &Row, offset: usize) -> i32
    where
        Self: Sized,
    {
        row.get(offset + 0)
    }
}
