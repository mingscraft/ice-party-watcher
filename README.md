# ice-party-watcher
When we playing with pet projects, and we want to expose our home server,
We don't always has the luxury of static IP, only dynamic IP.
`ice-party-watcher` is a tiny daemon process that monitor your IP of your server,
and update your DNS A record when your server's dynamic IP has changed.

# Supported DNS Server provider
`ice-party-watcher` currently support the below DNS server provider:
- Cloud DNS (Google Cloud Platform)

