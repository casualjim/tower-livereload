use async_trait::async_trait;
use deadpool_postgres::{GenericClient, Pool};
use time::OffsetDateTime;
use tokio_postgres::error::SqlState;
use tower_sessions_core::{
  ExpiredDeletion, SessionStore,
  session::{Id, Record},
  session_store,
};

/// An error type for Postgres stores backed by deadpool/tokio-postgres.
#[derive(thiserror::Error, Debug)]
pub enum PgStoreError {
  /// A variant to map Postgres driver errors.
  #[error(transparent)]
  Postgres(#[from] tokio_postgres::Error),

  /// A variant to map pool acquisition errors.
  #[error(transparent)]
  Pool(#[from] deadpool_postgres::PoolError),

  /// A variant to map `rmp_serde` encode errors.
  #[error(transparent)]
  Encode(#[from] rmp_serde::encode::Error),

  /// A variant to map `rmp_serde` decode errors.
  #[error(transparent)]
  Decode(#[from] rmp_serde::decode::Error),

  #[error(transparent)]
  Generic(#[from] eyre::Report),
}

impl From<PgStoreError> for session_store::Error {
  fn from(err: PgStoreError) -> Self {
    match err {
      PgStoreError::Postgres(inner) => session_store::Error::Backend(inner.to_string()),
      PgStoreError::Pool(inner) => session_store::Error::Backend(inner.to_string()),
      PgStoreError::Decode(inner) => session_store::Error::Decode(inner.to_string()),
      PgStoreError::Encode(inner) => session_store::Error::Encode(inner.to_string()),
      PgStoreError::Generic(inner) => session_store::Error::Backend(inner.to_string()),
    }
  }
}

/// A PostgreSQL session store.
#[derive(Clone, Debug)]
pub struct PostgresStore {
  pool: Pool,
  schema_name: String,
  table_name: String,
}

impl PostgresStore {
  /// Create a new PostgreSQL store with the provided connection pool.
  pub fn new(pool: Pool) -> Self {
    Self {
      pool,
      schema_name: "tower_sessions".to_string(),
      table_name: "session".to_string(),
    }
  }

  /// Set the session table schema name with the provided name.
  pub fn with_schema_name(mut self, schema_name: impl AsRef<str>) -> Result<Self, String> {
    let schema_name = schema_name.as_ref();
    if !is_valid_identifier(schema_name) {
      return Err(format!(
        "Invalid schema name '{}'. Schema names must start with a letter or underscore \
                 (including letters with diacritical marks and non-Latin letters).Subsequent \
                 characters can be letters, underscores, digits (0-9), or dollar signs ($).",
        schema_name
      ));
    }

    schema_name.clone_into(&mut self.schema_name);
    Ok(self)
  }

  /// Set the session table name with the provided name.
  pub fn with_table_name(mut self, table_name: impl AsRef<str>) -> Result<Self, String> {
    let table_name = table_name.as_ref();
    if !is_valid_identifier(table_name) {
      return Err(format!(
        "Invalid table name '{}'. Table names must start with a letter or underscore \
                 (including letters with diacritical marks and non-Latin letters).Subsequent \
                 characters can be letters, underscores, digits (0-9), or dollar signs ($).",
        table_name
      ));
    }

    table_name.clone_into(&mut self.table_name);
    Ok(self)
  }

  /// Migrate the session schema.
  pub async fn migrate(&self) -> Result<(), PgStoreError> {
    let mut client = self.pool.get().await?;
    let tx = client.transaction().await?;

    let create_schema_query = format!(
      r#"create schema if not exists "{schema_name}""#,
      schema_name = self.schema_name,
    );

    if let Err(err) = tx.batch_execute(&create_schema_query).await {
      let duplicate = matches!(
        err.code(),
        Some(code) if code == &SqlState::DUPLICATE_SCHEMA || code == &SqlState::UNIQUE_VIOLATION
      );

      if !duplicate {
        return Err(err.into());
      }
    }

    let create_table_query = format!(
      r#"
            create table if not exists "{schema_name}"."{table_name}"
            (
                id text primary key not null,
                data bytea not null,
                expiry_date timestamptz not null
            )
            "#,
      schema_name = self.schema_name,
      table_name = self.table_name
    );
    tx.batch_execute(&create_table_query).await?;

    tx.commit().await?;
    Ok(())
  }

  async fn id_exists(&self, client: &impl GenericClient, id: &Id) -> session_store::Result<bool> {
    let query = format!(
      r#"
            select exists(select 1 from "{schema_name}"."{table_name}" where id = $1)
            "#,
      schema_name = self.schema_name,
      table_name = self.table_name
    );

    let row = client
      .query_one(query.as_str(), &[&id.to_string()])
      .await
      .map_err(PgStoreError::from)?;
    Ok(row.get::<_, bool>(0))
  }

  async fn save_with_conn(
    &self,
    client: &impl GenericClient,
    record: &Record,
  ) -> session_store::Result<()> {
    let query = format!(
      r#"
            insert into "{schema_name}"."{table_name}" (id, data, expiry_date)
            values ($1, $2, $3)
            on conflict (id) do update
            set
              data = excluded.data,
              expiry_date = excluded.expiry_date
            "#,
      schema_name = self.schema_name,
      table_name = self.table_name
    );

    let payload = rmp_serde::to_vec(record).map_err(PgStoreError::Encode)?;
    let expiry = timestamp_from_offset(record.expiry_date).map_err(PgStoreError::from)?;

    client
      .execute(query.as_str(), &[&record.id.to_string(), &payload, &expiry])
      .await
      .map_err(PgStoreError::from)?;

    Ok(())
  }
}

#[async_trait]
impl ExpiredDeletion for PostgresStore {
  async fn delete_expired(&self) -> session_store::Result<()> {
    let query = format!(
      r#"
            delete from "{schema_name}"."{table_name}"
            where expiry_date < (now() at time zone 'utc')
            "#,
      schema_name = self.schema_name,
      table_name = self.table_name
    );
    let client = self.pool.get().await.map_err(PgStoreError::from)?;
    client
      .execute(query.as_str(), &[])
      .await
      .map_err(PgStoreError::from)?;
    Ok(())
  }
}

#[async_trait]
impl SessionStore for PostgresStore {
  async fn create(&self, record: &mut Record) -> session_store::Result<()> {
    let mut client = self.pool.get().await.map_err(PgStoreError::from)?;
    let tx = client.transaction().await.map_err(PgStoreError::from)?;

    while self.id_exists(&tx, &record.id).await? {
      record.id = Id::default();
    }
    self.save_with_conn(&tx, record).await?;

    tx.commit().await.map_err(PgStoreError::from)?;

    Ok(())
  }

  async fn save(&self, record: &Record) -> session_store::Result<()> {
    let client = self.pool.get().await.map_err(PgStoreError::from)?;
    self.save_with_conn(&client, record).await
  }

  async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
    let query = format!(
      r#"
            select data from "{schema_name}"."{table_name}"
            where id = $1 and expiry_date > $2
            "#,
      schema_name = self.schema_name,
      table_name = self.table_name
    );
    let client = self.pool.get().await.map_err(PgStoreError::from)?;
    let now = timestamp_from_offset(OffsetDateTime::now_utc()).map_err(PgStoreError::from)?;
    let record_value = client
      .query_opt(query.as_str(), &[&session_id.to_string(), &now])
      .await
      .map_err(PgStoreError::from)?;

    if let Some(row) = record_value {
      let data: Vec<u8> = row.get(0);
      Ok(Some(
        rmp_serde::from_slice(&data).map_err(PgStoreError::Decode)?,
      ))
    } else {
      Ok(None)
    }
  }

  async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
    let query = format!(
      r#"delete from "{schema_name}"."{table_name}" where id = $1"#,
      schema_name = self.schema_name,
      table_name = self.table_name
    );
    let client = self.pool.get().await.map_err(PgStoreError::from)?;
    client
      .execute(query.as_str(), &[&session_id.to_string()])
      .await
      .map_err(PgStoreError::from)?;

    Ok(())
  }
}

/// A valid PostreSQL identifier must start with a letter or underscore
/// (including letters with diacritical marks and non-Latin letters). Subsequent
/// characters in an identifier or key word can be letters, underscores, digits
/// (0-9), or dollar signs ($). See https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS for details.
fn is_valid_identifier(name: &str) -> bool {
  !name.is_empty()
    && name
      .chars()
      .next()
      .map(|c| c.is_alphabetic() || c == '_')
      .unwrap_or_default()
    && name
      .chars()
      .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

fn timestamp_from_offset(dt: OffsetDateTime) -> eyre::Result<jiff::Timestamp> {
  Ok(jiff::Timestamp::from_nanosecond(dt.unix_timestamp_nanos())?)
}
