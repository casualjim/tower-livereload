mod app;
mod config;
mod error;
pub mod logging;
mod pgdb;
mod routes;
pub mod server;
pub mod tokio_postgres_sessions;

pub use config::load_config;
mod assets;

use tracing::error;

pub fn init_propagation() -> eyre::Result<()> {
  setup_panic_hook();
  if let Err(e) = init_tracing_opentelemetry::init_propagator() {
    error!("Failed to initialize tracing propagator: {e}");
    return Err(e.into());
  }
  Ok(())
}

fn setup_panic_hook() {
  std::panic::set_hook(Box::new(|panic| {
    if let Some(location) = panic.location() {
      error!(
        message = %panic,
        panic.file = location.file(),
        panic.line = location.line(),
        panic.column = location.column(),
      );
    } else {
      error!(message = %panic);
    }
    futures::executor::block_on(async {
      opentelemetry::global::tracer_provider();
    });
  }));
}
