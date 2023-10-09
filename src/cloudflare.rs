use std::{env, fmt, io};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ureq::Agent;

pub struct Client {
    pub agent: Agent,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("request failed: {0}")]
    Request(#[from] Box<ureq::Error>),
    #[error("JSON parsing failed: {0}")]
    JsonParse(#[from] io::Error),
    #[error("{0}")]
    Cloudflare(ErrorMessage),
}

#[derive(Deserialize)]
struct Response<T> {
    result: T,
    errors: Vec<ErrorMessage>,
    success: bool,
}

#[derive(Debug, Deserialize)]
pub struct ErrorMessage {
    pub code: u32,
    pub message: String,
}

impl fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cloudflare error {}: {}", self.code, self.message)
    }
}

#[derive(Deserialize)]
pub struct DnsRecord {
    pub content: String,
    pub r#type: String,
    pub id: String,
    pub zone_id: String,
}

#[derive(Serialize)]
pub struct InputDnsRecord {
    pub content: String,
}

impl Client {
    pub fn new() -> Self {
        Self {
            agent: Agent::new(),
        }
    }

    #[allow(clippy::result_large_err)]
    pub fn get_my_ip(&self, protocol: &str) -> Result<String, Error> {
        let response = self
            .agent
            .get(&format!("https://{protocol}.icanhazip.com/"))
            .call()
            .map_err(Box::new)?;

        Ok(response.into_string()?.trim_end().into())
    }

    #[allow(clippy::result_large_err)]
    pub fn get_dns_record(&self, zone_id: &str, record_id: &str) -> Result<DnsRecord, Error> {
        let response = self
            .agent
            .get(&format!(
                "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records/{record_id}"
            ))
            .set(
                "Authorization",
                &format!("Bearer {}", env::var("CLOUDFLARE_TOKEN").unwrap()),
            )
            .call()
            .map_err(Box::new)?
            .into_json::<Response<DnsRecord>>()?;

        if !response.success {
            return Err(Error::Cloudflare(
                response.errors.into_iter().next().unwrap(),
            ));
        }

        Ok(response.result)
    }

    #[allow(clippy::result_large_err)]
    pub fn update_dns_record(
        &self,
        zone_id: &str,
        record_id: &str,
        record: InputDnsRecord,
    ) -> Result<DnsRecord, Error> {
        let response = self
            .agent
            .patch(&format!(
                "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records/{record_id}"
            ))
            .set(
                "Authorization",
                &format!("Bearer {}", env::var("CLOUDFLARE_TOKEN").unwrap()),
            )
            .send_json(record)
            .map_err(Box::new)?
            .into_json::<Response<DnsRecord>>()?;

        if !response.success {
            return Err(Error::Cloudflare(
                response.errors.into_iter().next().unwrap(),
            ));
        }

        Ok(response.result)
    }
}
