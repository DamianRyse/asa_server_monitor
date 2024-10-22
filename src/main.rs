use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use serde_yaml_ng;
use chrono::Utc;
use std::error::Error;
use std::path::Path;
use std::fs;

const REQUEST_URL: &str = "https://cdn2.arkdedicated.com/servers/asa/officialserverlist.json";

#[derive(Deserialize,Debug)]
struct ServerData {
    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "NumPlayers")]
    num_players: u8
}

#[derive(Debug, Deserialize)]
struct InfluxConfig {
    url: String,
    token: String,
    org: String,
    bucket: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    influxdb: InfluxConfig
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = Path::new("/etc/asa_server_monitor/config.yaml");
    let config_content = fs::read_to_string(config_path)?;
    let config: Config = serde_yaml_ng::from_str(&config_content)?;

    Ok(config)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = load_config()?;

    // Get the raw data from the URL
    let response = reqwest::get(REQUEST_URL).await?;

    // Parse the JSON
    let json_array: Vec<Value> = response.json().await?;


    // Loop through the json_array and get the valid data
    let game_servers: Vec<ServerData> = json_array.into_iter()
        .filter_map(|value| {
            serde_json::from_value(value).ok()
        })
    .collect();

    // Convert to line protocol data
    let line_protocol_data = build_line_protocol(&game_servers);

    // Send to InfluxDB
    send_to_influxdb(&config.influxdb, &line_protocol_data).await?;

    Ok(())
}

fn build_line_protocol(data: &[ServerData]) -> String {
    let timestamp = Utc::now().timestamp_nanos_opt().unwrap();
    let mut line_protocol_data = String::new();
    for entry in data {
        // Line data protocol format: measurement,tag=tag_value field=field_value timestamp
        line_protocol_data.push_str(&format!(
                "onlinePlayers,serverName={} playerCount={}i {}\n", entry.name.replace(" ","\\ "), entry.num_players, timestamp)
            );
    }
    line_protocol_data
}

async fn send_to_influxdb(influx_config: &InfluxConfig, data: &str) -> Result<(), Box<dyn Error>> {
    // HTTP client
    let client = Client::new();

    let response = client
        .post(&influx_config.url)
        .query(&[("org",&influx_config.org), ("bucket",&influx_config.bucket), ("precision",&"ns".to_string())])
        .header("Authorization", format!("Token {}", influx_config.token))
        .header("Content-Type","text/plain; charset=utf-8")
        .header("Accept", "application/json")
        .body(data.to_string())
        .send()
        .await?;

    if response.status().is_success() {
        println!("Data successfully written to InfluxDB.");
    } else {
        println!("Failed to write data to InfluxDB. Status: {}", response.status());
    }

    Ok(())
}

