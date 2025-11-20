#![allow(dead_code)]
#![allow(clippy::all)]

mod queries;

pub use queries::*;

pub fn create_pg_pool<S: AsRef<str>>(database_url: S) -> eyre::Result<deadpool_postgres::Pool> {
  let pool_mgr = deadpool_postgres::Manager::from_config(
    database_url.as_ref().parse()?,
    tokio_postgres::NoTls,
    Default::default(),
  );

  let pool = deadpool_postgres::Pool::builder(pool_mgr)
    .max_size(16)
    .build()?;
  Ok(pool)
}
