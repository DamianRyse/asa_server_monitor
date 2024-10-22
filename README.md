# ASA Server Monitor
#### Purpose
The whole purpose of this tool is to fetch the server info from the official servers of 'ARK: Survival Ascendend', get the server name and current player count and store them into an InfluxDB server.
For simplicity, no special integration is used for InfluxDB but the web API using POST method and the line protocol.

#### Setup
Create a new file in `/etc/asa_server_monitor/config.yaml` and enter your InfluxDB v2 details.
Example:
```yaml
influxdb:
  url: http://INFLUXDB-URL:8086/api/v2/write
  token: YOUR-TOKEN-HERE
  org: YOUR-ORG
  bucket: YOUR-BUCKET
```

For security reasons, you should `chmod 0600` on the file. Copy the compiled binary to a suitable location, e.g. `/usr/bin/` and `chmod +x` it. Then add a cronjob for root to execute the file as often as you desire. Personally, I've set it to run every 5 minutes.

#### Why Rust? A simple shell script would achieve the same
...and?

