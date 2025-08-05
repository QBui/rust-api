use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::{handlers::enterprise, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Admin-only audit trail endpoints
        .route("/audit/users/:user_id", get(enterprise::get_user_audit_trail))

        // Feature flag management (admin only)
        .route("/feature-flags", get(enterprise::list_feature_flags))
        .route("/feature-flags/:flag_name/toggle", post(enterprise::toggle_feature_flag))
        .route("/feature-flags/:flag_name/check", get(enterprise::check_feature_flag))

        // Circuit breaker demonstration
        .route("/circuit-breaker/demo", get(enterprise::circuit_breaker_demo))

        // Enhanced user profile with feature flags
        .route("/profile/enhanced", get(enterprise::get_enhanced_profile))
}
