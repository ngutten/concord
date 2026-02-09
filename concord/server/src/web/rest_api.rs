use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::engine::chat_engine::ChatEngine;
use crate::engine::events::HistoryMessage;

#[derive(Deserialize)]
pub struct HistoryParams {
    pub before: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct HistoryResponse {
    pub channel: String,
    pub messages: Vec<HistoryMessage>,
    pub has_more: bool,
}

pub async fn get_channel_history(
    State(engine): State<Arc<ChatEngine>>,
    Path(channel_name): Path<String>,
    Query(params): Query<HistoryParams>,
) -> impl IntoResponse {
    let channel = if channel_name.starts_with('#') {
        channel_name
    } else {
        format!("#{}", channel_name)
    };

    let limit = params.limit.unwrap_or(50).min(200);

    match engine
        .fetch_history(&channel, params.before.as_deref(), limit)
        .await
    {
        Ok((messages, has_more)) => Json(HistoryResponse {
            channel,
            messages,
            has_more,
        })
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn get_channels(
    State(engine): State<Arc<ChatEngine>>,
) -> impl IntoResponse {
    Json(engine.list_channels())
}
