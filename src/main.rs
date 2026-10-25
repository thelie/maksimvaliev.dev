mod lang;

use std::{
    net::SocketAddr,
    sync::{LazyLock, OnceLock},
    time::Instant,
};

use axum::{
    Router,
    http::{HeaderMap, HeaderValue, StatusCode, Uri, header},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use lang::Lang;

/// A page rendered and compressed at build time.
struct Page {
    raw: &'static [u8],
    br: &'static [u8],
    gzip: &'static [u8],
    etag: &'static str,
}

/// Every page of the site in one language.
struct Site {
    index: Page,
    not_found: Page,
}

// Defines `SITES`, `SOURCE`, `COMMIT` and `RUSTC`.
include!(concat!(env!("OUT_DIR"), "/site.rs"));

const VERSION: &str = env!("CARGO_PKG_VERSION");

static STARTED: LazyLock<Instant> = LazyLock::new(Instant::now);
static MODE: OnceLock<String> = OnceLock::new();

fn accepts(req: &HeaderMap, encoding: &str) -> bool {
    req.get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.split(',').any(|e| e.split(';').next().unwrap().trim() == encoding))
}

impl Page {
    fn respond(&self, status: StatusCode, req: &HeaderMap) -> Response {
        let fresh = req.get(header::IF_NONE_MATCH).is_some_and(|v| v.as_bytes() == self.etag.as_bytes());

        let (body, encoding) = if accepts(req, "br") {
            (self.br, Some("br"))
        } else if accepts(req, "gzip") {
            (self.gzip, Some("gzip"))
        } else {
            (self.raw, None)
        };

        let mut res = if status.is_success() && fresh {
            StatusCode::NOT_MODIFIED.into_response()
        } else {
            let mut res = (status, body).into_response();
            if let Some(encoding) = encoding {
                res.headers_mut()
                    .insert(header::CONTENT_ENCODING, HeaderValue::from_static(encoding));
            }
            res
        };

        let h = res.headers_mut();
        h.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"));
        h.insert(header::VARY, HeaderValue::from_static("accept-encoding"));
        h.insert(header::ETAG, HeaderValue::from_static(self.etag));
        h.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=300"));
        h.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'none'; style-src 'unsafe-inline'; img-src data:; \
                 base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
            ),
        );
        h.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
        h.insert(header::REFERRER_POLICY, HeaderValue::from_static("strict-origin-when-cross-origin"));
        res
    }
}

fn site(lang: Lang) -> &'static Site {
    &SITES[lang as usize]
}

fn preferred(headers: &HeaderMap) -> Lang {
    headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .map_or(Lang::En, Lang::negotiate)
}

async fn root(headers: HeaderMap) -> Response {
    let mut res = Redirect::temporary(preferred(&headers).path()).into_response();
    let h = res.headers_mut();
    h.insert(header::VARY, HeaderValue::from_static("accept-language"));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    res
}

async fn not_found(uri: Uri, headers: HeaderMap) -> Response {
    let lang = Lang::ALL
        .into_iter()
        .find(|l| uri.path().strip_prefix(l.path()).is_some_and(|r| r.is_empty() || r.starts_with('/')))
        .unwrap_or_else(|| preferred(&headers));
    let mut res = site(lang).not_found.respond(StatusCode::NOT_FOUND, &headers);
    res.headers_mut()
        .append(header::VARY, HeaderValue::from_static("accept-language"));
    res
}

/// The binary reporting on itself.
async fn status() -> impl IntoResponse {
    let binary = std::env::current_exe()
        .and_then(std::fs::metadata)
        .map_or("unknown".into(), |m| format!("{:.1} MB", m.len() as f64 / 1_048_576.0));
    let up = STARTED.elapsed().as_secs();
    let body = format!(
        "maksimvaliev-dev {VERSION} ({COMMIT})\n\
         binary   {binary}\n\
         uptime   {}d {}h {}m {}s\n\
         serving  {}\n\
         built    {RUSTC}\n",
        up / 86400,
        up / 3600 % 24,
        up / 60 % 60,
        up % 60,
        MODE.get().map_or("", String::as_str),
    );
    ([(header::CACHE_CONTROL, "no-store")], body)
}

/// The sources this binary was built from, embedded at compile time.
async fn source() -> impl IntoResponse {
    let disposition = format!("attachment; filename=\"maksimvaliev.dev-{VERSION}-{COMMIT}.tar.gz\"");
    (
        [
            (header::CONTENT_TYPE, "application/gzip".to_owned()),
            (header::CONTENT_DISPOSITION, disposition),
            (header::CACHE_CONTROL, "public, max-age=3600".to_owned()),
        ],
        SOURCE,
    )
}

fn app() -> Router {
    Lang::ALL
        .into_iter()
        .fold(Router::new().route("/", get(root)), |router, lang| {
            router.route(
                lang.path(),
                get(move |headers: HeaderMap| async move {
                    site(lang).index.respond(StatusCode::OK, &headers)
                }),
            )
        })
        .route("/status", get(status))
        .route("/source", get(source))
        .route("/healthz", get(|| async { "ok" }))
        .fallback(not_found)
}

fn port(var: &str, default: u16) -> u16 {
    std::env::var(var).ok().and_then(|p| p.parse().ok()).unwrap_or(default)
}

async fn shutdown() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.ok() };
    #[cfg(unix)]
    let term = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<Option<()>>();
    tokio::select! { _ = ctrl_c => {}, _ = term => {} }
}

#[tokio::main]
async fn main() {
    LazyLock::force(&STARTED);
    println!("maksimvaliev-dev {VERSION} ({COMMIT})");

    #[cfg(feature = "tls")]
    if let Ok(domains) = std::env::var("ACME_DOMAIN") {
        let domains: Vec<String> = domains.split(',').map(|d| d.trim().to_owned()).collect();
        return tls::serve(app(), domains).await;
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port("PORT", 3000)));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind address");
    MODE.set(format!("http on {addr}")).unwrap();
    println!("listening on http://{addr}");

    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown())
        .await
        .expect("server error");
}

/// HTTPS with certificates from Let's Encrypt, obtained and renewed by the binary itself
/// (TLS-ALPN-01 challenge on the HTTPS port). Plain HTTP only redirects.
#[cfg(feature = "tls")]
mod tls {
    use std::{net::SocketAddr, sync::Arc, time::Duration};

    use axum::{
        Router,
        http::{HeaderMap, HeaderValue, Uri, header},
        middleware::map_response,
        response::{Redirect, Response},
    };
    use futures_util::StreamExt;
    use rustls_acme::{AcmeConfig, caches::DirCache};

    use crate::{MODE, port, shutdown};

    pub async fn serve(app: Router, domains: Vec<String>) {
        let staging = std::env::var("ACME_STAGING").is_ok_and(|v| v != "0");
        let cache = std::env::var("ACME_CACHE").unwrap_or_else(|_| "acme-cache".into());
        let mut state = AcmeConfig::new(domains.clone())
            .contact(std::env::var("ACME_EMAIL").ok().map(|e| format!("mailto:{e}")))
            .cache(DirCache::new(cache))
            .directory_lets_encrypt(!staging)
            .state();

        // The default config advertises no ALPN protocols, which would pin browsers to HTTP/1.1.
        let mut tls = (*state.default_rustls_config()).clone();
        tls.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        let acceptor = state.axum_acceptor(Arc::new(tls));

        tokio::spawn(async move {
            while let Some(event) = state.next().await {
                match event {
                    Ok(ok) => println!("acme: {ok:?}"),
                    Err(err) => eprintln!("acme error: {err:?}"),
                }
            }
        });

        let https_port = port("HTTPS_PORT", 443);
        let http_addr = SocketAddr::from(([0, 0, 0, 0], port("HTTP_PORT", 80)));
        let https_addr = SocketAddr::from(([0, 0, 0, 0], https_port));

        let redirect = Router::new().fallback(move |headers: HeaderMap, uri: Uri| {
            let domains = domains.clone();
            async move {
                // Only redirect to hosts we serve, so the Host header can't steer it elsewhere.
                let host = headers
                    .get(header::HOST)
                    .and_then(|h| h.to_str().ok())
                    .map(|h| h.split(':').next().unwrap())
                    .filter(|h| domains.iter().any(|d| d == h))
                    .unwrap_or(&domains[0]);
                let port = if https_port == 443 { String::new() } else { format!(":{https_port}") };
                let path = uri.path_and_query().map_or("/", |p| p.as_str());
                Redirect::permanent(&format!("https://{host}{port}{path}"))
            }
        });
        let listener = tokio::net::TcpListener::bind(http_addr).await.expect("bind HTTP address");
        tokio::spawn(async move {
            axum::serve(listener, redirect).with_graceful_shutdown(shutdown()).await
        });

        let handle = axum_server::Handle::new();
        tokio::spawn({
            let handle = handle.clone();
            async move {
                shutdown().await;
                handle.graceful_shutdown(Some(Duration::from_secs(10)));
            }
        });

        MODE.set(format!("https on {https_addr} (h2, Let's Encrypt{})", if staging { " staging" } else { "" }))
            .unwrap();
        println!("listening on https://{https_addr}, redirecting http://{http_addr}");

        let app = app.layer(map_response(|mut res: Response| async move {
            res.headers_mut().insert(
                header::STRICT_TRANSPORT_SECURITY,
                HeaderValue::from_static("max-age=63072000; includeSubDomains"),
            );
            res
        }));
        axum_server::bind(https_addr)
            .handle(handle)
            .acceptor(acceptor)
            .serve(app.into_make_service())
            .await
            .expect("server error");
    }
}
