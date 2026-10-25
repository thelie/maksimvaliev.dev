# maksimvaliev.dev

Personal site, and a single self-contained binary: Rust + Axum, with HTTPS and Let's Encrypt built in.

- **Compile-time rendering.** `build.rs` renders every page from `content/*.toml`, compresses each with Brotli (q11) and gzip (level 9), and embeds the bytes. At runtime the server only picks a pre-built response — no templating, no compression, no startup work.
- **TLS in the binary.** With `ACME_DOMAIN` set, it obtains and renews certificates itself (TLS-ALPN-01 via `rustls-acme`), serves HTTP/2 + HTTP/1.1 on :443 and redirects :80. No reverse proxy, no certbot.
- **Static.** The Docker build produces a musl binary with no glibc dependency — copy it to any Linux box.
- **Self-describing.** `/status` reports version, commit, binary size and uptime; `/source` serves the exact sources it was built from.

## Develop

```sh
$EDITOR content/en.toml content/ru.toml   # all copy, per language
cargo run                                 # http://localhost:3000 → /en or /ru
cargo test
```

## Ship

The site runs on a shared Selectel server (`ssh gaure`), behind its pingap reverse proxy: the binary serves plain HTTP on `127.0.0.1:8090`, pingap terminates TLS for maksimvaliev.dev, and certbot (webroot `/var/www/acme`) issues the certificate. The pingap config lives in the repo of the other site on that server, `site/server/pingap.toml`.

```sh
docker build --platform linux/amd64 --output dist .   # → dist/maksimvaliev-dev (static, x86_64)
scp dist/maksimvaliev-dev gaure:/tmp/
ssh gaure 'sudo install -m 755 /tmp/maksimvaliev-dev /usr/local/bin/ && sudo systemctl restart maksimvaliev-dev'
```

First-time setup: install `deploy/maksimvaliev-dev.service` into `/etc/systemd/system/` and `deploy/certbot-deploy-pingap.sh` as `/etc/letsencrypt/renewal-hooks/deploy/pingap-maksimvaliev.sh`.

On a server of its own the binary can do HTTPS itself (TLS-ALPN-01 on :443, see the comments in the unit file). Do the first run with `ACME_STAGING=1` to check issuance without hitting Let's Encrypt rate limits.

## Configuration

| Variable       | Default       | Meaning                                                   |
| -------------- | ------------- | --------------------------------------------------------- |
| `ACME_DOMAIN`  | —             | Comma-separated domains. Unset → plain HTTP on `PORT`.    |
| `ACME_EMAIL`   | —             | Let's Encrypt contact (expiry notices).                   |
| `ACME_CACHE`   | `acme-cache`  | Directory for the ACME account and certificates.          |
| `ACME_STAGING` | off           | `1` uses the Let's Encrypt staging environment.           |
| `HTTPS_PORT`   | `443`         | TLS listener.                                             |
| `HTTP_PORT`    | `80`          | Redirect-to-HTTPS listener.                               |
| `PORT`         | `3000`        | Plain HTTP listener when TLS is off (e.g. behind a proxy).|

`cargo build --release --no-default-features` drops the TLS stack (~0.9 MB instead of ~3.4 MB for static Linux) for running behind a reverse proxy.

## Routes

`/en`, `/ru` — the site · `/` — redirect by `Accept-Language` · `/status` · `/source` · `/healthz` · anything else → localized 404.

## Layout

- `content/` — copy per language: bio, projects, résumé, links, UI strings
- `build.rs` — renders, compresses and embeds everything; packs `/source`
- `build/` — content model, [Maud](https://maud.lambda.xyz) templates, CSS (build-time only)
- `src/main.rs` — server: routing, content negotiation, caching headers, TLS
- `src/lang.rs` — languages and `Accept-Language` negotiation (shared with `build.rs`)
- `deploy/` — systemd unit
