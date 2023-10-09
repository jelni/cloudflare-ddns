#![warn(clippy::pedantic, clippy::nursery)]

use std::env;

use cloudflare::{Client, InputDnsRecord};
use ureq::ErrorKind;

mod cloudflare;

fn main() {
    dotenvy::dotenv().ok();

    let client = Client::new();

    let zone_id = env::var("ZONE_ID").unwrap();

    for record_id in env::var("RECORD_IDS").unwrap().split(',') {
        let record = client.get_dns_record(&zone_id, record_id).unwrap();

        let protocol = match record.r#type.as_str() {
            "A" => "ipv4",
            "AAAA" => "ipv6",
            _ => panic!("unexpected {} record type", record.r#type),
        };

        let ip = match client.get_my_ip(protocol) {
            Err(cloudflare::Error::Request(err))
                if matches!(err.kind(), ErrorKind::ConnectionFailed) =>
            {
                println!("{protocol} address not available");
                continue;
            }
            response => response.unwrap(),
        };

        if record.content == ip {
            println!(
                "record {} already has the address {}",
                record.id, record.content
            );
            continue;
        }

        client
            .update_dns_record(
                &record.zone_id,
                &record.id,
                InputDnsRecord {
                    content: ip.clone(),
                },
            )
            .unwrap();

        println!(
            "updated record {} address from {} to {ip}",
            record.id, record.content
        );
    }
}
