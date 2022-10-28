use crate::DnsServerUpdator;
use gcp_auth::AuthenticationManager;
use reqwest::{header, Client};
use secrecy::{ExposeSecret, Secret};

pub struct CloudDns {
    token: Secret<String>,
    client: Client,
    // Google credential key ID
    key_id: Secret<String>,
    managed_zone: String,
    record_name: String,
    project_id: String,
}

impl CloudDns {
    pub async fn new(
        key_id: String,
        managed_zone: String,
        record_name: String,
    ) -> anyhow::Result<Self> {
        // Load credential from GOOGLE_APPLICATION_CREDENTIALS
        // For example: export GOOGLE_APPLICATION_CREDENTIALS=/home/user/cred.json
        let authentication_manager = AuthenticationManager::new().await?;
        let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
        let token = authentication_manager.get_token(scopes).await?;
        let project_id = authentication_manager.project_id().await?;

        let client = reqwest::Client::builder().build()?;

        Ok(Self {
            token: Secret::new(token.as_str().to_string()),
            client,
            key_id: Secret::new(key_id),
            managed_zone,
            project_id,
            record_name,
        })
    }
}

#[async_trait::async_trait]
impl DnsServerUpdator for CloudDns {
    async fn update_ip_in_dns(
        &self,
        ip: std::net::IpAddr,
    ) -> Result<(), crate::DnsServerUpdatorError> {
        let project_id = &self.project_id;
        let managed_zone = &self.managed_zone;
        let record_name = &self.record_name;
        let record_type = "A";
        let key = &self.key_id.expose_secret();
        let bearer_token = self.token.expose_secret();
        let ip = ip.to_string();

        let resp = self.client.patch(format!(
            "https://dns.googleapis.com/dns/v1/projects/{project_id}/managedZones/{managed_zone}/rrsets/{record_name}/{record_type}?key={key}",
        ))
        .header(header::AUTHORIZATION, format!("Bearer {bearer_token}"))
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .body(format!(r#"{{"rrdatas":["{ip}"]}}"#))
        .send().await.map_err(|e| crate::DnsServerUpdatorError::FailedToSend(anyhow::anyhow!(e)))?;

        if !resp.status().is_success() {
            return Err(crate::DnsServerUpdatorError::ErrorResponse(
                anyhow::anyhow!(format!(
                    "status: {}. error: {:?}",
                    resp.status(),
                    resp.text()
                        .await
                        .map_err(|e| crate::DnsServerUpdatorError::Other(anyhow::anyhow!(
                            "invalid error body: {}",
                            e
                        )))?
                )),
            ));
        }

        Ok(())
    }
}
