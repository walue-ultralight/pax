# Pax Reactive Streaming Architecture

Bu proje, Pax Designer için real-time streaming ve reactive güncellemeler mimarisinin bir örneğidir. WebSocket tabanlı bir sistem kullanarak, sunucu kaynaklı component güncellemelerinin real-time olarak derlenip görüntülenmesini sağlar.

## Mimari Özellikleri

### 🔄 Real-time Component Streaming
- WebSocket tabanlı bidirectional communication
- Component source code değişikliklerinin anlık iletimi
- Hot reload compilation pipeline
- DAG (Directed Acyclic Graph) tabanlı dependency tracking

### 🎯 Reactive UI Updates
- Property değişikliklerinin reactive propagation'ı
- Component structure değişikliklerinin real-time reflection'ı
- Event handler güncellemelerinin hot swapping'i
- Style değişikliklerinin anlık uygulanması

### 🏗️ Component DAG System
- Component dependencies'in graph representation'ı
- Affected components'in otomatik hesaplanması
- Circular dependency detection
- Optimized update propagation

### ⚡ Hot Reload Compilation
- Background compilation queue
- Priority-based task scheduling
- WebAssembly hot loading
- Incremental compilation support

## Sistem Bileşenleri

### 1. ReactiveStreamingServer
```rust
// WebSocket server that manages real-time updates
let server = ReactiveStreamingServer::new();
server.start("127.0.0.1:8081").await?;
```

**Özellikler:**
- Client connection management
- Component DAG maintenance
- Hot reload compilation pipeline
- Update broadcasting

### 2. ReactiveClient
```rust
// Client-side reactive component integration
let client = ReactiveClient::new("ws://localhost:8081".to_string());
client.connect().await?;
```

**Özellikler:**
- WebSocket connection handling
- Component cache management
- Update handler registration
- Real-time UI synchronization

### 3. ComponentDAG
```rust
// DAG representation of component dependencies
pub struct ComponentDAG {
    graph: Graph<ComponentNode, DependencyEdge>,
    component_map: HashMap<String, NodeIndex>,
}
```

**Özellikler:**
- Dependency tracking
- Affected component calculation
- Graph-based update propagation
- Circular dependency prevention

### 4. HotReloadCompiler
```rust
// Background compilation system
pub struct HotReloadCompiler {
    compilation_queue: Arc<RwLock<Vec<CompilationTask>>>,
}
```

**Özellikler:**
- Priority-based compilation queue
- Incremental compilation
- WebAssembly generation
- Error handling and recovery

## Update Types

### SourceCodeChanged
```json
{
  "component_id": "demo_button",
  "update_type": "SourceCodeChanged",
  "data": {
    "source_code": "<Button text=\"Updated!\"/>",
    "change_type": "manual_edit"
  }
}
```

### PropertyChanged
```json
{
  "component_id": "demo_counter",
  "update_type": "PropertyChanged",
  "data": {
    "count": 42,
    "color": "blue"
  }
}
```

### StructureChanged
```json
{
  "component_id": "dynamic_layout",
  "update_type": "StructureChanged",
  "data": {
    "new_children": [
      {"type": "Rectangle", "id": "rect1"},
      {"type": "Text", "id": "text1"}
    ]
  }
}
```

### CompiledWasmReady
```json
{
  "component_id": "demo_component",
  "update_type": "CompiledWasmReady",
  "data": {
    "wasm_size": 1024,
    "compilation_time": 1500
  }
}
```

## Kullanım Senaryoları

### 1. External Data Source Integration
```rust
let external_source = ExternalUpdateSource::new("ws://localhost:8081".to_string());

// Simulate external code change
external_source.simulate_code_change(
    "user_profile",
    "<UserProfile name={user.name} avatar={user.avatar}/>"
).await?;

// Simulate property update from API
external_source.simulate_property_update(
    "live_chart",
    json!({"data": new_chart_data, "timestamp": now})
).await?;
```

### 2. Real-time Collaborative Editing
```rust
// Multiple designers working on the same project
client.register_update_handler(
    "shared_component".to_string(),
    Box::new(CollaborativeEditHandler::new())
).await;
```

### 3. Live Data Visualization
```rust
// Real-time data streaming to Pax components
let data_stream = DataStreamHandler::new();
data_stream.connect_to_api("wss://api.example.com/live-data").await;
data_stream.pipe_to_component("live_dashboard").await;
```

### 4. A/B Testing Integration
```rust
// Dynamic component switching based on experiments
let ab_test_handler = ABTestHandler::new();
ab_test_handler.register_variant("button_color", "blue").await;
ab_test_handler.register_variant("button_color", "red").await;
```

## Çalıştırma

### 1. Server'ı Başlatın
```bash
cd pax-reactive-streaming-example
cargo run --bin server
```

### 2. Pax Uygulamasını Çalıştırın
```bash
# Pax CLI ile reactive component'i çalıştırın
./pax run --target=web --port 8080
```

### 3. Browser'da Test Edin
- http://localhost:8080 - Pax reactive component
- WebSocket connection: ws://localhost:8081

## Gelişmiş Özellikler

### 1. Dependency Analysis
```rust
impl ComponentDAG {
    pub fn analyze_dependencies(&self, source_code: &str) -> Vec<String> {
        // Parse imports, property bindings, event handlers
        // Return list of dependent components
    }
}
```

### 2. Conflict Resolution
```rust
impl ReactiveStreamingServer {
    pub async fn resolve_conflicts(&self, updates: Vec<ComponentUpdate>) -> ComponentUpdate {
        // Merge conflicting updates
        // Apply conflict resolution strategies
    }
}
```

### 3. Performance Optimization
```rust
impl HotReloadCompiler {
    pub async fn compile_incremental(&self, diff: &ComponentDiff) -> Result<Vec<u8>> {
        // Only recompile changed parts
        // Use cached compilation artifacts
    }
}
```

### 4. State Synchronization
```rust
impl ReactiveClient {
    pub async fn sync_state(&self, component_id: &str) -> Result<()> {
        // Synchronize component state across clients
        // Handle state conflicts and merging
    }
}
```

## Entegrasyon Noktaları

### 1. Pax Designer Integration
- Designer UI'dan real-time code editing
- Visual property manipulation
- Component tree restructuring
- Style editor integration

### 2. External APIs
- REST API endpoints for component updates
- GraphQL subscriptions for real-time data
- Database change streams
- File system watchers

### 3. Development Tools
- VS Code extension integration
- Hot reload development server
- Debug console integration
- Performance monitoring

### 4. Production Deployment
- Load balancing for WebSocket connections
- Redis for distributed state management
- Kubernetes scaling
- Monitoring and alerting

## Güvenlik Considerations

### 1. Authentication & Authorization
```rust
impl ReactiveStreamingServer {
    pub async fn authenticate_client(&self, token: &str) -> Result<ClientPermissions> {
        // JWT token validation
        // Role-based access control
    }
}
```

### 2. Input Validation
```rust
impl ComponentUpdate {
    pub fn validate(&self) -> Result<()> {
        // Sanitize source code
        // Validate component structure
        // Check for malicious content
    }
}
```

### 3. Rate Limiting
```rust
impl ReactiveStreamingServer {
    pub async fn check_rate_limit(&self, client_id: &Uuid) -> bool {
        // Implement rate limiting per client
        // Prevent spam and abuse
    }
}
```

Bu mimari, Pax Designer'da modern web development ihtiyaçlarını karşılayan, scalable ve performant bir real-time streaming sistemi sağlar.