use anyhow::Result;
use pax_reactive_streaming::{ReactiveStreamingServer, ComponentUpdate, UpdateType};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Pax Reactive Streaming Server...");

    // Create and start the server
    let server = ReactiveStreamingServer::new();
    
    // Start the server in a background task
    let server_handle = {
        let server = server.clone();
        tokio::spawn(async move {
            if let Err(e) = server.start("127.0.0.1:8081").await {
                eprintln!("Server error: {}", e);
            }
        })
    };

    // Simulate external updates for demo purposes
    tokio::spawn(async move {
        sleep(Duration::from_secs(2)).await;
        
        loop {
            // Simulate code change
            let update = ComponentUpdate {
                component_id: "demo_button".to_string(),
                update_type: UpdateType::SourceCodeChanged,
                data: json!({
                    "source_code": format!(
                        "<Button text=\"Click me! ({})\"/>", 
                        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                    ),
                    "change_type": "auto_generated"
                }),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                affected_components: vec!["demo_button".to_string(), "parent_component".to_string()],
            };

            if let Err(e) = server.handle_external_update(update).await {
                eprintln!("Failed to handle external update: {}", e);
            }

            sleep(Duration::from_secs(5)).await;

            // Simulate property change
            let property_update = ComponentUpdate {
                component_id: "demo_counter".to_string(),
                update_type: UpdateType::PropertyChanged,
                data: json!({
                    "count": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() % 100,
                    "color": if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() % 2 == 0 {
                        "blue"
                    } else {
                        "red"
                    }
                }),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                affected_components: vec!["demo_counter".to_string()],
            };

            if let Err(e) = server.handle_external_update(property_update).await {
                eprintln!("Failed to handle property update: {}", e);
            }

            sleep(Duration::from_secs(3)).await;

            // Simulate structure change
            let structure_update = ComponentUpdate {
                component_id: "dynamic_layout".to_string(),
                update_type: UpdateType::StructureChanged,
                data: json!({
                    "new_children": [
                        {"type": "Rectangle", "id": "rect1"},
                        {"type": "Text", "id": "text1"},
                        {"type": "Button", "id": "btn1"}
                    ],
                    "layout_type": "vertical"
                }),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                affected_components: vec!["dynamic_layout".to_string(), "main_app".to_string()],
            };

            if let Err(e) = server.handle_external_update(structure_update).await {
                eprintln!("Failed to handle structure update: {}", e);
            }

            sleep(Duration::from_secs(7)).await;
        }
    });

    info!("Server started on ws://127.0.0.1:8081");
    info!("Simulating external updates every few seconds...");
    info!("Press Ctrl+C to stop");

    // Wait for the server
    server_handle.await?;

    Ok(())
}