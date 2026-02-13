use crate::activations::*;
use crate::config::CommsConfig;
use plexus_core::DynamicHub;
use std::sync::Arc;

/// Build the communications hub with all enabled activations
pub async fn build_comms_hub(config: CommsConfig) -> Result<Arc<DynamicHub>, String> {
    let mut hub = DynamicHub::new("comms");

    // Register email activation (always available, uses runtime account registration)
    let email = Email::new()
        .await
        .map_err(|e| format!("Failed to initialize email activation: {}", e))?;
    hub = hub.register(email);
    tracing::info!("Email activation registered (multi-account mode)");

    // Register Discord activation (always available, uses runtime account registration)
    let discord = Discord::new()
        .await
        .map_err(|e| format!("Failed to initialize Discord activation: {}", e))?;
    hub = hub.register(discord);
    tracing::info!("Discord activation registered (multi-account mode)");

    // Register SMS activation if configured
    if let Some(sms_config) = config.sms {
        let sms = Sms::new(sms_config)
            .await
            .map_err(|e| format!("Failed to initialize SMS activation: {}", e))?;
        hub = hub.register(sms);
        tracing::info!("SMS activation registered");
    }

    // Register push notifications if configured
    if let Some(push_config) = config.push {
        let push = Push::new(push_config)
            .await
            .map_err(|e| format!("Failed to initialize push activation: {}", e))?;
        hub = hub.register(push);
        tracing::info!("Push activation registered");
    }

    // Register Telegram if configured
    if let Some(telegram_config) = config.telegram {
        let telegram = Telegram::new(telegram_config)
            .await
            .map_err(|e| format!("Failed to initialize Telegram activation: {}", e))?;
        hub = hub.register(telegram);
        tracing::info!("Telegram activation registered");
    }

    // Register WhatsApp if configured
    if let Some(whatsapp_config) = config.whatsapp {
        let whatsapp = Whatsapp::new(whatsapp_config)
            .await
            .map_err(|e| format!("Failed to initialize WhatsApp activation: {}", e))?;
        hub = hub.register(whatsapp);
        tracing::info!("WhatsApp activation registered");
    }

    // Register Slack if configured
    if let Some(slack_config) = config.slack {
        let slack = Slack::new(slack_config)
            .await
            .map_err(|e| format!("Failed to initialize Slack activation: {}", e))?;
        hub = hub.register(slack);
        tracing::info!("Slack activation registered");
    }

    Ok(Arc::new(hub))
}

/// Build a default hub with minimal configuration (SMTP email only)
pub async fn build_default_hub() -> Result<Arc<DynamicHub>, String> {
    build_comms_hub(CommsConfig::default()).await
}

/// Load configuration from a TOML file and build the hub
pub async fn build_from_config_file(path: &str) -> Result<Arc<DynamicHub>, String> {
    let config_str = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: CommsConfig = toml::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    build_comms_hub(config).await
}
