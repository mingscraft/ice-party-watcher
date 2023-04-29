/// # ice-party-watcher
/// When we playing with pet projects, and we want to expose our home server,
/// We don't always has the luxury of static IP, only dynamic IP.
/// `ice-party-watcher` is a tiny daemon process that monitor your IP of your server,
/// and update your DNS A record when your server's dynamic IP has changed.
///
/// # Supported DNS Server provider
/// `ice-party-watcher` currently support the below DNS server provider:
/// - Cloud DNS (Google Cloud Platform)
use std::{net::IpAddr, time::Duration};
use thiserror::Error;
use tokio::time;
use tracing::instrument;

pub mod cloud_dns;
pub mod public_ip_resolver;
pub mod route53_dns;

#[derive(Error, Debug)]
pub enum DnsServerUpdatorError {
    #[error(transparent)]
    FailedToSend(anyhow::Error),
    #[error(transparent)]
    ErrorResponse(anyhow::Error),
    #[error(transparent)]
    Other(anyhow::Error),
}

/// Trait that provide utilities for dynamic IP updating.
#[async_trait::async_trait]
pub trait DnsServerUpdator {
    /// Update IP of A record in DNS
    async fn update_ip_in_dns(&self, ip: IpAddr) -> Result<(), DnsServerUpdatorError>;
}

#[derive(Error, Debug)]
pub enum PublicIpFetcherError {
    #[error("{0}")]
    NotAbleToFetch(String),
}

/// Trait to allow fetching current public IP.
#[async_trait::async_trait]
pub trait PublicIpFetcher {
    /// Fetch current IP
    async fn current_ip(&self) -> Result<IpAddr, PublicIpFetcherError>;
}

#[derive(Error, Debug)]
pub enum IcePartyWatcherError {
    #[error("{0}")]
    FailToFetchIp(PublicIpFetcherError),
    #[error("{0}")]
    FailToUpdateDns(DnsServerUpdatorError),
}

/// Struct had allow the functionalities to keep your hostname A record up to date.
pub struct IcePartyWatcher<IF, DS>
where
    IF: PublicIpFetcher + Send + Sync,
    DS: DnsServerUpdator + Send + Sync,
{
    /// The IP we believe the remove dns server hold. When `IcePartyWatcher` initially start,
    /// it would be None. We set the IP when we done the first update.
    ip_in_dns: Option<IpAddr>,
    /// Fetch current public IP
    ip_fetcher: IF,
    /// DNS server that hold the A record for our hostname
    dns_server: DS,
    /// How often we run our check and update
    cadence: Duration,
}

impl<IF, DS> IcePartyWatcher<IF, DS>
where
    IF: PublicIpFetcher + Send + Sync,
    DS: DnsServerUpdator + Send + Sync,
{
    /// Create a new instance of IcePartyWatch
    pub fn new(ip_fetcher: IF, dns_server: DS, cadence: Option<Duration>) -> Self {
        Self {
            ip_fetcher,
            dns_server,
            ip_in_dns: None,
            // Default is 10 mins, if not defined.
            cadence: cadence.unwrap_or(Duration::from_secs(600)),
        }
    }

    /// Run Watcher to keep ip up to date in DNS server
    #[instrument(skip(self))]
    pub async fn run(&mut self) -> Result<(), IcePartyWatcherError> {
        let mut interval = time::interval(self.cadence);
        loop {
            interval.tick().await;
            self.sync_ip().await?;
        }
    }

    /// Sync IP with DNS Server
    #[instrument(skip(self))]
    async fn sync_ip(&mut self) -> Result<(), IcePartyWatcherError> {
        let current_ip = self
            .ip_fetcher
            .current_ip()
            .await
            .map_err(|e| IcePartyWatcherError::FailToFetchIp(e))?;
        tracing::info!("current ip is: {}", current_ip);

        if self.should_update(current_ip) {
            tracing::info!("updating recored: no reconciliation record or ip record outdated");
            self.dns_server
                .update_ip_in_dns(current_ip)
                .await
                .map_err(IcePartyWatcherError::FailToUpdateDns)?;
            self.ip_in_dns = Some(current_ip);
        } else {
            tracing::info!("record up to date");
        }
        tracing::info!("reconciliation finish");
        Ok(())
    }

    /// Check update is needed
    #[instrument(skip(self))]
    fn should_update(&self, new_ip: IpAddr) -> bool {
        if let Some(ip_in_dns) = self.ip_in_dns {
            new_ip != ip_in_dns
        } else {
            true
        }
    }
}
