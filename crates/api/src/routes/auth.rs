use axum::{
    routing::post,
    Router,
};
use std::sync::Arc;

use crate::{handlers::auth, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/refresh", post(auth::refresh_token))
}
