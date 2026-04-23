use std::{str::FromStr, sync::Arc};

use crate::opcua::{Config, NodeConfig, NodeManager, UaValue};
use axum::{
    Json, Router, extract::State, http::StatusCode, response::{IntoResponse, Response}, routing::{get, post}
};
use opcua_server::{SubscriptionCache, address_space::NodeType};
use opcua_types::{DataEncoding, DataValue, NodeId, NumericRange, TimestampsToReturn};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SharedState {
    config: Config,
    node_manager: NodeManager,
    subscriptions: Arc<RwLock<SubscriptionCache>>,
}

pub async fn start_webserver(state: SharedState) {
    let state = Arc::new(state);
    let app = Router::new()
        .route("/config", get(get_config))
        .route("/nodes", get(get_nodes))
        .route("/nodes", post(post_nodes))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// #[axum::debug_handler]
async fn get_config(State(state): State<Arc<SharedState>>) -> Result<Json<NodeConfig>, HttpError> {
    Ok(Json(state.config.nodes.clone()))
}

#[derive(Debug, Serialize)]
struct GetNodeResponse {
    node_id: String,
    value: Option<UaValue>,
}

// #[axum::debug_handler]
async fn get_nodes(
    State(state): State<Arc<SharedState>>,
    Json(node_ids): Json<Vec<String>>,
) -> Result<Json<Vec<GetNodeResponse>>, HttpError> {
    let space = state.node_manager.address_space().read();
    let mut resp = Vec::new();
    for node_id_str in node_ids {
        let Ok(node_id) = NodeId::from_str(&node_id_str) else {
            continue;
        };

        let Ok(node_id) = node_id.as_variable_id() else {
            continue;
        };

        let Some(node) = space.find_node(node_id) else {
            continue;
        };

        let NodeType::Variable(variable) = node else {
            continue;
        };

        let value = variable.value(
            TimestampsToReturn::Server,
            &NumericRange::default(),
            &DataEncoding::default(),
            f64::MAX,
        );

        resp.push(GetNodeResponse {
            node_id: node_id_str,
            value: value.value.map(UaValue::from),
        });
    }
    return Ok(Json(resp));
}

#[derive(Debug, Deserialize)]
struct PostNodesRequest {
    node_id: String,
    value: Option<UaValue>,
}

#[axum::debug_handler]
async fn post_nodes(
    State(state): State<Arc<SharedState>>,
    Json(node_values): Json<Vec<PostNodesRequest>>,
) -> Result<(), HttpError> {
    let mut values: Vec<(&NodeId, Option<&NumericRange>, DataValue)> =
        Vec::with_capacity(node_values.len());
    for entry in node_values {
        let node_id = NodeId::from_str(&entry.node_id)?;
        let value = DataValue::new_now(entry.value);
        values.push((&node_id, None, value));
    }
    let subscriptions = state.subscriptions.read().await;
    state
        .node_manager
        .set_values(&subscriptions, values.into_iter());
    Ok(())
}

struct HttpError(anyhow::Error);

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}

impl<E> From<E> for HttpError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
