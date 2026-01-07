//! The API endpoints.

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;

use crate::middleware;

pub mod health;
pub mod root;

/// Routes an [`Router`] with the endpoints defined by this module.
pub fn route_from(mut app: Router) -> Router {
    app = route_gets(app);
    app = route_posts(app);
    app.layer(TraceLayer::new_for_http())
}

fn route_gets(app: Router) -> Router {
    app.route("/health", get(health::get))
}

fn route_posts(app: Router) -> Router {
    app.route(
        "/",
        post(root::post)
            .route_layer(middleware::auth::layers::KESSOKU_PRIVATE_CI_AUTHORIZATION.to_owned()),
    )
}
