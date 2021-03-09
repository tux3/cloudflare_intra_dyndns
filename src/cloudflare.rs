use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct BasicResponse {
    success: bool,
}

#[derive(Deserialize)]
struct ZonesResponse {
    success: bool,
    result: Option<Vec<Zone>>,
}

#[derive(Deserialize, Debug)]
struct Zone {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct DnsRecordsResponse {
    success: bool,
    result: Option<Vec<DnsRecord>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub content: String,
    pub proxied: bool,
    pub ttl: isize,
}

pub struct Cloudflare {
    client: reqwest::Client,
}

impl Cloudflare {
    pub fn new(token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        let auth_bearer = "Bearer ".to_string() + token;
        let mut auth_value = HeaderValue::from_str(&auth_bearer)?;
        auth_value.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_value);
        let client = reqwest::Client::builder().default_headers(headers).build()?;

        Ok(Self { client })
    }

    pub async fn zone_id(&self, zone_name: &str) -> Result<String> {
        let response: ZonesResponse = self.client.get(&format!("https://api.cloudflare.com/client/v4/zones?name={}", zone_name))
            .send()
            .await?
            .json()
            .await?;
        if !response.success {
            bail!("Failed to list zones")
        }
        let mut zones = response.result.unwrap();
        if zones.is_empty() {
            bail!("Failed to find zone {}", zone_name);
        }
        let zone = zones.remove(0);
        assert_eq!(zone.name, zone_name);
        Ok(zone.id)
    }

    pub async fn list_records(&self, zone_id: &str, name: Option<&str>) -> Result<Vec<DnsRecord>> {
        let mut url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id);
        if let Some(name) = name {
            url = url + "?name=" + name;
        }
        let response: DnsRecordsResponse = self.client.get(&url)
            .send()
            .await?
            .json()
            .await?;

        if !response.success {
            bail!("Failed to list DNS records")
        }
        Ok(response.result.unwrap())
    }

    pub async fn patch_record(&self, zone_id: &str, record: &DnsRecord) -> Result<()> {
        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", zone_id, &record.id);
        let response: BasicResponse = self.client.patch(&url)
            .json(record)
            .send()
            .await?
            .json()
            .await?;
        if !response.success {
            bail!("Failed to patch DNS record '{}'", record.name)
        }

        Ok(())
    }
}
