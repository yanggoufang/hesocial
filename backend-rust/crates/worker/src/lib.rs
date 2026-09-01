#![allow(clippy::result_large_err)]
use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::HeaderName;
use axum::http::header::{
    ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN, ORIGIN, VARY,
};
use axum::http::{HeaderMap, HeaderValue, Method, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use serde_json::json;
use tower_service::Service;
use worker::send::SendFuture;
use worker::{Context, Env, HttpRequest, Result, event};

mod admin_handlers;
mod analytics_db_handlers;
mod analytics_handlers;
mod auth;
mod auth_handlers;
mod db;
mod event_handlers;
mod handlers;
mod media_handlers;
mod oauth_handlers;
mod participant_handlers;
mod registration_handlers;
mod sales_handlers;

const DEFAULT_CORS_ORIGIN: &str = "http://localhost:3000";
const EXTRA_CORS_ORIGINS: [&str; 2] = ["http://127.0.0.1:3000", "http://localhost:5000"];

const RATE_LIMITER_BINDING: &str = "RATE_LIMITER";
const RATE_LIMITER_DISABLED_ENV: &str = "AUTH_RATE_LIMIT_DISABLED";
const RATE_LIMITER_KEY_HEADER: HeaderName = HeaderName::from_static("cf-connecting-ip");
const RATE_LIMITER_UNKEYED_CLIENT: &str = "unknown";

#[derive(Clone)]
pub struct AppState {
    allowed_origins: Vec<String>,
    pub env: Env,
}

impl AppState {
    fn from_env(env: &Env) -> Self {
        let configured_origins = env
            .var("CORS_ORIGINS")
            .map(|value| value.to_string())
            .unwrap_or_else(|_| DEFAULT_CORS_ORIGIN.to_owned());
        let mut allowed_origins: Vec<String> = configured_origins
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .map(str::to_owned)
            .collect();

        for origin in EXTRA_CORS_ORIGINS {
            if !allowed_origins.iter().any(|allowed| allowed == origin) {
                allowed_origins.push(origin.to_owned());
            }
        }

        Self {
            allowed_origins,
            env: env.clone(),
        }
    }

    /// Express redirects OAuth results to `config.corsOrigins[0]`, i.e. the
    /// first configured origin (the extras below are appended, never first).
    pub fn frontend_origin(&self) -> &str {
        &self.allowed_origins[0]
    }
}

fn cors_headers(request_headers: &HeaderMap, state: &AppState) -> HeaderMap {
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );
    response_headers.insert(
        ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET,POST,PUT,DELETE,PATCH,OPTIONS"),
    );
    response_headers.insert(
        ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Content-Type,Authorization,X-Requested-With"),
    );
    response_headers.insert(VARY, HeaderValue::from_static("Origin"));

    if let Some(origin) = request_headers
        .get(ORIGIN)
        .and_then(|value| value.to_str().ok())
        .filter(|origin| {
            state
                .allowed_origins
                .iter()
                .any(|allowed| allowed == origin)
        })
        .and_then(|origin| HeaderValue::from_str(origin).ok())
    {
        response_headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    }

    response_headers
}

async fn cors_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let cors = cors_headers(request.headers(), &state);
    let mut response = if request.method() == Method::OPTIONS {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::NO_CONTENT;
        response
    } else {
        next.run(request).await
    };

    {
        let headers = response.headers_mut();
        for (key, value) in &cors {
            headers.insert(key, value.clone());
        }
    }

    response
}

fn rate_limiter_disabled(state: &AppState) -> bool {
    state
        .env
        .var(RATE_LIMITER_DISABLED_ENV)
        .is_ok_and(|value| value.to_string() == "true")
}

fn too_many_requests() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({
            "success": false,
            "error": "Too many requests, please try again later."
        })),
    )
        .into_response()
}

async fn apply_auth_rate_limit(state: AppState, request: Request<Body>, next: Next) -> Response {
    if rate_limiter_disabled(&state) {
        return next.run(request).await;
    }

    let Ok(limiter) = state.env.rate_limiter(RATE_LIMITER_BINDING) else {
        return next.run(request).await;
    };

    let client = request
        .headers()
        .get(RATE_LIMITER_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map_or_else(|| RATE_LIMITER_UNKEYED_CLIENT.to_owned(), str::to_owned);

    match limiter.limit(client).await {
        Ok(outcome) if !outcome.success => too_many_requests(),
        _ => next.run(request).await,
    }
}

async fn auth_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    SendFuture::new(apply_auth_rate_limit(state, request, next)).await
}

fn auth_routes(state: &AppState) -> Router<AppState> {
    let rate_limited = Router::new()
        .route("/register", post(auth_handlers::register))
        .route("/login", post(auth_handlers::login))
        .route(
            "/profile",
            get(auth_handlers::profile).put(auth_handlers::update_profile),
        )
        .route("/refresh", post(auth_handlers::refresh))
        .route("/logout", post(auth_handlers::logout))
        .route("/google", get(oauth_handlers::google_start))
        .route("/google/callback", get(oauth_handlers::google_callback))
        .route("/linkedin", get(oauth_handlers::linkedin_unavailable))
        .route(
            "/linkedin/callback",
            get(oauth_handlers::linkedin_unavailable),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_rate_limit_middleware,
        ));

    // /api/auth/validate stays outside the rate limiter: Express never limits
    // it, and the frontend boot path (useAuth) calls it on every page load —
    // a 429 there would log returning users out.
    Router::new()
        .route(
            "/validate",
            get(auth_handlers::validate).post(auth_handlers::validate),
        )
        .merge(rate_limited)
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(handlers::health))
        .route("/api/health/status", get(handlers::health_status))
        .route(
            "/api/events",
            get(handlers::list_events).post(event_handlers::create_event),
        )
        .route(
            "/api/events/{id}",
            get(event_handlers::get_event)
                .put(event_handlers::update_event)
                .delete(event_handlers::delete_event),
        )
        .route(
            "/api/events/{id}/publish",
            post(event_handlers::publish_event),
        )
        .route(
            "/api/events/{id}/approve",
            post(event_handlers::approve_event),
        )
        .route(
            "/api/registrations/stats/{event_id}",
            get(registration_handlers::registration_stats),
        )
        .route(
            "/api/registrations/user",
            get(registration_handlers::get_user_registrations),
        )
        .route(
            "/api/registrations/events/{event_id}",
            post(registration_handlers::register_for_event),
        )
        .route(
            "/api/registrations/{id}",
            get(registration_handlers::get_registration)
                .put(registration_handlers::update_registration)
                .delete(registration_handlers::cancel_registration),
        )
        .route(
            "/api/registrations/{id}/payment",
            post(registration_handlers::update_payment_status),
        )
        .route(
            "/api/events/{event_id}/participants",
            get(participant_handlers::list_participants),
        )
        .route(
            "/api/events/{event_id}/participant-access",
            get(participant_handlers::get_participant_access),
        )
        .route(
            "/api/events/{event_id}/participants/{participant_id}",
            get(participant_handlers::get_participant),
        )
        .route(
            "/api/events/{event_id}/participants/{participant_id}/contact",
            post(participant_handlers::initiate_contact),
        )
        .route(
            "/api/events/{event_id}/privacy-settings",
            get(participant_handlers::get_privacy_settings)
                .put(participant_handlers::update_privacy_settings),
        )
        .route("/api/categories", get(handlers::list_categories))
        .route("/api/venues", get(handlers::list_venues))
        .route(
            "/api/media/events/{event_id}/images",
            post(media_handlers::upload_event_images),
        )
        .route(
            "/api/media/events/{event_id}/documents",
            post(media_handlers::upload_event_documents),
        )
        .route(
            "/api/media/events/{event_id}",
            get(media_handlers::get_event_media),
        )
        .route(
            "/api/media/venues/{venue_id}/images",
            post(media_handlers::upload_venue_images),
        )
        .route(
            "/api/media/venues/{venue_id}",
            get(media_handlers::get_venue_media),
        )
        .route(
            "/api/media/{media_id}",
            delete(media_handlers::delete_media),
        )
        .route(
            "/api/sales/leads",
            get(sales_handlers::list_leads).post(sales_handlers::create_lead),
        )
        .route(
            "/api/sales/leads/{id}",
            get(sales_handlers::get_lead)
                .put(sales_handlers::update_lead)
                .delete(sales_handlers::delete_lead),
        )
        .route(
            "/api/sales/opportunities",
            get(sales_handlers::list_opportunities).post(sales_handlers::create_opportunity),
        )
        .route(
            "/api/sales/opportunities/{id}",
            put(sales_handlers::update_opportunity),
        )
        .route(
            "/api/sales/activities",
            get(sales_handlers::list_activities).post(sales_handlers::create_activity),
        )
        .route("/api/sales/metrics", get(sales_handlers::get_metrics))
        .route(
            "/api/sales/pipeline/stages",
            get(sales_handlers::get_pipeline_stages),
        )
        .route("/api/sales/team", get(sales_handlers::get_sales_team))
        .route(
            "/api/analytics/visitors",
            get(analytics_handlers::visitors_overview),
        )
        .route(
            "/api/analytics/visitors/daily",
            get(analytics_handlers::visitors_daily),
        )
        .route(
            "/api/analytics/visitors/{visitor_id}",
            get(analytics_handlers::visitor_detail),
        )
        .route(
            "/api/analytics/pages/popular",
            get(analytics_handlers::popular_pages),
        )
        .route(
            "/api/analytics/conversion",
            get(analytics_handlers::conversion),
        )
        .route(
            "/api/analytics/events/engagement",
            get(analytics_handlers::events_engagement),
        )
        .route(
            "/api/analytics/events/track",
            post(analytics_handlers::track_event),
        )
        .route(
            "/api/analytics/events/overview",
            get(analytics_db_handlers::events_overview),
        )
        .route(
            "/api/analytics/events/performance",
            get(analytics_db_handlers::events_performance),
        )
        .route(
            "/api/analytics/events/{id}/performance",
            get(analytics_db_handlers::event_performance_detail),
        )
        .route(
            "/api/analytics/revenue/events",
            get(analytics_db_handlers::revenue_events),
        )
        .route(
            "/api/analytics/engagement/members",
            get(analytics_db_handlers::members_engagement),
        )
        .route("/api/users", get(admin_handlers::list_users))
        .route(
            "/api/users/stats/overview",
            get(admin_handlers::user_stats_overview),
        )
        .route(
            "/api/users/{id}",
            get(admin_handlers::get_user)
                .put(admin_handlers::update_user)
                .delete(admin_handlers::delete_user),
        )
        .route("/api/users/{id}/verify", post(admin_handlers::verify_user))
        .route(
            "/api/users/{id}/role",
            post(admin_handlers::update_user_role),
        )
        .route(
            "/api/admin/database/stats",
            get(admin_handlers::database_stats),
        )
        .nest("/api/auth", auth_routes(&state))
        .fallback(handlers::fallback)
        // Express order: visitorTracking runs after CORS, before routes. The
        // CORS layer (outermost here) short-circuits OPTIONS, so preflights
        // are never tracked — same as Express's cors preflight handling.
        .layer(middleware::from_fn_with_state(
            state.clone(),
            analytics_handlers::visitor_tracking_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            cors_middleware,
        ))
        .with_state(state)
}

#[event(fetch)]
async fn fetch(
    request: HttpRequest,
    env: Env,
    _context: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router(AppState::from_env(&env)).call(request).await?)
}
