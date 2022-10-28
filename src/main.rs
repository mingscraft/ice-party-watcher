use ice_party_watch::cloud_dns::CloudDns;
use ice_party_watch::public_ip_resolver::PublicIpResolver;
use ice_party_watch::IcePartyWatcher;
use std::env;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

enum DnsType {
    CloudDns,
}

impl TryFrom<&str> for DnsType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "cloud_dns" => Ok(DnsType::CloudDns),
            _ => Err("Unsupported dns type. Supported type: cloud_dns".into()),
        }
    }
}

#[tokio::main]
async fn main() {
    let formatting_layer = BunyanFormattingLayer::new("tracing_demo".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(JsonStorageLayer)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
    match start().await {
        Ok(_) => {}
        Err(err) => {
            tracing::error!("{:?}", err);
            std::process::exit(1);
        }
    }
}

async fn start() -> anyhow::Result<()> {
    let dns_type = match env::var("DNS_TYPE") {
        Ok(val) => val,
        Err(_) => {
            return Err(anyhow::anyhow!("DNS_TYPE not set. DNS_TYPE=[cloud_dns]"));
        }
    };

    let dns_type: DnsType = dns_type
        .as_str()
        .try_into()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let dns = match dns_type {
        DnsType::CloudDns => CloudDns::new(),
    };

    let ip_fetcher = PublicIpResolver::new();

    let watcher = IcePartyWatcher::new(ip_fetcher, dns, None);

    watcher.run().await?;

    Ok(())
}
