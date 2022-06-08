mod cloudflare;
mod config;

use cloudflare::*;
use config::Config;

use clap::{Arg, Command};
use anyhow::{Result, bail};
use std::time::Duration;
use redis::AsyncCommands;
use std::ffi::OsStr;

async fn update_record_ip(cf: &Cloudflare, zone_id: &str, record_name: &str, new_ip: &str) -> Result<()> {
    let mut records = cf.list_records(&zone_id, Some(record_name)).await?;
    if records.is_empty() {
        bail!("Failed to find record {}", record_name);
    }
    let mut record = records.remove(0);
    record.content = new_ip.to_owned();
    cf.patch_record(zone_id, &record).await?;
    Ok(())
}

async fn get_current_record_content(cf: &Cloudflare, zone_id: &str, record_name: &str) -> Result<String> {
    let mut records = cf.list_records(&zone_id, Some(record_name)).await?;
    if records.is_empty() {
        bail!("Failed to find DNS record {}", record_name);
    }
    let record = records.remove(0);
    Ok(record.content)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Command::new("aegisd")
        .about("Cloudflare Intra DynDNS")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .takes_value(true)
                .required(false)
                .help("Path to the config file"),
        )
        .get_matches();
    let config_path = args.value_of_os("config").map(OsStr::as_ref);

    let config = Config::from_file(config_path)?;
    let mut redis_url = config.redis_host;
    if !redis_url.starts_with("redis://") {
        redis_url = "redis://".to_owned() + &redis_url;
    }

    let cf = Cloudflare::new(&config.cf_token)?;
    let zone_id = cf.zone_id(&config.zone_name).await?;

    let redis_client = redis::Client::open(redis_url)?;
    let mut redis = redis_client.get_async_connection().await?;

    let mut record_content = get_current_record_content(&cf, &zone_id, &config.record_name).await?;

    println!("Running and connected to redis");
    loop {
        if let Ok(redis_value) = redis.get::<_, String>(&config.redis_key).await {
            if redis_value != record_content {
                update_record_ip(&cf, &zone_id, &config.record_name, &redis_value).await?;
                println!("Updated record {} to {}", &config.record_name, redis_value);
                record_content = redis_value;
            }
        }
        tokio::time::sleep(Duration::from_millis(config.redis_poll_interval)).await;
    }
}
