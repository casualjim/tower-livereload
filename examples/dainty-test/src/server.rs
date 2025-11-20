use std::{
  net::{Ipv4Addr, SocketAddr, TcpListener},
  sync::Arc,
  time::Duration,
};

use axum::{
  BoxError, Router,
  extract::{Path, Request as AxumRequest, State},
  handler::HandlerWithoutStateExt as _,
  middleware::{self, Next},
  response::{IntoResponse as _, Redirect, Response},
  routing::get,
};

use axum_helmet::{
  ContentSecurityPolicy, CrossOriginOpenerPolicy, CrossOriginResourcePolicy, Helmet, HelmetLayer,
  OriginAgentCluster, ReferrerPolicy, StrictTransportSecurity, XContentTypeOptions,
  XDNSPrefetchControl, XDownloadOptions, XFrameOptions, XPermittedCrossDomainPolicies,
  XXSSProtection,
};
use axum_otel_metrics::{HttpMetricsLayer, HttpMetricsLayerBuilder};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use futures::StreamExt;
#[cfg(debug_assertions)]
use http::Request;
use http::{
  HeaderMap, HeaderValue, StatusCode, Uri,
  header::{CONTENT_ENCODING, CONTENT_TYPE},
};
use listenfd::ListenFd;
use rustls::ServerConfig;
use rustls_acme::{AcmeConfig, caches::DirCache};
use tokio::{
  signal,
  task::{AbortHandle, JoinHandle},
  time::sleep,
};
use tower_http::compression::CompressionLayer;
use tower_sessions::{SessionManagerLayer, session_store::ExpiredDeletion};
use tracing::{debug, error, info, warn};

use crate::{app::AppState, config::AppConfig, tokio_postgres_sessions::PostgresStore};

fn build_admin_router() -> Router {
  Router::new().route("/healthz", get(|| async { "OK" }))
}

const IDX_HTTP: usize = 0;
const IDX_HTTPS: usize = 1;
const IDX_MONITORING: usize = 2;

async fn build_app(state: AppState) -> eyre::Result<Router> {
  debug!("Building server app");

  // Generate OpenAPI and compose routes
  let protected_routes = crate::routes::router(state.clone());

  debug!("Created server routes");

  // Global layers - CSP configured for Alpine.js
  #[cfg(debug_assertions)]
  let script_src = vec!["'self'", "'unsafe-eval'", "'unsafe-inline'"];
  #[cfg(not(debug_assertions))]
  let script_src = vec!["'self'", "'unsafe-eval'"];

  let csp = ContentSecurityPolicy::new()
    .default_src(vec!["'self'"])
    .base_uri(vec!["'self'"])
    .font_src(vec!["'self'", "https:", "data:"])
    .form_action(vec!["'self'"])
    .frame_ancestors(vec!["'self'"])
    .img_src(vec!["'self'", "data:"])
    .object_src(vec!["'none'"])
    .script_src(script_src)
    .script_src_attr(vec!["'unsafe-eval'"])
    .style_src(vec!["'self'", "https:", "'unsafe-inline'"])
    .upgrade_insecure_requests();

  let helmet = Helmet::new()
    .add(csp)
    .add(CrossOriginOpenerPolicy::same_origin())
    .add(CrossOriginResourcePolicy::same_origin())
    .add(OriginAgentCluster::new(true))
    .add(ReferrerPolicy::no_referrer())
    .add(
      StrictTransportSecurity::new()
        .max_age(15552000)
        .include_sub_domains(),
    )
    .add(XContentTypeOptions::nosniff())
    .add(XDNSPrefetchControl::off())
    .add(XDownloadOptions::noopen())
    .add(XFrameOptions::same_origin())
    .add(XPermittedCrossDomainPolicies::none())
    .add(XXSSProtection::off());

  let app = Router::new()
    .merge(protected_routes)
    .nest_service("/static", static_file_handler(state.clone()))
    .layer(HelmetLayer::new(helmet))
    .layer(OtelInResponseLayer)
    .layer(OtelAxumLayer::default())
    .layer(metrics_layer())
    .with_state(state);

  debug!("Created routers");
  Ok(app)
}

#[derive(Clone, Copy)]
struct Ports {
  http: u16,
  https: u16,
}

#[cfg(debug_assertions)]
fn not_htmx_predicate<T>(req: &Request<T>) -> bool {
  !req.headers().contains_key("hx-request")
}

pub async fn run(
  args: crate::config::AppConfig,
  shutdown_token: Option<tokio_util::sync::CancellationToken>,
) -> eyre::Result<()> {
  debug!("Entering run function");
  // Extract all needed fields from args first
  let tls_enabled = args.server.tls_enabled;
  let http_port = args.server.http_port;
  let https_port = args.server.https_port;

  debug!("Enabling TLS: {tls_enabled}");
  // Get TLS config if needed (before moving indexer)
  let tls_config_result = if tls_enabled {
    Some(make_tls_config(&args.server).await?)
  } else {
    None
  };

  let state = AppState::new(&args).await?;

  let session_store = PostgresStore::new(state.pgdb());
  session_store.migrate().await?;

  let deletion_task = tokio::task::spawn(
    session_store
      .clone()
      .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
  );

  let server_handle = Handle::new();

  let session_layer = SessionManagerLayer::new(session_store)
    .with_secure(true)
    .with_http_only(true)
    .with_same_site(tower_sessions::cookie::SameSite::Lax);

  let app = build_app(state.clone()).await?.layer(session_layer);
  debug!("App built, config: {args:?}");

  #[cfg(debug_assertions)]
  let app = {
    use notify::Watcher;
    let livereload = tower_livereload::LiveReloadLayer::new().request_predicate(not_htmx_predicate);
    let reloader = livereload.reloader();
    let mut watcher = notify::recommended_watcher(move |_| reloader.reload()).unwrap();
    watcher
      .watch(
        std::path::Path::new("public"),
        notify::RecursiveMode::Recursive,
      )
      .unwrap();

    info!("Reloading!");
    app.layer(livereload)
  };

  let app = app.layer(CompressionLayer::new().quality(tower_http::CompressionLevel::Default));

  // Prepare listenfd and start admin server
  let mut listenfd = prepare_listenfd();
  start_admin_server(&mut listenfd, &args, server_handle.clone())?;

  tokio::spawn(graceful_shutdown(
    server_handle.clone(),
    deletion_task.abort_handle(),
    shutdown_token.clone(),
  ));

  if let Some((tls_config, _jh)) = tls_config_result {
    run_tls(
      app,
      &mut listenfd,
      http_port,
      https_port,
      server_handle.clone(),
      tls_config,
    )
    .await?;
  } else {
    run_http(app, &mut listenfd, http_port, server_handle.clone()).await?;
  }

  debug!("Server run function completing");
  Ok(())
}

fn metrics_layer() -> HttpMetricsLayer {
  HttpMetricsLayerBuilder::new().build()
}

fn prepare_listenfd() -> ListenFd {
  ListenFd::from_env()
}

fn acquire_listener(
  listenfd: &mut ListenFd,
  idx: usize,
  port: u16,
  name: &str,
) -> eyre::Result<TcpListener> {
  if let Some(l) = listenfd.take_tcp_listener(idx)? {
    return Ok(l);
  }
  let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
  TcpListener::bind(addr).map_err(|e| eyre::eyre!("failed to bind {name} to {}: {}", addr, e))
}

fn start_admin_server(
  listenfd: &mut ListenFd,
  args: &AppConfig,
  handle: Handle,
) -> eyre::Result<()> {
  let listener = acquire_listener(
    listenfd,
    IDX_MONITORING,
    args.server.monitoring_port,
    "monitoring",
  )?;
  let router = build_admin_router();
  spawn_admin_server(listener, router, handle);
  Ok(())
}

fn spawn_admin_server(listener: TcpListener, router: Router, handle: Handle) {
  tokio::spawn(async move {
    debug!(addr = %listener.local_addr().unwrap(), "starting admin HTTP server");
    if let Err(e) = axum_server::from_tcp(listener)
      .handle(handle)
      .serve(router.into_make_service())
      .await
    {
      error!("Admin server error: {}", e);
    }
    info!("Admin server stopped");
  });
}

async fn run_tls(
  app: Router,
  listenfd: &mut ListenFd,
  http_port: u16,
  https_port: u16,
  handle: Handle,
  tls_config: RustlsConfig,
) -> eyre::Result<()> {
  let http_listener = acquire_listener(listenfd, IDX_HTTP, http_port, "HTTP")?;
  let https_listener = acquire_listener(listenfd, IDX_HTTPS, https_port, "HTTPS")?;

  let ports = Ports {
    http: http_port,
    https: https_port,
  };

  tokio::spawn(redirect_http_to_https(ports, http_listener, handle.clone()));

  debug!(addr = %https_listener.local_addr().unwrap(), "starting HTTPS server");
  let mut server = axum_server::from_tcp_rustls(https_listener, tls_config).handle(handle.clone());
  server.http_builder().http2().enable_connect_protocol();
  server
    .serve(app.into_make_service_with_connect_info::<SocketAddr>())
    .await?;

  Ok(())
}

async fn run_http(
  app: Router,
  listenfd: &mut ListenFd,
  http_port: u16,
  handle: Handle,
) -> eyre::Result<()> {
  let http_listener = acquire_listener(listenfd, IDX_HTTP, http_port, "HTTP")?;
  debug!(addr = %http_listener.local_addr().unwrap(), "starting HTTP server (TLS disabled)");
  axum_server::from_tcp(http_listener)
    .handle(handle.clone())
    .serve(app.into_make_service_with_connect_info::<SocketAddr>())
    .await?;

  Ok(())
}

async fn make_tls_config(
  args: &crate::config::Server,
) -> eyre::Result<(RustlsConfig, JoinHandle<()>)> {
  let (config, maybe_state) = match (&args.tls_cert, &args.tls_key) {
    (None, None) => {
      // we're in acme mode
      let state = AcmeConfig::new(args.domains.iter())
        .contact(args.email.iter().map(|e| format!("mailto:{}", e)))
        .cache_option(args.cache.clone().map(DirCache::new))
        .directory_lets_encrypt(args.production)
        .state();

      let tls_config = Arc::new(
        ServerConfig::builder()
          .with_no_client_auth()
          .with_cert_resolver(state.resolver()),
      );
      tracing::debug!("starting server with letsencrypt");

      (RustlsConfig::from_config(tls_config), Some(state))
    }
    (Some(cert_path), Some(key_path)) => {
      tracing::debug!("starting server: key={key_path:?}, cert={cert_path:?}");

      let config = RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .map_err(|e| eyre::eyre!("Failed to load TLS certificates: {}", e))?;

      (config, None)
    }
    _ => {
      return Err(eyre::anyhow!(
        "Both --tls-cert and --tls-key must be provided together"
      ));
    }
  };

  // Clone values for the reloading task before moving into the async block
  let config_clone = config.clone();
  let cert_path = args.tls_cert.clone();
  let key_path = args.tls_key.clone();

  let jh = tokio::spawn(async move {
    // Start periodic reloading task if we're in keypair mode
    if let (Some(cert_path), Some(key_path)) = (cert_path, key_path) {
      tokio::spawn(async move {
        // Run periodic reload every hour
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));

        loop {
          interval.tick().await;

          match config_clone
            .reload_from_pem_file(&cert_path, &key_path)
            .await
          {
            Ok(_) => info!("TLS certificates reloaded successfully"),
            Err(e) => warn!("Failed to reload TLS certificates: {}", e),
          }
        }
      });
    }

    // Handle ACME state if present
    if let Some(mut state) = maybe_state {
      loop {
        match state.next().await.unwrap() {
          Ok(ok) => info!("event: {ok:?}"),
          Err(err) => error!("error: {:?}", err),
        }
      }
    } else {
      // For keypair mode, we need to keep the task alive
      // The reloading is handled in the spawned task above
      futures::future::pending::<()>().await;
    }
  });

  Ok((config, jh))
}

async fn redirect_http_to_https(ports: Ports, listener: TcpListener, handle: Handle) {
  fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
    let mut parts = uri.into_parts();

    parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

    if parts.path_and_query.is_none() {
      parts.path_and_query = Some("/".parse().unwrap());
    }

    let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
    parts.authority = Some(https_host.parse()?);

    Ok(Uri::from_parts(parts)?)
  }

  let redirect = move |request: AxumRequest| async move {
    let host = request.uri().host().unwrap().to_string();
    let uri = request.uri().clone();
    match make_https(host, uri, ports) {
      Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
      Err(error) => {
        tracing::warn!(%error, "failed to convert URI to HTTPS");
        Err(StatusCode::BAD_REQUEST)
      }
    }
  };

  tracing::debug!("listening on {}", listener.local_addr().unwrap());

  let server = axum_server::from_tcp(listener)
    .handle(handle)
    .serve(redirect.into_make_service());

  if let Err(e) = server.await {
    error!("HTTP redirect server error: {}", e);
  }
  info!("HTTP redirect server stopped");
}

async fn graceful_shutdown(
  handle: Handle,
  deletion_task: AbortHandle,
  external_token: Option<tokio_util::sync::CancellationToken>,
) {
  match external_token {
    Some(token) => {
      // Wait only for the external token - signals are handled elsewhere
      token.cancelled().await;
      info!("Shutdown requested via external token");
    }
    None => {
      // Only set up signal handlers if no external token is provided
      let ctrl_c = async {
        signal::ctrl_c()
          .await
          .expect("failed to install Ctrl+C handler");
      };

      #[cfg(unix)]
      let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
          .expect("failed to install signal handler")
          .recv()
          .await;
      };

      #[cfg(not(unix))]
      let terminate = std::future::pending::<()>();

      tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
      }
    }
  }

  info!("waiting for connections to close");
  handle.graceful_shutdown(Some(Duration::from_secs(10)));
  loop {
    let count = handle.connection_count();
    if count == 0 {
      break;
    }
    debug!("alive connections count={count}",);
    sleep(Duration::from_secs(1)).await;
  }
  deletion_task.abort();
  debug!("graceful shutdown complete");
}

fn static_file_handler(state: AppState) -> Router {
  Router::new()
    .route(
      "/{*file}",
      get(
        |State(state): State<AppState>, path: Path<String>| async move {
          info!("serving static file: {}", path.as_str());
          let assets = state.assets();
          let Some(asset) = assets.get_from_path(&path) else {
            return StatusCode::NOT_FOUND.into_response();
          };

          let mut headers = HeaderMap::new();

          // We set the content type explicitly here as it will otherwise
          // be inferred as an `octet-stream`
          headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(asset.ext().unwrap_or("")),
          );

          if let Some(ext) = asset.ext()
            && let Some(mime) = mime_guess::from_ext(ext).first()
          {
            headers.insert(CONTENT_TYPE, HeaderValue::from_str(mime.as_ref()).unwrap());
          }

          if [Some("css"), Some("js")].contains(&asset.ext()) {
            headers.insert(CONTENT_ENCODING, HeaderValue::from_static("br"));
          }

          // `bytes::Bytes` clones are cheap
          (headers, asset.contents.clone()).into_response()
        },
      ),
    )
    .layer(middleware::from_fn(cache_control))
    .with_state(state)
}

async fn cache_control(request: AxumRequest, next: Next) -> Response {
  let mut response = next.run(request).await;

  if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
    const CACHEABLE_CONTENT_TYPES: [&str; 6] = [
      "text/css",
      "application/javascript",
      "image/svg+xml",
      "image/webp",
      "font/woff2",
      "image/png",
    ];

    if CACHEABLE_CONTENT_TYPES.iter().any(|&ct| content_type == ct) {
      let value = format!("public, max-age={}", 60 * 60 * 24);

      if let Ok(value) = HeaderValue::from_str(&value) {
        response.headers_mut().insert("cache-control", value);
      }
    }
  }

  response
}
