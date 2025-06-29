use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use pax_engine::api::*;
use pax_std::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Reactive client that connects to the streaming server
pub struct ReactiveClient {
    server_url: String,
    component_cache: Arc<RwLock<HashMap<String, CachedComponent>>>,
    update_handlers: Arc<RwLock<HashMap<String, Box<dyn UpdateHandler + Send + Sync>>>>,
}

#[derive(Debug, Clone)]
pub struct CachedComponent {
    pub id: String,
    pub source_code: String,
    pub compiled_wasm: Option<Vec<u8>>,
    pub properties: serde_json::Value,
    pub last_updated: u64,
}

pub trait UpdateHandler {
    fn handle_update(&self, update: &ComponentUpdate) -> Result<()>;
}

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

impl ReactiveClient {
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            component_cache: Arc::new(RwLock::new(HashMap::new())),
            update_handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connect to the streaming server and start receiving updates
    pub async fn connect(&self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.server_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let component_cache = self.component_cache.clone();
        let update_handlers = self.update_handlers.clone();

        // Handle incoming updates
        tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(update) = serde_json::from_str::<ComponentUpdate>(&text) {
                            Self::handle_component_update(
                                update,
                                component_cache.clone(),
                                update_handlers.clone(),
                            ).await;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("Connection closed by server");
                        break;
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Send subscription message
        let subscription = serde_json::json!({
            "type": "subscribe",
            "components": ["*"] // Subscribe to all components
        });
        
        ws_sender.send(Message::Text(subscription.to_string())).await?;

        Ok(())
    }

    async fn handle_component_update(
        update: ComponentUpdate,
        component_cache: Arc<RwLock<HashMap<String, CachedComponent>>>,
        update_handlers: Arc<RwLock<HashMap<String, Box<dyn UpdateHandler + Send + Sync>>>>,
    ) {
        println!("Received update for component: {}", update.component_id);

        // Update cache
        {
            let mut cache = component_cache.write().await;
            let cached_component = cache
                .entry(update.component_id.clone())
                .or_insert_with(|| CachedComponent {
                    id: update.component_id.clone(),
                    source_code: String::new(),
                    compiled_wasm: None,
                    properties: serde_json::Value::Null,
                    last_updated: 0,
                });

            match update.update_type {
                UpdateType::SourceCodeChanged => {
                    if let Some(source_code) = update.data["source_code"].as_str() {
                        cached_component.source_code = source_code.to_string();
                    }
                }
                UpdateType::CompiledWasmReady => {
                    // In a real implementation, this would load the WASM bytes
                    cached_component.compiled_wasm = Some(vec![]);
                }
                UpdateType::PropertyChanged => {
                    cached_component.properties = update.data.clone();
                }
                _ => {}
            }

            cached_component.last_updated = update.timestamp;
        }

        // Call update handlers
        {
            let handlers = update_handlers.read().await;
            for (component_id, handler) in handlers.iter() {
                if update.affected_components.contains(component_id) 
                    || component_id == &update.component_id {
                    if let Err(e) = handler.handle_update(&update) {
                        eprintln!("Update handler error for {}: {}", component_id, e);
                    }
                }
            }
        }
    }

    /// Register an update handler for a specific component
    pub async fn register_update_handler(
        &self,
        component_id: String,
        handler: Box<dyn UpdateHandler + Send + Sync>,
    ) {
        let mut handlers = self.update_handlers.write().await;
        handlers.insert(component_id, handler);
    }

    /// Get cached component data
    pub async fn get_component(&self, component_id: &str) -> Option<CachedComponent> {
        let cache = self.component_cache.read().await;
        cache.get(component_id).cloned()
    }
}

/// Pax component that integrates with reactive streaming
#[pax]
#[main]
#[file("reactive_component.pax")]
pub struct ReactiveStreamingComponent {
    pub client: Property<Option<ReactiveClient>>,
    pub connected_components: Property<Vec<String>>,
    pub last_update: Property<String>,
    pub update_count: Property<usize>,
}

impl ReactiveStreamingComponent {
    pub async fn connect_to_server(&mut self, _ctx: &NodeContext) {
        let client = ReactiveClient::new("ws://localhost:8081".to_string());
        
        // Register update handler
        let component_id = "reactive_demo".to_string();
        client.register_update_handler(
            component_id.clone(),
            Box::new(DemoUpdateHandler::new()),
        ).await;

        if let Err(e) = client.connect().await {
            eprintln!("Failed to connect to streaming server: {}", e);
            return;
        }

        self.client.set(Some(client));
        println!("Connected to reactive streaming server");
    }

    pub fn handle_update_received(&mut self, _ctx: &NodeContext, update: ComponentUpdate) {
        let current_count = self.update_count.get();
        self.update_count.set(current_count + 1);
        
        self.last_update.set(format!(
            "Component: {}, Type: {:?}, Time: {}",
            update.component_id,
            update.update_type,
            update.timestamp
        ));

        // Update connected components list
        let mut components = self.connected_components.get();
        if !components.contains(&update.component_id) {
            components.push(update.component_id);
            self.connected_components.set(components);
        }
    }
}

/// Demo update handler implementation
pub struct DemoUpdateHandler {
    component_id: String,
}

impl DemoUpdateHandler {
    pub fn new() -> Self {
        Self {
            component_id: "reactive_demo".to_string(),
        }
    }
}

impl UpdateHandler for DemoUpdateHandler {
    fn handle_update(&self, update: &ComponentUpdate) -> Result<()> {
        println!(
            "DemoUpdateHandler: Received update for {} - {:?}",
            update.component_id, update.update_type
        );

        match update.update_type {
            UpdateType::SourceCodeChanged => {
                println!("Source code changed, triggering hot reload...");
                // Trigger hot reload in the UI
            }
            UpdateType::CompiledWasmReady => {
                println!("New WASM compilation ready, updating runtime...");
                // Load new WASM module
            }
            UpdateType::PropertyChanged => {
                println!("Properties changed: {}", update.data);
                // Update component properties
            }
            _ => {}
        }

        Ok(())
    }
}

/// External API for triggering updates (simulates external data source)
pub struct ExternalUpdateSource {
    client: ReactiveClient,
}

impl ExternalUpdateSource {
    pub fn new(server_url: String) -> Self {
        Self {
            client: ReactiveClient::new(server_url),
        }
    }

    /// Simulate external data update
    pub async fn simulate_code_change(&self, component_id: &str, new_code: &str) -> Result<()> {
        // In a real implementation, this would send the update to the server
        // which would then broadcast it to all connected clients
        
        let update = ComponentUpdate {
            component_id: component_id.to_string(),
            update_type: UpdateType::SourceCodeChanged,
            data: serde_json::json!({
                "source_code": new_code,
                "change_type": "manual_edit"
            }),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            affected_components: vec![component_id.to_string()],
        };

        println!("Simulating external code change for: {}", component_id);
        // This would normally be sent via HTTP POST or WebSocket to the server
        
        Ok(())
    }

    /// Simulate property update from external system
    pub async fn simulate_property_update(
        &self,
        component_id: &str,
        properties: serde_json::Value,
    ) -> Result<()> {
        let update = ComponentUpdate {
            component_id: component_id.to_string(),
            update_type: UpdateType::PropertyChanged,
            data: properties,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            affected_components: vec![component_id.to_string()],
        };

        println!("Simulating external property update for: {}", component_id);
        
        Ok(())
    }
}