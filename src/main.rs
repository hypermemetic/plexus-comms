use plexus_comms::{build_comms_hub, build_from_config_file, CommsConfig};
use plexus_transport::TransportServer;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Plexus Communications Server");

    // Load configuration
    let config_path = env::var("PLEXUS_COMMS_CONFIG").unwrap_or_else(|_| "config.toml".to_string());
    let hub = if std::path::Path::new(&config_path).exists() {
        tracing::info!("Loading configuration from {}", config_path);
        build_from_config_file(&config_path).await
            .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?
    } else {
        tracing::warn!(
            "Config file {} not found, using default configuration (SMTP only)",
            config_path
        );
        build_comms_hub(CommsConfig::default()).await
            .map_err(|e| anyhow::anyhow!("Failed to build hub: {}", e))?
    };

    // Get server port from environment or use default
    let port: u16 = env::var("PLEXUS_COMMS_PORT")
        .unwrap_or_else(|_| "4445".to_string())
        .parse()
        .unwrap_or(4445);

    let activations = hub.list_activations_info();
    tracing::info!("Registered activations ({}):", activations.len());
    for activation in &activations {
        tracing::info!("  {} v{} - {}",
            activation.namespace,
            activation.version,
            activation.description
        );
    }
    tracing::info!("Server listening on ws://127.0.0.1:{}", port);

    // Configure transport server using plexus-transport library
    let rpc_converter = |arc| {
        use plexus_core::plexus::DynamicHub;
        DynamicHub::arc_into_rpc_module(arc)
            .map_err(|e| anyhow::anyhow!("Failed to create RPC module: {}", e))
    };

    // Start the transport server
    TransportServer::builder(hub, rpc_converter)
        .with_websocket(port)
        .build().await?
        .serve().await
}
