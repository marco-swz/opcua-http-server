use std::sync::Arc;

use axum::{
    Json, Router, extract::State, routing::get
};
use opcua_server::address_space::NodeType;
use opcua_types::{DataEncoding, NodeId, NumericRange, TimestampsToReturn};
use serde::Serialize;
use crate::opcua::{Config, NodeConfig, NodeManager, UaValue};

#[derive(Clone)]
struct SharedState {
    config: Config,
    node_manager: NodeManager
}

pub async fn start_webserver(state: SharedState) {
    let state = Arc::new(state);
    let app = Router::new()
        .route("/config", get(get_config))
        .route("/nodes", get(get_nodes))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn get_config(
    State(state): State<Arc<SharedState>>,
) -> Json<NodeConfig> {
    Json(state.config.nodes.clone())
}

#[derive(Debug, Serialize)]
struct GetNodeEntry {
    node_id: String,
    value: UaValue,
}

#[axum::debug_handler]
async fn get_nodes(
    State(state): State<Arc<SharedState>>,
    Json(node_ids): Json<Vec<String>>,
) -> Json<Vec<GetNodeEntry>> {
    let space = state.node_manager.address_space().read();
    let resp = Vec::new();
    for node_id in node_ids {
        let Some(node) = space.find_node(NodeId::from(node_id)) else {
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

        resp.push(GetNodeEntry{
            node_id: node_id,
            value: UaValue::String("".into())
        });

    }
    return Json(resp);
}
