use crate::PublicIpFetcher;

pub struct PublicIpResolver {}

impl PublicIpResolver {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl PublicIpFetcher for PublicIpResolver {
    async fn current_ip(&self) -> Result<std::net::IpAddr, crate::PublicIpFetcherError> {
        if let Some(ip) = public_ip::addr().await {
            Ok(ip)
        } else {
            Err(crate::PublicIpFetcherError::NotAbleToFetch(
                "Couldn't get the public IP address".into(),
            ))
        }
    }
}
