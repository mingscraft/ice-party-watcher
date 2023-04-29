use crate::DnsServerUpdator;
use aws_sdk_route53::types::{
    Change, ChangeAction, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType,
};
use aws_sdk_route53::Client;
use tracing::instrument;

pub struct Route53Dns {
    record_name: String,
    zone_id: String,
    client: Client,
    ttl: i64,
}

impl Route53Dns {
    /// Create instance of `Route53Dns`
    pub async fn new(record_name: &str, zone_id: &str, ttl: Option<i64>) -> anyhow::Result<Self> {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        let ttl = ttl.unwrap_or(600);
        Ok(Route53Dns {
            record_name: record_name.to_owned(),
            zone_id: zone_id.to_owned(),
            ttl,
            client,
        })
    }
}

#[async_trait::async_trait]
impl DnsServerUpdator for Route53Dns {
    #[instrument(skip(self))]
    async fn update_ip_in_dns(
        &self,
        ip: std::net::IpAddr,
    ) -> Result<(), crate::DnsServerUpdatorError> {
        let record = ResourceRecord::builder().value(ip.to_string()).build();

        let change = Change::builder()
            .action(ChangeAction::Upsert)
            .resource_record_set(
                ResourceRecordSet::builder()
                    .name(&format!("{name}", name = self.record_name))
                    .ttl(self.ttl)
                    .r#type(RrType::A)
                    .resource_records(record)
                    .build(),
            )
            .build();

        let change_batch = ChangeBatch::builder()
            .comment("Update IP address")
            .changes(change)
            .build();

        let resp = self
            .client
            .change_resource_record_sets()
            .hosted_zone_id(&self.zone_id)
            .change_batch(change_batch)
            .send()
            .await
            .map_err(|e| crate::DnsServerUpdatorError::FailedToSend(anyhow::anyhow!("{e:?}")))?;
        tracing::info!("Update Record response: {:?}", resp);
        Ok(())
    }
}
