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

                let (status, payload) = if email == "admin@example.com" && password == "secret" {
                    (
                        200,
                        r#"{"success":true,"data":{"token":"e2e-admin-token","user":{"id":"9","email":"admin@example.com","role":"admin"}}}"#,
                    )
                } else if email == "ok@example.com" && password == "secret"
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

            if path_only == "/api/auth/validate" && *request.method() == Method::Get {
                let token = request_bearer(&request);
                let (status, payload) = stub_validate_payload(token.as_deref());
                let header = Header::from_bytes(b"Content-Type", b"application/json")
                    .expect("json content-type");
                let _ = request.respond(
                    Response::from_string(payload)
                        .with_status_code(StatusCode::from(status))
                        .with_header(header),
                );
                continue;
            }

            if path_only == "/api/auth/profile" && *request.method() == Method::Get {
                let token = request_bearer(&request);
                let (status, payload) = stub_profile_payload(token.as_deref());
                let header = Header::from_bytes(b"Content-Type", b"application/json")
                    .expect("json content-type");
                let _ = request.respond(
                    Response::from_string(payload)
                        .with_status_code(StatusCode::from(status))
                        .with_header(header),
                );
                continue;
            }

            if path_only == "/api/events" && *request.method() == Method::Get {
                let query = request.url().split_once('?').map(|(_, q)| q).unwrap_or("");
                let (status, payload) = stub_events_payload(query);
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

fn request_bearer(request: &tiny_http::Request) -> Option<String> {
    request.headers().iter().find_map(|header| {
        if header.field.equiv("Authorization") {
            header
                .value
                .as_str()
                .strip_prefix("Bearer ")
                .map(str::to_string)
        } else {
            None
        }
    })
}

fn stub_user_json(token: Option<&str>) -> Option<&'static str> {
    match token {
        Some("e2e-admin-token") => Some(
            r#"{"id":"9","email":"admin@example.com","firstName":"Admin","lastName":"User","role":"admin","membershipTier":"Black Card","isVerified":true,"verificationStatus":"approved","age":40,"profession":"System Administrator","annualIncome":5000000,"netWorth":30000000,"privacyLevel":5,"bio":"Admin","interests":["ops"],"profilePicture":null}"#,
        ),
        Some("e2e-login-token") | Some("oauth-jwt-from-callback") => Some(
            r#"{"id":"1","email":"ok@example.com","firstName":"Ok","lastName":"User","role":"user","membershipTier":"Platinum","isVerified":true,"verificationStatus":"approved","age":null,"profession":null,"annualIncome":null,"netWorth":null,"privacyLevel":3,"bio":null,"interests":null,"profilePicture":null}"#,
        ),
        _ => None,
    }
}

fn stub_validate_payload(token: Option<&str>) -> (u16, String) {
    match stub_user_json(token) {
        Some(user) => (
            200,
            format!(r#"{{"success":true,"data":{{"user":{user},"valid":true}}}}"#),
        ),
        None => (
            401,
            r#"{"success":false,"error":"Access token required"}"#.to_string(),
        ),
    }
}

fn stub_profile_payload(token: Option<&str>) -> (u16, String) {
    match stub_user_json(token) {
        Some(user) => (200, format!(r#"{{"success":true,"data":{{"user":{user}}}}}"#)),
        None => (
            401,
            r#"{"success":false,"error":"Access token required"}"#.to_string(),
        ),
    }
}

fn query_param(query: &str, key: &str) -> String {
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next().unwrap_or("");
        let v = parts.next().unwrap_or("");
        if k == key {
            return v.replace('+', " ");
        }
    }
    String::new()
}

fn stub_events_payload(query: &str) -> (u16, String) {
    let search = query_param(query, "search");
    let page = query_param(query, "page");
    if search.contains("FORCE_FAIL") {
        return (500, r#"{"success":false}"#.to_string());
    }

    let truffle = r#"{
        "id": 11,
        "name": "松露季私宴",
        "description": "白松露當季，主廚八道式無菜單。",
        "dateTime": "2026-10-04T12:00:00.000Z",
        "venue": {"name": "Taipei Private Dining Room", "address": "Da'an", "rating": 5},
        "exclusivityLevel": null,
        "pricing": {"vip": 15000, "vvip": 15000, "currency": "TWD"},
        "currentAttendees": 0,
        "capacity": 12,
        "images": ["https://media.example/e11.webp"]
    }"#;
    let yacht = r#"{
        "id": 2,
        "name": "Autumn Yacht Social",
        "description": "Sunset cruise around Keelung Harbor.",
        "dateTime": "2026-10-10T12:00:00.000Z",
        "venue": {"name": "Keelung Luxury Yacht", "address": "Pier 8", "rating": 4},
        "exclusivityLevel": null,
        "pricing": {"vip": 18000, "vvip": 18000, "currency": "TWD"},
        "currentAttendees": 1,
        "capacity": 30,
        "images": ["https://media.example/e17.webp"]
    }"#;
    let sunrise = r#"{
        "id": 16,
        "name": "日出遊艇早餐",
        "description": "清晨出海，海上日出佐香檳早餐。",
        "dateTime": "2026-10-09T12:00:00.000Z",
        "venue": {"name": "Keelung Luxury Yacht", "address": "Pier 8", "rating": 4},
        "exclusivityLevel": null,
        "pricing": {"vip": 16000, "vvip": 16000, "currency": "TWD"},
        "currentAttendees": 0,
        "capacity": 24,
        "images": ["https://media.example/e16.webp"]
    }"#;

    let (data, page_num, total, total_pages) = if !search.is_empty() {
        if search.contains("Yacht") {
            (yacht.to_string(), 1, 1, 1)
        } else {
            (String::new(), 1, 0, 1)
        }
    } else if page == "2" {
        (sunrise.to_string(), 2, 12, 2)
    } else {
        (format!("{truffle},{yacht}"), 1, 12, 2)
    };

    let payload = format!(
        r#"{{"success":true,"data":[{data}],"pagination":{{"page":{page_num},"limit":9,"total":{total},"totalPages":{total_pages}}}}}"#
    );
    (200, payload)
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

#[tokio::test(flavor = "multi_thread")]
async fn events_list_renders_cards_from_stubbed_api() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&format!("{url}events")).await?;
        wait_text(&driver, "events-heading", "精選活動").await?;
        wait_present(&driver, "event-card-11").await?;
        wait_present(&driver, "event-card-2").await?;
        wait_text(&driver, "events-page-label", "第 1 / 2 頁").await?;
        let body = driver.find(By::Tag("body")).await?.text().await?;
        for needle in [
            "松露季私宴",
            "Taipei Private Dining Room",
            "Autumn Yacht Social",
            "Keelung Luxury Yacht",
            "NT$ 15,000",
            "查看詳情",
            "第 1 / 2 頁",
        ] {
            assert!(body.contains(needle), "missing {needle:?} in {body:?}");
        }
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn events_search_filters_the_list() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&format!("{url}events")).await?;
        wait_present(&driver, "event-card-11").await?;
        let search = wait_present(&driver, "events-search").await?;
        search.send_keys("Yacht").await?;
        wait_present(&driver, "event-card-2").await?;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        loop {
            let missing = driver.find(By::Id("event-card-11")).await.is_err();
            let page = if let Ok(el) = driver.find(By::Id("events-page-label")).await {
                el.text().await.unwrap_or_default()
            } else {
                String::new()
            };
            if missing && page.contains("第 1 / 1 頁") {
                break;
            }
            if tokio::time::Instant::now() >= deadline {
                panic!("search did not filter to Yacht-only page; page={page:?}");
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        let body = driver.find(By::Tag("body")).await?.text().await?;
        assert!(body.contains("Autumn Yacht Social"));
        assert!(!body.contains("松露季私宴"));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn events_pagination_moves_to_next_page() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&format!("{url}events")).await?;
        wait_present(&driver, "event-card-11").await?;
        wait_text(&driver, "events-page-label", "第 1 / 2 頁").await?;
        driver.find(By::Id("events-next")).await?.click().await?;
        wait_present(&driver, "event-card-16").await?;
        wait_text(&driver, "events-page-label", "第 2 / 2 頁").await?;
        let body = driver.find(By::Tag("body")).await?.text().await?;
        assert!(body.contains("日出遊艇早餐"));
        assert!(!body.contains("松露季私宴"));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn events_api_failure_shows_empty_state_not_a_crash() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&format!("{url}events")).await?;
        wait_present(&driver, "event-card-11").await?;
        driver
            .execute(
                "const el = document.getElementById('events-search'); el.value = 'FORCE_FAIL'; el.dispatchEvent(new Event('input', { bubbles: true }));",
                vec![],
            )
            .await?;
        wait_present(&driver, "events-empty").await?;
        wait_text(&driver, "events-heading", "精選活動").await?;
        let body = driver.find(By::Tag("body")).await?.text().await?;
        assert!(body.contains("找不到符合條件的活動。"));
        assert!(body.contains("請嘗試調整您的篩選條件，或稍後再試。"));
        assert!(!body.contains("松露季私宴"));
        Ok(())
    })
    .await
}

async fn wait_gone(driver: &WebDriver, id: &str) -> WebDriverResult<()> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        if driver.find(By::Id(id)).await.is_err() {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("timed out waiting for #{id} to unmount");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn element_class(driver: &WebDriver, id: &str) -> WebDriverResult<String> {
    let el = wait_present(driver, id).await?;
    Ok(el.attr("class").await?.unwrap_or_default())
}

async fn login_as(driver: &WebDriver, url: &str, email: &str, password: &str) -> WebDriverResult<()> {
    driver.goto(&format!("{url}login")).await?;
    wait_present(driver, "email").await?;
    driver.find(By::Id("email")).await?.send_keys(email).await?;
    driver
        .find(By::Id("password"))
        .await?
        .send_keys(password)
        .await?;
    driver.find(By::Id("login-submit")).await?.click().await?;
    wait_present(driver, "nav-user-button").await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn signed_out_shell_renders_nav_and_footer() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&url).await?;
        wait_present(&driver, "nav").await?;
        wait_present(&driver, "footer").await?;
        wait_present(&driver, "nav-login").await?;
        wait_present(&driver, "nav-register").await?;
        let body = driver.find(By::Tag("body")).await?.text().await?;
        for needle in [
            "首頁",
            "精選活動",
            "VVIP專區",
            "登入",
            "註冊",
            "專為高淨值人士打造的頂級社交平台，提供獨家尊榮體驗與精緻社交活動。",
            "私人晚宴",
            "concierge@hesocial.com",
            "系統正常運行",
        ] {
            assert!(body.contains(needle), "missing {needle:?} in {body:?}");
        }
        assert!(
            driver.find(By::Id("nav-user-button")).await.is_err(),
            "avatar must not render signed out"
        );
        let home = wait_present(&driver, "nav-item-home").await?;
        let class = home.attr("class").await?.unwrap_or_default();
        assert!(
            class.contains("text-luxury-gold"),
            "home route must highlight 首頁, class={class}"
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn user_dropdown_opens_and_closes_with_exit_class() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        login_as(&driver, &url, "ok@example.com", "secret").await?;
        wait_present(&driver, "nav-user-button").await?;
        assert!(
            driver.find(By::Id("nav-login")).await.is_err(),
            "登入 must not render signed in"
        );
        driver.find(By::Id("nav-user-button")).await?.click().await?;
        wait_present(&driver, "nav-user-menu").await?;
        let open_class = element_class(&driver, "nav-user-menu").await?;
        assert!(
            open_class.contains("hs-dropdown-enter"),
            "open dropdown must use enter class, class={open_class}"
        );
        wait_present(&driver, "nav-profile").await?;
        wait_present(&driver, "nav-registrations").await?;
        wait_present(&driver, "nav-logout").await?;
        assert!(
            driver.find(By::Id("nav-admin")).await.is_err(),
            "plain user must not see admin entries"
        );

        driver.find(By::Id("nav-user-button")).await?.click().await?;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        let mut saw_exit = false;
        loop {
            match driver.find(By::Id("nav-user-menu")).await {
                Ok(el) => {
                    let class = el.attr("class").await?.unwrap_or_default();
                    if class.contains("hs-dropdown-exit") {
                        saw_exit = true;
                    }
                }
                Err(_) => break,
            }
            if tokio::time::Instant::now() >= deadline {
                panic!("dropdown did not unmount after close; saw_exit={saw_exit}");
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(
            saw_exit,
            "exit class never appeared; CSS presence did not keep the node mounted while animating out"
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn mobile_toggle_opens_and_closes_panel() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.set_window_rect(0, 0, 375, 812).await?;
        driver.goto(&url).await?;
        wait_present(&driver, "nav-mobile-toggle").await?;
        assert!(
            driver.find(By::Id("nav-mobile-panel")).await.is_err(),
            "mobile panel starts closed"
        );
        driver
            .find(By::Id("nav-mobile-toggle"))
            .await?
            .click()
            .await?;
        wait_present(&driver, "nav-mobile-panel").await?;
        let class = element_class(&driver, "nav-mobile-panel").await?;
        assert!(
            class.contains("hs-mobile-enter"),
            "open mobile panel must use enter class, class={class}"
        );
        tokio::time::sleep(Duration::from_millis(400)).await;
        let panel = driver.find(By::Id("nav-mobile-panel")).await?.text().await?;
        for needle in ["首頁", "精選活動", "VVIP專區", "登入", "註冊"] {
            assert!(panel.contains(needle), "missing {needle:?} in {panel:?}");
        }

        driver
            .find(By::Id("nav-mobile-toggle"))
            .await?
            .click()
            .await?;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        let mut saw_exit = false;
        loop {
            match driver.find(By::Id("nav-mobile-panel")).await {
                Ok(el) => {
                    let class = el.attr("class").await?.unwrap_or_default();
                    if class.contains("hs-mobile-exit") {
                        saw_exit = true;
                    }
                }
                Err(_) => break,
            }
            if tokio::time::Instant::now() >= deadline {
                panic!("mobile panel did not unmount; saw_exit={saw_exit}");
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(saw_exit, "mobile exit class never appeared");
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_dropdown_shows_admin_entries_only_for_admin() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        login_as(&driver, &url, "admin@example.com", "secret").await?;
        driver.find(By::Id("nav-user-button")).await?.click().await?;
        wait_present(&driver, "nav-admin").await?;
        wait_present(&driver, "nav-event-mgmt").await?;
        wait_present(&driver, "nav-sales").await?;
        wait_present(&driver, "nav-system").await?;
        let menu = driver.find(By::Id("nav-user-menu")).await?.text().await?;
        for needle in ["管理後台", "活動管理", "銷售管理", "系統健康"] {
            assert!(menu.contains(needle), "missing {needle:?} in {menu:?}");
        }
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn logout_clears_token_and_returns_signed_out_shell() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        login_as(&driver, &url, "ok@example.com", "secret").await?;
        let token = local_storage_get(&driver, "hesocial_token").await?;
        assert_eq!(token.as_deref(), Some("e2e-login-token"));
        driver.find(By::Id("nav-user-button")).await?.click().await?;
        wait_present(&driver, "nav-logout").await?;
        driver.find(By::Id("nav-logout")).await?.click().await?;
        wait_present(&driver, "nav-login").await?;
        wait_present(&driver, "nav-register").await?;
        wait_gone(&driver, "nav-user-button").await?;
        let token = local_storage_get(&driver, "hesocial_token").await?;
        assert_eq!(token, None, "logout must clear hesocial_token");
        Ok(())
    })
    .await
}

async fn seed_token(driver: &WebDriver, url: &str, token: &str) -> WebDriverResult<()> {
    driver.goto(url).await?;
    wait_present(driver, "nav").await?;
    let script = format!("window.localStorage.setItem('hesocial_token', {token:?});");
    driver.execute(script, vec![]).await?;
    driver.goto(url).await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn hard_reload_with_admin_token_restores_user_and_admin_entry() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        seed_token(&driver, &url, "e2e-admin-token").await?;
        wait_present(&driver, "nav-user-button").await?;
        driver.find(By::Id("nav-user-button")).await?.click().await?;
        wait_present(&driver, "nav-admin").await?;
        wait_present(&driver, "nav-event-mgmt").await?;
        let menu = driver.find(By::Id("nav-user-menu")).await?.text().await?;
        assert!(menu.contains("管理後台"), "restored admin must reveal admin entry, menu={menu:?}");
        let token = local_storage_get(&driver, "hesocial_token").await?;
        assert_eq!(token.as_deref(), Some("e2e-admin-token"));
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn hard_reload_with_invalid_token_clears_session_and_signed_out_shell() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        seed_token(&driver, &url, "expired-token").await?;
        wait_present(&driver, "nav-login").await?;
        wait_present(&driver, "nav-register").await?;
        wait_gone(&driver, "nav-user-button").await?;
        let token = local_storage_get(&driver, "hesocial_token").await?;
        assert_eq!(token, None, "401 validate must clear hesocial_token");
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn signed_out_profile_redirects_to_login() -> WebDriverResult<()> {
    with_browser(|driver, url| async move {
        driver.goto(&format!("{url}profile")).await?;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
        let last = loop {
            let current = driver
                .current_url()
                .await
                .map(|u| u.to_string())
                .unwrap_or_default();
            if current.contains("/login") {
                break current;
            }
            if tokio::time::Instant::now() >= deadline {
                panic!("signed-out /profile did not redirect to /login; url={current}");
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        };
        wait_text(&driver, "login-heading", "歡迎回來").await?;
        assert!(
            last.contains("/login"),
            "must land on login, url={last}"
        );
        Ok(())
    })
    .await
}
