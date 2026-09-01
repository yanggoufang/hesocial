#![cfg(not(target_arch = "wasm32"))]

use std::fs::File;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use thirtyfour::prelude::*;
use tiny_http::{Header, Response, Server};

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

fn start_static_server(root: PathBuf) -> (SocketAddr, Arc<Server>) {
    let server = Server::http("127.0.0.1:0").expect("bind static server");
    let addr = server
        .server_addr()
        .to_ip()
        .expect("static server must bind an IP port");
    let server = Arc::new(server);
    let serving = server.clone();
    thread::spawn(move || {
        for request in serving.incoming_requests() {
            let raw = request.url().split('?').next().unwrap_or("/");
            let relative = raw.trim_start_matches('/');
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
    (addr, server)
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

#[tokio::test(flavor = "multi_thread")]
async fn home_heading_and_toggle_in_browser() -> WebDriverResult<()> {
    let dist = find_dist();
    let (addr, _server) = start_static_server(dist);
    let url = format!("http://{addr}/");

    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    caps.set_no_sandbox()?;
    caps.set_disable_gpu()?;
    caps.set_disable_dev_shm_usage()?;
    caps.add_arg("--window-size=1280,720")?;

    let driver = WebDriver::managed(caps).await?;
    let result: WebDriverResult<()> = async {
        driver.goto(&url).await?;
        let heading = wait_text(&driver, "scaffold-heading", "HeSocial").await?;
        assert_eq!(heading, "HeSocial");

        let initial = wait_text(&driver, "toggle-btn", "Off").await?;
        assert_eq!(initial, "Off");

        driver.find(By::Id("toggle-btn")).await?.click().await?;
        let after = wait_text(&driver, "toggle-btn", "On").await?;
        assert_eq!(after, "On");
        Ok(())
    }
    .await;
    let quit = driver.quit().await;
    result?;
    quit
}
