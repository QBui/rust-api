use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use app_core::enterprise::CircuitBreakerConfig;
use app_core::error::Result;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing, reject all requests
    HalfOpen,  // Testing if service recovered
}

#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<AtomicU32>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    half_open_calls: Arc<AtomicU32>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            half_open_calls: Arc::new(AtomicU32::new(0)),
        }
    }

    pub async fn call<F, T, E>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> std::result::Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Check if circuit is open
        if self.is_open().await {
            return Err(anyhow::anyhow!("Circuit breaker is open").into());
        }

        // Execute the operation
        match operation() {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(anyhow::anyhow!("Operation failed: {}", e).into())
            }
        }
    }

    async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.config.recovery_timeout {
                        drop(state);
                        self.transition_to_half_open().await;
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited calls in half-open state
                self.half_open_calls.load(Ordering::Relaxed) >= self.config.half_open_max_calls
            }
            CircuitState::Closed => false,
        }
    }

    async fn on_success(&self) {
        let current_state = *self.state.read().await;

        match current_state {
            CircuitState::HalfOpen => {
                // Success in half-open state, transition to closed
                self.transition_to_closed().await;
                info!("Circuit breaker transitioned to CLOSED after successful recovery");
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::Open => {
                // Should not happen, but reset if it does
                warn!("Unexpected success in OPEN state");
            }
        }
    }

    async fn on_failure(&self) {
        let current_state = *self.state.read().await;
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

        error!("Circuit breaker recorded failure #{}", failures);

        match current_state {
            CircuitState::Closed => {
                if failures >= self.config.failure_threshold {
                    drop(current_state);
                    self.transition_to_open().await;
                    error!("Circuit breaker transitioned to OPEN after {} failures", failures);
                }
            }
            CircuitState::HalfOpen => {
                // Failure in half-open state, transition back to open
                drop(current_state);
                self.transition_to_open().await;
                warn!("Circuit breaker transitioned back to OPEN from HALF_OPEN");
            }
            CircuitState::Open => {
                // Already open, update last failure time
                *self.last_failure_time.write().await = Some(Instant::now());
            }
        }
    }

    async fn transition_to_open(&self) {
        *self.state.write().await = CircuitState::Open;
        *self.last_failure_time.write().await = Some(Instant::now());
        self.half_open_calls.store(0, Ordering::Relaxed);
    }

    async fn transition_to_half_open(&self) {
        *self.state.write().await = CircuitState::HalfOpen;
        self.half_open_calls.store(0, Ordering::Relaxed);
        info!("Circuit breaker transitioned to HALF_OPEN for recovery testing");
    }

    async fn transition_to_closed(&self) {
        *self.state.write().await = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        *self.last_failure_time.write().await = None;
        self.half_open_calls.store(0, Ordering::Relaxed);
    }

    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }

    pub fn get_failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Relaxed)
    }
}
