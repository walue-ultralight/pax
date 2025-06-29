use anyhow::Result;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use petgraph::{Graph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Reactive streaming server for real-time Pax component updates
pub struct ReactiveStreamingServer {
    /// Connected clients
    clients: Arc<DashMap<Uuid, ClientConnection>>,
    /// Component DAG
    component_dag: Arc<RwLock<ComponentDAG>>,
    /// Broadcast channel for component updates
    update_sender: broadcast::Sender<ComponentUpdate>,
    /// Hot reload compilation pipeline
    compiler: Arc<HotReloadCompiler>,
}

#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub id: Uuid,
    pub sender: tokio::sync::mpsc::UnboundedSender<Message>,
    pub subscribed_components: Vec<String>,
}

/// DAG representation of Pax components and their dependencies
#[derive(Debug)]
pub struct ComponentDAG {
    graph: Graph<ComponentNode, DependencyEdge>,
    component_map: std::collections::HashMap<String, NodeIndex>,
}

#[derive(Debug, Clone)]
pub struct ComponentNode {
    pub id: String,
    pub component_type: String,
    pub source_code: String,
    pub compiled_wasm: Option<Vec<u8>>,
    pub properties: serde_json::Value,
    pub last_updated: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct DependencyEdge {
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Parent,
    Child,
    Property,
    Event,
    Style,
}

/// Real-time component update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentUpdate {
    pub component_id: String,
    pub update_type: UpdateType,
    pub data: serde_json::Value,
    pub timestamp: u64,
    pub affected_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    SourceCodeChanged,
    PropertyChanged,
    StructureChanged,
    StyleChanged,
    EventHandlerChanged,
    CompiledWasmReady,
}

/// Hot reload compiler for real-time compilation
pub struct HotReloadCompiler {
    compilation_queue: Arc<RwLock<Vec<CompilationTask>>>,
}

#[derive(Debug, Clone)]
pub struct CompilationTask {
    pub component_id: String,
    pub source_code: String,
    pub dependencies: Vec<String>,
    pub priority: u8,
}

impl ReactiveStreamingServer {
    pub fn new() -> Self {
        let (update_sender, _) = broadcast::channel(1000);
        
        Self {
            clients: Arc::new(DashMap::new()),
            component_dag: Arc::new(RwLock::new(ComponentDAG::new())),
            update_sender,
            compiler: Arc::new(HotReloadCompiler::new()),
        }
    }

    /// Start the streaming server
    pub async fn start(&self, addr: &str) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("Reactive streaming server listening on: {}", addr);

        // Start background tasks
        self.start_compilation_worker().await;
        self.start_dag_update_worker().await;

        while let Ok((stream, addr)) = listener.accept().await {
            info!("New connection from: {}", addr);
            
            let clients = self.clients.clone();
            let component_dag = self.component_dag.clone();
            let update_sender = self.update_sender.clone();
            let compiler = self.compiler.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_connection(
                    stream, 
                    clients, 
                    component_dag, 
                    update_sender,
                    compiler
                ).await {
                    error!("Error handling connection: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Start background compilation worker
    async fn start_compilation_worker(&self) {
        let compiler = self.compiler.clone();
        let update_sender = self.update_sender.clone();
        
        tokio::spawn(async move {
            loop {
                if let Some(task) = compiler.get_next_task().await {
                    info!("Compiling component: {}", task.component_id);
                    
                    match compile_component(&task).await {
                        Ok(compiled_wasm) => {
                            let update = ComponentUpdate {
                                component_id: task.component_id.clone(),
                                update_type: UpdateType::CompiledWasmReady,
                                data: serde_json::json!({
                                    "wasm_size": compiled_wasm.len(),
                                    "compilation_time": std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis()
                                }),
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                affected_components: task.dependencies,
                            };
                            
                            if let Err(e) = update_sender.send(update) {
                                error!("Failed to send compilation update: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Compilation failed for {}: {}", task.component_id, e);
                        }
                    }
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
    }

    /// Start DAG update worker
    async fn start_dag_update_worker(&self) {
        let component_dag = self.component_dag.clone();
        let update_sender = self.update_sender.clone();
        
        tokio::spawn(async move {
            let mut update_receiver = update_sender.subscribe();
            
            while let Ok(update) = update_receiver.recv().await {
                let mut dag = component_dag.write().await;
                
                match update.update_type {
                    UpdateType::SourceCodeChanged => {
                        dag.update_component_source(&update.component_id, &update.data).await;
                    }
                    UpdateType::StructureChanged => {
                        dag.rebuild_dependencies(&update.component_id).await;
                    }
                    _ => {}
                }
            }
        });
    }

    /// Handle incoming component update from external source
    pub async fn handle_external_update(&self, update: ComponentUpdate) -> Result<()> {
        // Add to compilation queue if needed
        if matches!(update.update_type, UpdateType::SourceCodeChanged) {
            let task = CompilationTask {
                component_id: update.component_id.clone(),
                source_code: update.data["source_code"].as_str().unwrap_or("").to_string(),
                dependencies: update.affected_components.clone(),
                priority: 1,
            };
            
            self.compiler.add_task(task).await;
        }

        // Broadcast to all connected clients
        if let Err(e) = self.update_sender.send(update) {
            error!("Failed to broadcast update: {}", e);
        }

        Ok(())
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    clients: Arc<DashMap<Uuid, ClientConnection>>,
    component_dag: Arc<RwLock<ComponentDAG>>,
    update_sender: broadcast::Sender<ComponentUpdate>,
    _compiler: Arc<HotReloadCompiler>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    let client_id = Uuid::new_v4();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    
    let client = ClientConnection {
        id: client_id,
        sender: tx,
        subscribed_components: Vec::new(),
    };
    
    clients.insert(client_id, client);
    
    // Handle outgoing messages
    let clients_clone = clients.clone();
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = ws_sender.send(message).await {
                error!("Failed to send message to client {}: {}", client_id, e);
                break;
            }
        }
        clients_clone.remove(&client_id);
    });

    // Handle incoming messages
    let mut update_receiver = update_sender.subscribe();
    tokio::spawn(async move {
        while let Ok(update) = update_receiver.recv().await {
            if let Some(client) = clients.get(&client_id) {
                let message = Message::Text(serde_json::to_string(&update).unwrap());
                if let Err(e) = client.sender.send(message) {
                    error!("Failed to send update to client {}: {}", client_id, e);
                    break;
                }
            }
        }
    });

    // Handle client messages
    while let Some(msg) = ws_receiver.next().await {
        match msg? {
            Message::Text(text) => {
                info!("Received from client {}: {}", client_id, text);
                // Handle client commands (subscribe, unsubscribe, etc.)
            }
            Message::Close(_) => {
                info!("Client {} disconnected", client_id);
                break;
            }
            _ => {}
        }
    }

    clients.remove(&client_id);
    Ok(())
}

impl ComponentDAG {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            component_map: std::collections::HashMap::new(),
        }
    }

    pub async fn update_component_source(&mut self, component_id: &str, data: &serde_json::Value) {
        if let Some(&node_index) = self.component_map.get(component_id) {
            if let Some(node) = self.graph.node_weight_mut(node_index) {
                if let Some(source_code) = data["source_code"].as_str() {
                    node.source_code = source_code.to_string();
                    node.last_updated = std::time::SystemTime::now();
                }
            }
        }
    }

    pub async fn rebuild_dependencies(&mut self, _component_id: &str) {
        // Rebuild dependency graph based on component analysis
        // This would analyze imports, property bindings, etc.
    }

    pub fn get_dependent_components(&self, component_id: &str) -> Vec<String> {
        if let Some(&node_index) = self.component_map.get(component_id) {
            self.graph
                .neighbors(node_index)
                .filter_map(|idx| self.graph.node_weight(idx))
                .map(|node| node.id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl HotReloadCompiler {
    pub fn new() -> Self {
        Self {
            compilation_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_task(&self, task: CompilationTask) {
        let mut queue = self.compilation_queue.write().await;
        queue.push(task);
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub async fn get_next_task(&self) -> Option<CompilationTask> {
        let mut queue = self.compilation_queue.write().await;
        queue.pop()
    }
}

/// Compile a Pax component to WebAssembly
async fn compile_component(task: &CompilationTask) -> Result<Vec<u8>> {
    // Simulate compilation process
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // In a real implementation, this would:
    // 1. Parse the Pax source code
    // 2. Generate Rust code
    // 3. Compile to WebAssembly using wasm-pack
    // 4. Return the compiled WASM bytes
    
    Ok(vec![0x00, 0x61, 0x73, 0x6d]) // WASM magic number
}