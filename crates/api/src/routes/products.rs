use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;

use crate::{handlers::products, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(products::list_products).post(products::create_product))
        .route("/:id", get(products::get_product).put(products::update_product).delete(products::delete_product))
}
