use clap::{Args, Parser};
use confique::Config as _;
use std::path::PathBuf;

#[derive(confique::Config, Debug, Clone)]
pub struct AppConfig {
  #[config(nested)]
  pub server: Server,
  #[config(nested)]
  pub postgres: Postgres,
}

#[derive(confique::Config, Debug, Clone)]
pub struct Server {
  // Networking
  #[config(default = 8080, env = "{{project-name | shouty_snake_case}}_HTTP_PORT")]
  pub http_port: u16,
  #[config(
    default = 8443,
    env = "{{project-name | shouty_snake_case}}_HTTPS_PORT"
  )]
  pub https_port: u16,
  #[config(
    default = 9090,
    env = "{{project-name | shouty_snake_case}}_MONITORING_PORT"
  )]
  pub monitoring_port: u16,

  // TLS toggle
  #[config(
    default = false,
    env = "{{project-name | shouty_snake_case}}_TLS_ENABLED"
  )]
  pub tls_enabled: bool,

  // TLS/ACME configuration (optional)
  #[config(default = [])]
  pub domains: Vec<String>,
  #[config(default = [])]
  pub email: Vec<String>,

  pub cache: Option<PathBuf>,
  #[config(default = false)]
  pub production: bool,
  pub tls_key: Option<PathBuf>,
  pub tls_cert: Option<PathBuf>,
}

#[derive(Debug, Parser, Default, Clone)]
struct CliArgs {
  #[command(flatten)]
  server: ServerArgs,
  #[command(flatten)]
  auth: AuthArgs,
}

#[derive(Debug, Args, Default, Clone)]
struct ServerArgs {
  #[arg(long = "http-port")]
  http_port: Option<u16>,
  #[arg(long = "https-port")]
  https_port: Option<u16>,
  #[arg(long = "monitoring-port")]
  monitoring_port: Option<u16>,

  /// Whether to enable TLS
  #[arg(long = "tls-enabled")]
  pub tls_enabled: bool,

  /// The private key when tls-mode is keypair
  #[arg(long = "tls-key")]
  pub tls_key: Option<PathBuf>,

  /// The public key when tls-mode is keypair
  #[arg(long = "tls-cert")]
  pub tls_cert: Option<PathBuf>,
}

#[derive(Debug, Args, Default, Clone)]
struct AuthArgs {
  #[arg(long = "issuer-url")]
  issuer_url: Option<String>,
  #[arg(long = "audience")]
  audience: Option<String>,
  #[arg(long = "leeway-seconds")]
  leeway_seconds: Option<u64>,
}

pub fn load_config() -> eyre::Result<AppConfig> {
  load_config_with_args(std::env::args_os())
}

pub fn load_config_with_args<I, T>(args: I) -> eyre::Result<AppConfig>
where
  I: IntoIterator<Item = T>,
  T: Into<std::ffi::OsString> + Clone,
{
  // files + env
  let mut cfg = AppConfig::builder()
    .env()
    .file("config/local.toml")
    .file("/etc/{{project-name}}/secrets.toml")
    .file("/etc/{{project-name}}/config.toml")
    .file("config/default.toml")
    .load()
    .map_err(|e| eyre::eyre!(e.to_string()))?;

  // CLI overlay
  let cli = CliArgs::parse_from(args);
  if let Some(v) = cli.server.http_port {
    cfg.server.http_port = v;
  }
  if let Some(v) = cli.server.https_port {
    cfg.server.https_port = v;
  }
  if let Some(v) = cli.server.monitoring_port {
    cfg.server.monitoring_port = v;
  }
  if cli.server.tls_enabled {
    cfg.server.tls_enabled = true;
  }
  if let Some(v) = cli.server.tls_key {
    cfg.server.tls_key = Some(v);
  }
  if let Some(v) = cli.server.tls_cert {
    cfg.server.tls_cert = Some(v);
  }

  Ok(cfg)
}

#[derive(confique::Config, Debug, Clone)]
pub struct Postgres {
  #[config(env = "DATABASE_URL")]
  pub url: String,
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{
    env, fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
  };
  use uuid::Uuid;

  // Serialize env-dependent tests
  static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
  fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap()
  }

  fn write_temp_toml(contents: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("{{project-name}}-test-{}", Uuid::now_v7()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.toml");
    fs::write(&path, contents).unwrap();
    path
  }

  // Provide minimal uploads env so AppConfig can load without altering defaults.
  fn with_test_env<F, R>(f: F) -> R
  where
    F: FnOnce() -> R,
  {
    unsafe {
      env::set_var(
        "DATABASE_URL",
        "postgres://postgres:password@localhost:5432/{{database_name}}",
      );
    }
    let out = f();
    unsafe {
      env::remove_var("DATABASE_URL");
    }
    out
  }

  #[test]
  fn config_defaults_when_no_sources() {
    // No files, CLI empty. We set only uploads env to satisfy required fields.
    let _g = env_lock();
    let cfg = with_test_env(|| AppConfig::builder().env().load().unwrap());
    assert_eq!(cfg.server.http_port, 8080);
    assert_eq!(cfg.server.https_port, 8443);
    assert_eq!(cfg.server.monitoring_port, 9090);
    assert!(!cfg.server.tls_enabled);
  }

  #[test]
  fn file_overrides_defaults() {
    let _g = env_lock();
    let path = write_temp_toml(
      r#"[server]
http_port = 18080
https_port = 18443
monitoring_port = 19090
tls_enabled = true
"#,
    );

    let cfg = with_test_env(|| AppConfig::builder().env().file(&path).load().unwrap());

    assert_eq!(cfg.server.http_port, 18080);
    assert_eq!(cfg.server.https_port, 18443);
    assert_eq!(cfg.server.monitoring_port, 19090);
    assert!(cfg.server.tls_enabled);
  }

  #[test]
  fn env_overrides_files() {
    let _g = env_lock();
    let path = write_temp_toml(
      r#"[server]
http_port = 18080
"#,
    );

    unsafe {
      env::set_var("{{project-name | shouty_snake_case}}_HTTP_PORT", "28080");
    }

    let cfg = with_test_env(|| AppConfig::builder().env().file(&path).load().unwrap());

    assert_eq!(cfg.server.http_port, 28080);

    unsafe {
      env::remove_var("{{project-name | shouty_snake_case}}_HTTP_PORT");
    }
  }

  #[test]
  fn cli_overrides_env() {
    let _g = env_lock();
    unsafe {
      env::set_var("{{project-name | shouty_snake_case}}_HTTP_PORT", "28080");
    }

    let cfg = with_test_env(|| {
      super::load_config_with_args(["{{project-name}}-server", "--http-port", "38080"]).unwrap()
    });

    assert_eq!(cfg.server.http_port, 38080);

    unsafe {
      env::remove_var("{{project-name | shouty_snake_case}}_HTTP_PORT");
    }
  }
}
