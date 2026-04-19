use opcua_http_server::opcua::start_opcua_server;

#[tokio::main]
async fn main() {
    let opcua_config_path = "opcua.conf".into();
    let node_config_path = "nodes.yml".into();
    start_opcua_server(opcua_config_path, node_config_path).await;
}
