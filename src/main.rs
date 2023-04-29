use ice_party_watch::cloud_dns::CloudDns;
use ice_party_watch::public_ip_resolver::PublicIpResolver;
use ice_party_watch::route53_dns::Route53Dns;
use ice_party_watch::IcePartyWatcher;
use std::env;
use std::time::Duration;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

enum DnsType {
    CloudDns,
    Route53,
}

impl TryFrom<&str> for DnsType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "cloud_dns" => Ok(DnsType::CloudDns),
            "route53" => Ok(DnsType::Route53),
            _ => Err("Unsupported dns type. Supported type: cloud_dns".into()),
        }
    }
}

#[tokio::main]
async fn main() {
    let formatting_layer = BunyanFormattingLayer::new("ice-party-watcher".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(JsonStorageLayer)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
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

    let cadence = match env::var("CADENCE") {
        Ok(val) => {
            let val = u64::from_str_radix(&val, 10)?;
            Some(Duration::from_secs(val))
        }
        Err(_) => None,
    };

    let record_name = match env::var("RECORD_NAME") {
        Ok(val) => val,
        Err(_) => {
            return Err(anyhow::anyhow!(
                "RECORD_NAME not set. RECORD_NAME=<Managed Zone>"
            ));
        }
    };

    let ip_fetcher = PublicIpResolver::new();

    match dns_type {
        DnsType::CloudDns => {
            let key_id = match env::var("CRED_ID") {
                Ok(val) => val,
                Err(_) => {
                    return Err(anyhow::anyhow!("CRED_ID not set. CRED_ID=<Credential ID>"));
                }
            };

            let managed_zone = match env::var("MANAGED_ZONE") {
                Ok(val) => val,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "MANAGED_ZONE not set. MANAGED_ZONE=<Managed Zone>"
                    ));
                }
            };

            match env::var("GOOGLE_APPLICATION_CREDENTIALS") {
                Ok(_) => {}
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "GOOGLE_APPLICATION_CREDENTIALS not set. GOOGLE_APPLICATION_CREDENTIALS=<file path of the credential>"
                    ));
                }
            };

            let dns = CloudDns::new(key_id.into(), managed_zone.into(), record_name.into()).await?;

            let mut watcher = IcePartyWatcher::new(ip_fetcher, dns, cadence);

            watcher.run().await?;
        }
        DnsType::Route53 => {
            let zone_id = match env::var("ZONE_ID") {
                Ok(val) => val,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "ZONE_ID not set. ZONE_ID=<Route53 Zone ID>"
                    ));
                }
            };

            let ttl = match env::var("TTL") {
                Ok(val) => Some(
                    val.parse::<i64>()
                        .expect("TTL should be number of sections in number."),
                ),
                Err(_) => None,
            };

            let dns = Route53Dns::new(&record_name, &zone_id, ttl).await?;

            let mut watcher = IcePartyWatcher::new(ip_fetcher, dns, cadence);

            watcher.run().await?;
        }
    };

    Ok(())
}
