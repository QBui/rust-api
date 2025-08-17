use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

use app_core::{enterprise::FeatureFlag, error::Result};

#[async_trait]
pub trait FeatureFlagService: Send + Sync {
    async fn is_enabled(&self, flag_name: &str, user_id: Option<&str>, context: Option<&Value>) -> bool;
    async fn get_flag(&self, flag_name: &str) -> Result<Option<FeatureFlag>>;
    async fn set_flag(&self, flag: FeatureFlag) -> Result<()>;
    async fn delete_flag(&self, flag_name: &str) -> Result<bool>;
    async fn list_flags(&self) -> Result<Vec<FeatureFlag>>;
}

#[derive(Clone)]
pub struct InMemoryFeatureFlagService {
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
}

impl InMemoryFeatureFlagService {
    pub fn new() -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn initialize_default_flags(&self) -> Result<()> {
        let default_flags = vec![
            FeatureFlag {
                name: "user_registration".to_string(),
                enabled: true,
                rollout_percentage: 100.0,
                conditions: None,
                created_at: time::OffsetDateTime::now_utc(),
                updated_at: time::OffsetDateTime::now_utc(),
            },
            FeatureFlag {
                name: "beta_features".to_string(),
                enabled: false,
                rollout_percentage: 10.0,
                conditions: Some(serde_json::json!({
                    "user_tier": ["premium", "enterprise"]
                })),
                created_at: time::OffsetDateTime::now_utc(),
                updated_at: time::OffsetDateTime::now_utc(),
            },
            FeatureFlag {
                name: "advanced_analytics".to_string(),
                enabled: true,
                rollout_percentage: 50.0,
                conditions: None,
                created_at: time::OffsetDateTime::now_utc(),
                updated_at: time::OffsetDateTime::now_utc(),
            },
        ];

        let mut flags = self.flags.write().await;
        for flag in default_flags {
            flags.insert(flag.name.clone(), flag);
        }

        info!("Initialized {} default feature flags", flags.len());
        Ok(())
    }

    fn evaluate_conditions(&self, flag: &FeatureFlag, context: Option<&Value>) -> bool {
        if let (Some(conditions), Some(ctx)) = (&flag.conditions, context) {
            // Simple condition evaluation - in production, use a more sophisticated rules engine
            if let Some(user_tier_conditions) = conditions.get("user_tier") {
                if let Some(user_tier) = ctx.get("user_tier") {
                    if let Some(tier_array) = user_tier_conditions.as_array() {
                        return tier_array.contains(user_tier);
                    }
                }
                return false;
            }
        }
        true
    }

    fn check_rollout(&self, flag: &FeatureFlag, user_id: Option<&str>) -> bool {
        if flag.rollout_percentage >= 100.0 {
            return true;
        }

        if let Some(uid) = user_id {
            // Consistent hashing based on user ID for stable rollout
            let hash = self.hash_user_id(uid);
            let percentage = (hash % 100) as f32;
            percentage < flag.rollout_percentage
        } else {
            // Random rollout for anonymous users
            rand::random::<f32>() * 100.0 < flag.rollout_percentage
        }
    }

    fn hash_user_id(&self, user_id: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        hasher.finish() as u32
    }
}

#[async_trait]
impl FeatureFlagService for InMemoryFeatureFlagService {
    #[instrument(skip(self, context))]
    async fn is_enabled(&self, flag_name: &str, user_id: Option<&str>, context: Option<&Value>) -> bool {
        let flags = self.flags.read().await;

        if let Some(flag) = flags.get(flag_name) {
            if !flag.enabled {
                return false;
            }

            // Check conditions first
            if !self.evaluate_conditions(flag, context) {
                return false;
            }

            // Check rollout percentage
            self.check_rollout(flag, user_id)
        } else {
            false // Flag doesn't exist, default to disabled
        }
    }

    #[instrument(skip(self))]
    async fn get_flag(&self, flag_name: &str) -> Result<Option<FeatureFlag>> {
        let flags = self.flags.read().await;
        Ok(flags.get(flag_name).cloned())
    }

    #[instrument(skip(self))]
    async fn set_flag(&self, flag: FeatureFlag) -> Result<()> {
        let mut flags = self.flags.write().await;
        flags.insert(flag.name.clone(), flag);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete_flag(&self, flag_name: &str) -> Result<bool> {
        let mut flags = self.flags.write().await;
        Ok(flags.remove(flag_name).is_some())
    }

    #[instrument(skip(self))]
    async fn list_flags(&self) -> Result<Vec<FeatureFlag>> {
        let flags = self.flags.read().await;
        Ok(flags.values().cloned().collect())
    }
}

/// Feature flag evaluation macro for easy usage
#[macro_export]
macro_rules! feature_enabled {
    ($service:expr, $flag:expr) => {
        $service.is_enabled($flag, None, None).await
    };

    ($service:expr, $flag:expr, $user_id:expr) => {
        $service.is_enabled($flag, Some($user_id), None).await
    };

    ($service:expr, $flag:expr, $user_id:expr, $context:expr) => {
        $service.is_enabled($flag, Some($user_id), Some($context)).await
    };
}
