use deadpool_postgres::Pool;
use http::StatusCode;

use crate::{
  assets::{AssetCache, SharedAssetCache},
  config::AppConfig,
  error::AppError,
  pgdb,
};

fn leak_alloc<G>(value: G) -> &'static G {
  Box::leak(Box::new(value))
}

#[derive(Clone)]
pub struct AppState {
  pgdb: Pool,
  assets: SharedAssetCache,
}

impl AppState {
  pub async fn new(config: &AppConfig) -> eyre::Result<Self> {
    let pgdb = pgdb::create_pg_pool(&config.postgres.url)?;
    let assets = leak_alloc(AssetCache::load_files(None, &[]).await);

    Ok(Self { pgdb, assets })
  }

  pub fn assets(&self) -> SharedAssetCache {
    self.assets
  }

  pub fn pgdb(&self) -> Pool {
    self.pgdb.clone()
  }

  pub async fn try_pgconn(&self) -> Result<deadpool_postgres::Client, AppError> {
    self
      .pgdb
      .get()
      .await
      .map_err(|e| AppError::new(&e.to_string()).with_status(StatusCode::INTERNAL_SERVER_ERROR))
  }
}
