use crate::DnsServerUpdator;

pub struct CloudDns {}

impl CloudDns {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl DnsServerUpdator for CloudDns {
    async fn update_ip_in_dns(
        &self,
        ip: std::net::IpAddr,
    ) -> Result<(), crate::DnsServerUpdatorError> {
        todo!()
    }
}
