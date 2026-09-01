#![cfg(not(target_arch = "wasm32"))]

use std::fs::File;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use thirtyfour::prelude::*;
use tiny_http::{Header, Method, Response, Server, StatusCode};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn find_dist() -> PathBuf {
    let root = crate_root();
    let candidates = [
        root.join("dist"),
        root.join("dist/public"),
        root.join("target/dx/hesocial-frontend/release/web/public"),
        root.join("target/dx/hesocial-frontend/debug/web/public"),
    ];
    for candidate in candidates {
        if candidate.join("index.html").is_file() {
            return candidate;
        }
    }
    panic!(
        "built SPA not found under frontend-rust/dist (looked for index.html). \
         Run `dx bundle --web --release --out-dir dist` first."
    );
}

fn mime_for(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "text/javascript",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css",
        Some("svg") => "image/svg+xml",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

struct StaticHarness {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl StaticHarness {
    fn shutdown(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread.take() {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                let _ = handle.join();
                let _ = tx.send(());
            });
            rx.recv_timeout(Duration::from_secs(3))
                .expect("static server thread did not exit after shutdown");
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if addr_closed(self.addr) {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!(
            "static server still listening on {} after shutdown",
            self.addr
        );
    }
}

impl Drop for StaticHarness {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

fn start_static_server(root: PathBuf) -> StaticHarness {
    let server = Server::http("127.0.0.1:0").expect("bind static server");
    let addr = server
        .server_addr()
        .to_ip()
        .expect("static server must bind an IP port");
    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = stop.clone();
    let thread = thread::spawn(move || {
        while !stop_thread.load(Ordering::SeqCst) {
            let mut request = match server.recv_timeout(Duration::from_millis(50)) {
                Ok(Some(request)) => request,
                Ok(None) => continue,
                Err(_) => break,
            };
            let raw = request.url().split('?').next().unwrap_or("/");
            let path_only = raw.split('#').next().unwrap_or(raw);

            if path_only == "/api/auth/login" && *request.method() == Method::Post {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let parsed: serde_json::Value =
                    serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
                let email = parsed.get("email").and_then(|v| v.as_str()).unwrap_or("");
                let password = parsed
                    .get("password")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if email == "slow@example.com" {
                    thread::sleep(Duration::from_millis(1500));
                }

                let (status, payload) = if email == "ok@example.com" && password == "secret"
                    || email == "slow@example.com"
                {
                    (
                        200,
                        r#"{"success":true,"data":{"token":"e2e-login-token","user":{"id":"1","email":"ok@example.com"}}}"#,
                    )
                } else {
                    (
                        401,
                        r#"{"success":false,"error":"Invalid email or password"}"#,
                    )
                };
                let header = Header::from_bytes(b"Content-Type", b"application/json")
                    .expect("json content-type");
                let _ = request.respond(
                    Response::from_string(payload)
                        .with_status_code(StatusCode::from(status))
                        .with_header(header),
                );
                continue;
            }

            if path_only == "/api/auth/google" {
                let header = Header::from_bytes(b"Content-Type", b"text/html; charset=utf-8")
                    .expect("html content-type");
                let _ = request.respond(
                    Response::from_string(
                        r#"<!doctype html><html><body><h1 id="google-oauth-stub">Google OAuth stub</h1></body></html>"#,
                    )
                    .with_header(header),
                );
                continue;
            }

            let relative = path_only.trim_start_matches('/');
            let mut path = if relative.is_empty() {
                root.join("index.html")
            } else {
                root.join(relative)
            };
            if !path.exists() && !relative.contains('.') {
                path = root.join("index.html");
            }
            if !path.exists() {
                let _ = request.respond(Response::from_string("not found").with_status_code(404));
                continue;
            }
            match File::open(&path) {
                Ok(file) => {
                    let header = Header::from_bytes(b"Content-Type", mime_for(&path).as_bytes())
                        .expect("content-type header");
                    let _ = request.respond(Response::from_file(file).with_header(header));
                }
                Err(_) => {
                    let _ =
                        request.respond(Response::from_string("unreadable").with_status_code(500));
                }
            }
        }
    });
    StaticHarness {
        addr,
        stop,
        thread: Some(thread),
    }
}

fn http_get_status(addr: SocketAddr, path: &str) -> std::io::Result<u16> {
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    )?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    let _ = stream.shutdown(Shutdown::Both);
    let code = buf
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    Ok(code)
}

fn addr_closed(addr: SocketAddr) -> bool {
    TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_err()
}

#[test]
fn harness_starts_and_stops_twice() {
    let dist = find_dist();

    let first = start_static_server(dist.clone());
    let first_addr = first.addr;
    let status = http_get_status(first_addr, "/").expect("first harness must serve");
    assert_eq!(status, 200, "first harness GET /");
    first.shutdown();
    assert!(
        addr_closed(first_addr),
        "first harness still listening on {first_addr} after shutdown"
    );

    let second = start_static_server(dist);
    let second_addr = second.addr;
    let status = http_get_status(second_addr, "/").expect("second harness must serve");
    assert_eq!(status, 200, "second harness GET /");
    second.shutdown();
    assert!(
        addr_closed(second_addr),
        "second harness still listening on {second_addr} after shutdown"
    );
}

async fn wait_text(driver: &WebDriver, id: &str, expected: &str) -> WebDriverResult<String> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    let mut last = String::new();
    loop {
        if let Ok(el) = driver.find(By::Id(id)).await {
            last = el.text().await.unwrap_or_default();
            if last == expected {
                return Ok(last);
            }
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("timed out waiting for #{id} text {expected:?}; last={last:?}");
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

async fn wait_present(driver: &WebDriver, id: &str) -> WebDriverResult<WebElement> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        if let Ok(el) = driver.find(By::Id(id)).await {
            return Ok(el);
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("timed out waiting for #{id}");
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

async fn local_storage_get(driver: &WebDriver, key: &str) -> WebDriverResult<Option<String>> {
    let script = format!("return window.localStorage.getItem({key:?});");
    let ret = driver.execute(script, vec![]).await?;
    Ok(match ret.json() {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Null => None,
        other => panic!("unexpected localStorage value for {key}: {other}"),
    })
}

async fn launch_chrome() -> WebDriverResult<WebDriver> {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    caps.set_no_sandbox()?;
    caps.set_disable_gpu()?;
    caps.set_disable_dev_shm_usage()?;
    caps.add_arg("--window-size=1280,720")?;
    WebDriver::managed(caps).await
}

async fn with_browser<F, Fut>(test: F) -> WebDriverResult<()>
where
    F: FnOnce(WebDriver, String) -> Fut,
    Fut: std::future::Future<Output = WebDriverResult<()>>,
{
    let dist = find_dist();
    let harness = start_static_server(dist);
    let url = format!("http://{}/", harness.addr);
    let driver = launch_chrome().await?;
    let result = test(driver.clone(), url).await;
    let quit = driver.quit().await;
    harness.shutdown();
    result?;
    quit
}

#[tokio::test(flavor = "multi_thread")]
async fn home_heading_and_toggle_in_browser() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&url).await?;
        let heading = wait_text(&driver, "scaffold-heading", "HeSocial").await?;
        assert_eq!(heading, "HeSocial");

        let initial = wait_text(&driver, "toggle-btn", "Off").await?;
        assert_eq!(initial, "Off");

        driver.find(By::Id("toggle-btn")).await?.click().await?;
        let after = wait_text(&driver, "toggle-btn", "On").await?;
        assert_eq!(after, "On");
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn login_page_renders_copy_and_linkedin_is_disabled() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&format!("{url}login")).await?;
        wait_text(&driver, "login-heading", "歡迎回來").await?;
        let body = driver.find(By::Tag("body")).await?.text().await?;
        for needle in [
            "登入您的尊榮帳戶",
            "電子郵件",
            "密碼",
            "記住我",
            "忘記密碼？",
            "或使用",
            "Google",
            "LinkedIn (即將推出)",
            "還沒有帳戶？",
            "立即申請加入",
        ] {
            assert!(body.contains(needle), "missing {needle:?} in {body:?}");
        }
        let linkedin = wait_present(&driver, "linkedin-login").await?;
        assert!(!linkedin.is_enabled().await?, "linkedin must stay disabled");
        let submit = wait_present(&driver, "login-submit").await?;
        assert!(submit.is_enabled().await?, "submit should start enabled");
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn login_success_stores_token_and_navigates_home() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&format!("{url}login")).await?;
        wait_present(&driver, "email").await?;
        driver
            .find(By::Id("email"))
            .await?
            .send_keys("ok@example.com")
            .await?;
        driver
            .find(By::Id("password"))
            .await?
            .send_keys("secret")
            .await?;
        driver.find(By::Id("login-submit")).await?.click().await?;
        wait_text(&driver, "scaffold-heading", "HeSocial").await?;
        let token = local_storage_get(&driver, "hesocial_token").await?;
        assert_eq!(token.as_deref(), Some("e2e-login-token"));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn login_401_renders_backend_error() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&format!("{url}login")).await?;
        wait_present(&driver, "email").await?;
        driver
            .find(By::Id("email"))
            .await?
            .send_keys("unknown@example.com")
            .await?;
        driver
            .find(By::Id("password"))
            .await?
            .send_keys("wrong")
            .await?;
        driver.find(By::Id("login-submit")).await?.click().await?;
        wait_text(&driver, "login-error", "Invalid email or password").await?;
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn login_submit_disabled_while_in_flight() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&format!("{url}login")).await?;
        wait_present(&driver, "email").await?;
        driver
            .find(By::Id("email"))
            .await?
            .send_keys("slow@example.com")
            .await?;
        driver
            .find(By::Id("password"))
            .await?
            .send_keys("secret")
            .await?;
        driver.find(By::Id("login-submit")).await?.click().await?;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        let mut saw_disabled = false;
        while tokio::time::Instant::now() < deadline {
            if let Ok(btn) = driver.find(By::Id("login-submit")).await {
                if !btn.is_enabled().await.unwrap_or(true) {
                    saw_disabled = true;
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(saw_disabled, "submit was never disabled while in flight");
        wait_text(&driver, "scaffold-heading", "HeSocial").await?;
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn google_button_leaves_the_spa() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&format!("{url}login")).await?;
        wait_present(&driver, "google-login").await?;
        driver.find(By::Id("google-login")).await?.click().await?;
        wait_text(&driver, "google-oauth-stub", "Google OAuth stub").await?;
        let current = driver.current_url().await?.to_string();
        assert!(
            current.contains("/api/auth/google"),
            "expected full-page navigation to /api/auth/google, got {current}"
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn oauth_token_is_claimed_before_complete_profile_redirect() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver
            .goto(&format!(
                "{url}complete-profile?token=oauth-jwt-from-callback"
            ))
            .await?;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
        let mut last_url;
        let mut last_token;
        loop {
            last_url = driver
                .current_url()
                .await
                .map(|u| u.to_string())
                .unwrap_or_default();
            last_token = local_storage_get(&driver, "hesocial_token")
                .await
                .unwrap_or(None);
            if last_token.as_deref() == Some("oauth-jwt-from-callback")
                && last_url.contains("/profile")
                && !last_url.contains("/login")
            {
                break;
            }
            if tokio::time::Instant::now() >= deadline {
                panic!(
                    "oauth token was not claimed before routing; url={last_url:?} token={last_token:?}"
                );
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        assert!(
            !last_url.contains("/login"),
            "guard bounced to login; token vanished. url={last_url}"
        );
        assert_eq!(last_token.as_deref(), Some("oauth-jwt-from-callback"));
        wait_present(&driver, "profile-stub").await?;
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn password_toggle_reveals_text() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&format!("{url}login")).await?;
        wait_present(&driver, "password").await?;
        let before: String = driver
            .execute(
                "return document.getElementById('password').getAttribute('type');",
                vec![],
            )
            .await?
            .convert()?;
        assert_eq!(before, "password");
        driver
            .find(By::Id("password-toggle"))
            .await?
            .click()
            .await?;
        let after: String = driver
            .execute(
                "return document.getElementById('password').getAttribute('type');",
                vec![],
            )
            .await?
            .convert()?;
        assert_eq!(after, "text");
        Ok(())
    })
    .await
}
