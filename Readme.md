Farmview is a service to monitor a farm of servers, and presents the results in a WebUI.

It uses a TOML configuration file:

```toml
refresh_delay = 30

# WebUI configuration
[http]
    port = 8080

# Locations help group servers by their IPs
[[locations]]
    name = "Home"
    ips = "192.168." # A simple prefix match

[[locations]]
    name = "Datacenter"
    ips = "8.8."

[[hosts]]
    name = "Server 1"
    address = "server1.mydomain.com"
    iface = "eth0"

[[hosts]]
    name = "Server 2"
    address = "server2.mydomain.com"
    iface = "eth0"
    location = "Backup center" # Location can be overriden

[[hosts]]
    name = "'Home' Server"
    address = "192.168.0.15"
    iface = "eno1"
```
