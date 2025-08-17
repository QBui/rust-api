use axum::{
    extract::{Path, Query, State},
    response::Json,
    Extension,
};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::state::AppState;
use auth::Claims;
use app_core::error::{ApiError, Result};
use app_core::enterprise::{AuditLog, FeatureFlag, PerformanceMetrics};
use monitoring::{audit_action, feature_enabled};

/// Get audit trail for a specific user (admin only)
#[instrument(skip(state))]
pub async fn get_user_audit_trail(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<AuditLog>>> {
    // Check admin permission
    if !claims.has_role("admin") {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    // Log this admin action
    let _ = audit_action!(
        state.audit_service,
        Some(claims.sub),
        "view_audit_trail",
        "user",
        Some(user_id),
        "127.0.0.1", // In real implementation, extract from request
        None,
        serde_json::json!({"target_user": user_id})
    );

    let audit_logs = state.audit_service.get_user_audit_trail(user_id, 100).await?;

    state.metrics_service.increment_counter("audit_trail_requests_total", &[
        ("requested_by", &claims.sub.to_string()),
        ("target_user", &user_id.to_string()),
    ]);

    Ok(Json(audit_logs))
}

/// Get all feature flags (admin only)
#[instrument(skip(state))]
pub async fn list_feature_flags(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<FeatureFlag>>> {
    if !claims.has_role("admin") {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let flags = state.feature_flags.list_flags().await?;

    // Log admin action
    let _ = audit_action!(
        state.audit_service,
        Some(claims.sub),
        "list_feature_flags",
        "feature_flag",
        None,
        "127.0.0.1",
        None
    );

    Ok(Json(flags))
}

/// Toggle a feature flag (admin only)
#[instrument(skip(state))]
pub async fn toggle_feature_flag(
    State(state): State<Arc<AppState>>,
    Path(flag_name): Path<String>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<FeatureFlag>> {
    if !claims.has_role("admin") {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let mut flag = state.feature_flags.get_flag(&flag_name).await?
        .ok_or_else(|| ApiError::NotFound("Feature flag not found".to_string()))?;

    flag.enabled = !flag.enabled;
    flag.updated_at = time::OffsetDateTime::now_utc();

    state.feature_flags.set_flag(flag.clone()).await?;

    // Log the toggle action
    let _ = audit_action!(
        state.audit_service,
        Some(claims.sub),
        "toggle_feature_flag",
        "feature_flag",
        None,
        "127.0.0.1",
        None,
        serde_json::json!({
            "flag_name": flag_name,
            "new_state": flag.enabled
        })
    );

    info!("Feature flag '{}' toggled to: {}", flag_name, flag.enabled);

    Ok(Json(flag))
}

/// Check if a feature is enabled for the current user
#[instrument(skip(state))]
pub async fn check_feature_flag(
    State(state): State<Arc<AppState>>,
    Path(flag_name): Path<String>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>> {
    // Create user context for feature flag evaluation
    let context = serde_json::json!({
        "user_tier": if claims.has_role("premium") { "premium" } else { "basic" },
        "user_id": claims.sub
    });

    let enabled = state.feature_flags.is_enabled(
        &flag_name,
        Some(&claims.sub.to_string()),
        Some(&context),
    ).await;

    Ok(Json(serde_json::json!({
        "flag_name": flag_name,
        "enabled": enabled,
        "user_id": claims.sub
    })))
}

/// Demonstrate circuit breaker functionality
#[instrument(skip(state))]
pub async fn circuit_breaker_demo(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>> {
    // Use circuit breaker to protect a potentially failing operation
    let result = state.circuit_breaker.call(|| {
        // Simulate a service call that might fail
        use rand::Rng;
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.3) { // 30% chance of failure
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Service unavailable"))
        } else {
            Ok("Service call successful")
        }
    }).await;

    let circuit_state = state.circuit_breaker.get_state().await;
    let failure_count = state.circuit_breaker.get_failure_count();

    match result {
        Ok(message) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": message,
            "circuit_state": format!("{:?}", circuit_state),
            "failure_count": failure_count
        }))),
        Err(_) => Ok(Json(serde_json::json!({
            "status": "failed",
            "message": "Circuit breaker prevented call or service failed",
            "circuit_state": format!("{:?}", circuit_state),
            "failure_count": failure_count
        })))
    }
}

/// Enhanced user profile endpoint with feature flag integration
#[instrument(skip(state))]
pub async fn get_enhanced_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>> {
    let user_repo = state.db_pool.user_repository();
    let user = user_repo.find_by_id(claims.sub).await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Check if beta features are enabled for this user
    let beta_enabled = feature_enabled!(
        state.feature_flags,
        "beta_features",
        &claims.sub.to_string(),
        &serde_json::json!({"user_tier": "premium"})
    );

    let analytics_enabled = feature_enabled!(
        state.feature_flags,
        "advanced_analytics",
        &claims.sub.to_string()
    );

    // Log profile access
    let _ = audit_action!(
        state.audit_service,
        Some(claims.sub),
        "view_enhanced_profile",
        "user",
        Some(user.id),
        "127.0.0.1",
        None,
        serde_json::json!({
            "beta_features": beta_enabled,
            "analytics": analytics_enabled
        })
    );

    let mut response = serde_json::json!({
        "id": user.id,
        "username": user.username,
        "email": user.email,
        "created_at": user.created_at,
        "features": {
            "beta_features": beta_enabled,
            "advanced_analytics": analytics_enabled
        }
    });

    // Add beta features if enabled
    if beta_enabled {
        response["beta_data"] = serde_json::json!({
            "new_dashboard": true,
            "experimental_features": ["ai_recommendations", "advanced_search"]
        });
    }

    // Add analytics if enabled
    if analytics_enabled {
        response["analytics"] = serde_json::json!({
            "login_count": 42,
            "last_active": user.updated_at,
            "engagement_score": 85.5
        });
    }

    Ok(Json(response))
}
