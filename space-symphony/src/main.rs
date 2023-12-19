pub mod gui;

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::RandomState, HashSet};

use axum::{routing::get, Json, Router};

use gui::{AuthState, SpaceConfig, SpaceSymphony};
use iced::{Application, Settings};

const APP_ID: &str = "space.symphony";

#[derive(Clone, Deserialize, Serialize)]
struct DurationSecs(i64);

impl From<Duration> for DurationSecs {
    fn from(duration: Duration) -> Self {
        DurationSecs(duration.num_seconds())
    }
}

impl From<DurationSecs> for Duration {
    fn from(val: DurationSecs) -> Self {
        Duration::seconds(val.0)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Response {
    access_token: String,
    expires_in: DurationSecs,
    expires_at: Option<DateTime<Utc>>,
    refresh_token: Option<String>,
    scopes: HashSet<String, RandomState>,
    logged: AuthState,
}

async fn setup_server() {
    let app = Router::new().route("/success", get(success));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8088")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn run_gui() -> iced::Result {
    SpaceSymphony::run(Settings {
        default_text_size: 15.0,
        window: iced::window::Settings {
            size: (1280, 800),
            ..Default::default()
        },
        ..Default::default()
    })
}

#[tokio::main]
async fn main() {
    let axum_server = tokio::spawn(setup_server());

    if let Err(e) = run_gui().await {
        eprintln!("Iced application error: {}", e);
    }

    if let Err(e) = axum_server.await {
        eprintln!("Axum server error: {}", e);
    }
}

async fn success() -> Result<Json<Response>, (StatusCode, String)> {
    match confy::load::<SpaceConfig>(APP_ID, None) {
        Ok(config) => {
            let resp: Response = Response {
                access_token: config.response.as_ref().unwrap().access_token.clone(),
                expires_in: config.response.as_ref().unwrap().expires_in.clone(),
                expires_at: config.response.as_ref().unwrap().expires_at,
                refresh_token: config.response.as_ref().unwrap().refresh_token.clone(),
                scopes: config.response.as_ref().unwrap().scopes.clone(),
                logged: AuthState::Yes,
            };

            Ok(Json(resp))
        }
        Err(e) => {
            // Log the error and return an appropriate response
            eprintln!("Error loading config: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error loading config".into(),
            ))
        }
    }
}
