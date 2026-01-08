//! Endpoint root.

use api_framework::{
    framework::{
        StateError, StateResult,
        queued_async::{QueuedAsyncFramework, QueuedAsyncFrameworkContext},
    },
    static_lazy_lock,
};

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::{
    config::{
        Config as _,
        services::{ServiceConfig, ServicesConfig},
    },
    env::DOCKER_WORKSPACE_DIR,
    transactions,
};

static_lazy_lock! {
    QUEUED_ASYNC: QueuedAsyncFramework<String> = QueuedAsyncFramework::new();
}

/// The payload of the post.
#[derive(Debug, Clone, Deserialize)]
pub struct PostPayload {
    /// The label of the service to update.
    pub service_label: String,
}

/// The client posted an update request.
/// Responds with [`StatusCode::OK`] right after the deployment is triggered.
///
/// See: [`PostPayload`], [`post_transaction`]
pub async fn post(Json(payload): Json<PostPayload>) -> impl IntoResponse {
    let services = match ServicesConfig::read() {
        Some(s) => s,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to read services config",
            )
                .into_response();
        }
    };

    let service = match services
        .services
        .iter()
        .find(|s| s.service_label == payload.service_label)
        .cloned()
    {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                format!("service with label '{}' not found", payload.service_label),
            )
                .into_response();
        }
    };

    match QUEUED_ASYNC
        .run(payload.service_label.clone(), move |cx| {
            Box::pin(post_transaction(cx, payload.clone(), service.clone()))
        })
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => {
            tracing::error!("failed to queue update transaction");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to queue update transaction",
            )
                .into_response()
        }
    }
}

async fn post_transaction(
    cx: QueuedAsyncFrameworkContext,
    _payload: PostPayload,
    service: ServiceConfig,
) -> StateResult<()> {
    async fn inner(cx: &QueuedAsyncFrameworkContext, service: &ServiceConfig) -> StateResult<()> {
        // cd to workspace
        transactions::sys::cd(&DOCKER_WORKSPACE_DIR)
            .await
            .map_err(|_| StateError::Retry)?;

        // login to docker
        transactions::docker::login()
            .await
            .map_err(|_| StateError::Retry)?;

        cx.check(())?;

        // pull image
        transactions::docker::pull_image(&service.image)
            .await
            .map_err(|_| StateError::Retry)?;

        cx.check(())?;

        // up container
        transactions::docker::compose_up(&service.container_name)
            .await
            .map_err(|_| StateError::Retry)?;

        // logout from docker
        transactions::docker::logout().await.ok();

        Ok(())
    }

    match inner(&cx, &service).await {
        Ok(_) => {
            tracing::info!("successfully updated service {}", service.identifier());
            Ok(())
        }
        err => {
            tracing::error!("failed to update service {}", service.identifier());
            err
        }
    }
}
