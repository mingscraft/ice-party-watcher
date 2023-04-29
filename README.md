# ice-party-watcher [dynamic IP watcher]
When we playing with pet projects, and we want to expose our home server,
We don't always has the luxury of static IP, only dynamic IP.
`ice-party-watcher` is a tiny daemon process that monitor your IP of your server,
and update your DNS A record when your server's dynamic IP has changed.

# Supported DNS Server provider
`ice-party-watcher` currently support the below DNS server provider:
- Cloud DNS (Google Cloud Platform)

# Development
Run
## Google Cloud DNS
```
RUST_LOG="ice_party_watch=info" DNS_TYPE=cloud_dns CRED_ID=<Cred ID> RECORD_NAME=<Record name> MANAGED_ZONE=<Managed zone> GOOGLE_APPLICATION_CREDENTIALS=<Credential file path> CADENCE=20 cargo run
```
## AWS Route53
```
AWS_ACCESS_KEY_ID=<AWS access key ID> AWS_SECRET_ACCESS_KEY=<AWS access key secret> ZONE_ID=<Route53 Zone ID> RUST_LOG="ice_party_watch=debug" DNS_TYPE=route53 CADENCE=20 RECORD_NAME=<DNS Record Name> TTL=600 cargo run
```
