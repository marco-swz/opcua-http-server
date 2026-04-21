use std::sync::Arc ;

use opcua_server::address_space::Variable;
use opcua_server::diagnostics::NamespaceMetadata;
use opcua_server::node_manager::memory::{
    InMemoryNodeManager, SimpleNodeManager, SimpleNodeManagerImpl, simple_node_manager,
};
use opcua_server::ServerBuilder;
use opcua_types::{NodeId, Variant};
use serde::{Deserialize, Serialize};

pub type NodeManager = Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>;

pub async fn start_opcua_server(opcua_config_path: String, node_config_path: String) -> (u16, NodeManager) {
    // Create an OPC UA server with sample configuration and default node set

    let (server, handle) = ServerBuilder::new()
        .with_config_from(opcua_config_path)
        //.build_info(BuildInfo {
        //    product_uri: "https://github.com/freeopcua/async-opcua".into(),
        //    manufacturer_name: "Rust OPC-UA".into(),
        //    product_name: "Rust OPC-UA sample server".into(),
        //    // Here you could use something to inject the build time, version, number at compile time
        //    software_version: "0.1.0".into(),
        //    build_number: "1".into(),
        //    build_date: DateTime::now(),
        //})
        .with_node_manager(simple_node_manager(
            // Set the namespace for the node manager. For simple node managers this decides
            // node ownership, so make sure to use a different value here than the application URI
            // in server.conf, as that is the namespace used by the diagnostic node manager.
            NamespaceMetadata {
                namespace_uri: "urn:SimpleServer".to_owned(),
                ..Default::default()
            },
            "simple",
        ))
        .trust_client_certs(true)
        .diagnostics_enabled(true)
        .build()
        .unwrap();

    let mut node_manager = handle
        .node_managers()
        .get_of_type::<SimpleNodeManager>()
        .unwrap();

    let ns = handle.get_namespace_index("urn:SimpleServer").unwrap();

    let node_config = vec![
        NodeConfig::Variable{
            node_id: "test".into(),
            node_name: "testing".into(),
            init_value: UaValue::String("".into()),
        },
    ];

    {
        build_nodes(ns, &mut node_manager, &node_config, &NodeId::objects_folder_id());
    }

    // If you don't register a ctrl-c handler, the server will close without
    // informing clients.
    let handle_c = handle.clone();
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            println!("Failed to register CTRL-C handler: {e}");
            return;
        }
        handle_c.cancel();
    });

    tokio::spawn(async move {
        // Run the server. This does not ordinarily exit so you must Ctrl+C to terminate
        server.run().await.unwrap();
    });

    return (ns, node_manager);
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UaValue {
    String(String),
    Boolean(bool),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float(f32),
    Double(f64),
}

impl Into<Variant> for UaValue {
    fn into(self) -> Variant {
        match self {
            UaValue::String(v) => Variant::String(v.into()),
            UaValue::Boolean(v) => Variant::Boolean(v),
            UaValue::Int16(v) => Variant::Int16(v),
            UaValue::Int32(v) => Variant::Int32(v),
            UaValue::Int64(v) => Variant::Int64(v),
            UaValue::UInt16(v) => Variant::UInt16(v),
            UaValue::UInt32(v) => Variant::UInt32(v),
            UaValue::UInt64(v) => Variant::UInt64(v),
            UaValue::Float(v) => Variant::Float(v),
            UaValue::Double(v) => Variant::Double(v),
        }
    }
}

impl From<Variant> for UaValue {
    fn from(value: Variant) -> Self {
        match value {
            Variant::String(v) => UaValue::String(v.into()),
            Variant::Boolean(v) => UaValue::Boolean(v),
            Variant::Int16(v) => UaValue::Int16(v),
            Variant::Int32(v) => UaValue::Int32(v),
            Variant::Int64(v) => UaValue::Int64(v),
            Variant::UInt16(v) => UaValue::UInt16(v),
            Variant::UInt32(v) => UaValue::UInt32(v),
            Variant::UInt64(v) => UaValue::UInt64(v),
            Variant::Float(v) => UaValue::Float(v),
            Variant::Double(v) => UaValue::Double(v),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub port: u8,
    pub nodes: NodeConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeConfig {
    Variable{
        node_id: String,
        node_name: String,
        init_value: UaValue,
    },
    Folder {
        node_id: String,
        node_name: String,
        children: Option<Vec<NodeConfig>>,
    }
}

/// Creates some sample variables, and some push / pull examples that update them
fn build_nodes(
    ns: u16,
    node_manager: &mut NodeManager,
    node_config: &Vec<NodeConfig>,
    parent_node_id: &NodeId,
) {
    for config in node_config {
        match config {
            NodeConfig::Folder{node_id, node_name, children} => {
                let node_id = NodeId::new(ns, node_id.clone());
                {
                    let address_space = node_manager.address_space();
                    address_space.write().add_folder(
                        &node_id,
                        node_name.clone(),
                        node_name,
                        parent_node_id,
                    );
                }
                if let Some(children) = children {
                    build_nodes(ns, node_manager, children, parent_node_id);
                }
            },
            NodeConfig::Variable{node_id, node_name, init_value} => {
                let node_id = NodeId::new(ns, node_id.clone());
                {
                    let address_space = node_manager.address_space();
                    address_space.write().add_variables(
                        vec![
                            Variable::new(
                                &node_id,
                                node_name.clone(),
                                node_name,
                                init_value.clone(),
                            ),
                        ],
                        &parent_node_id,
                    );
                }
            }
        }
    }
}
