extern crate reqwest;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

use dotenv::dotenv;
use reqwest::Client;
use roxmltree::Node;
use std::fs;
// use std::fs::File;
// use std::io;
use std::env;
// use tokio::runtime::Runtime;
use std::path::Path;



fn editable(elem: Option<Node>) -> bool {
    match elem {
        Some(el) => { el.attribute("Bool") != Some("false") },
        None => { true }
    }
}

fn two() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let xml: String = fs::read_to_string("odata_metadata.xml")?.parse()?;
    let doc = roxmltree::Document::parse(&xml).unwrap();
    for entity in doc.descendants()
        .filter(|n| n.has_tag_name("EntityType")) {
        println!("struct {} {{", entity.attribute("Name").unwrap());
        for prop in entity.descendants()
            .filter(|n| n.has_tag_name("Property")) {
            let elem =  prop.descendants()
                .find(|n| n.attribute("Term") == Some("NAV.AllowEdit"));
            if editable(elem) {
                for key in entity.descendants()
                    .filter(|n| n.has_tag_name("PropertyRef")) {
                    // println!("KEY {:#?}", key.attribute("Name"));
                }
                println!(
                    "\t#[validate(length = {})]",
                    prop.attribute("MaxLength").unwrap()
                );
                println!(
                    "\tpub {}: {}",
                    prop.attribute("Name").unwrap(),
                    prop.attribute("Type").unwrap()
                );
                // println!("{:#?}", prop.attribute("Type"));
                // println!("{:#?}", prop.attribute("Nullable"));
                // println!("{:#?}", prop.attribute("MaxLength"));
            }
        }
        println!("}}");
    }
    Ok(())
}

fn get_env(key: &str) -> String {
    return env::var(key)
        .unwrap_or_else(|_| "unknown".into())
        .parse()
        .expect("Can't parse .env variable");
}

fn get_odata_url() -> String {
    let host = get_env("ODATA_HOST");
    let mut protocol = "https://".to_string();
    protocol.push_str(&host);
    protocol.push_str("/nl_acceptatie/ODataV4/$metadata/");
    return protocol;
}

async fn download_metadata() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let user = get_env("NAV_USER");
    let password = get_env("NAV_PASSWORD");
    let url = get_odata_url();
    debug!("Fetching metadata from {}", url);
    let resp = Client::new()
        .get(&url)
        .basic_auth(user, Some(password))
        .send().await?.text().await?;
    fs::write("odata_metadata.xml", resp).expect("Unable to write file");
    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    dotenv().ok();
    pretty_env_logger::init();
    if !Path::new("odata_metadata.xml").exists() {
        download_metadata().await?;
        debug!("Downloaded the metadata");
    }
    // two().ok();
    Ok(())
}
