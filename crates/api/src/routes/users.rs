use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;

use crate::{handlers::users, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(users::list_users).post(users::create_user))
        .route("/:id", get(users::get_user).put(users::update_user).delete(users::delete_user))
        .route("/:id/profile", get(users::get_user_profile).put(users::update_user_profile))
}
