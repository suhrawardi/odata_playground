extern crate reqwest;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

use dotenv::dotenv;
use reqwest::Client;
use roxmltree::Node;
use std::fs;
use std::env;
use std::path::Path;


fn by_name(name: &'static str) -> Box<dyn FnMut(&Node) -> bool + 'static> {
    Box::new(move |n|
             n.has_tag_name("EntityType") &&
             n.attribute("Name") == Some(name))
}

fn is_prop() -> Box<dyn FnMut(&Node) -> bool> {
    Box::new(move |n| n.has_tag_name("Property"))
}

fn not_is_editable() -> Box<dyn FnMut(&Node) -> bool> {
    Box::new(move |n|
             n.attribute("Term") == Some("NAV.AllowEdit") &&
             n.attribute("Bool") == Some("false"))
}

fn is_editable_prop() -> Box<dyn FnMut(&Node) -> bool> {
    Box::new(move |n|
             is_prop()(n) &&
             n.descendants().find(not_is_editable()) == None)
}

fn nullable(prop: Node) -> bool {
    return prop.attribute("Nullable") == Some("false");
}

fn has_maxlength(prop: Node) -> bool {
    return !prop.attribute("MaxLength").is_none();
}

fn generate_create_struct(entity: Node) ->
        Result<(), Box<dyn std::error::Error + 'static>> {
    debug!("struct {} {{", entity.attribute("Name").unwrap());
    for prop in entity.descendants().filter(is_editable_prop()) {
        debug!(
            "\tpub {}: {}",
            prop.attribute("Name").unwrap(),
            prop.attribute("Type").unwrap()
        );
    }
    debug!("}}");
    Ok(())
}

fn generate_update_struct(entity: Node) ->
        Result<(), Box<dyn std::error::Error + 'static>> {
    debug!("struct {} {{", entity.attribute("Name").unwrap());
    for prop in entity.descendants().filter(is_editable_prop()) {
        if nullable(prop) {
            debug!("\t#[validate(required)]");
        }
        if has_maxlength(prop) {
            debug!(
                "\t#[validate(length={})]",
                prop.attribute("MaxLength").unwrap()
            );
        }
        debug!(
            "\tpub {}: {}",
            prop.attribute("Name").unwrap(),
            prop.attribute("Type").unwrap()
        );
    }
    debug!("}}");
    Ok(())
}

fn generate_id_impl(entity: Node) ->
        Result<(), Box<dyn std::error::Error + 'static>> {
    let mut keys = Vec::new();
    for key in entity.descendants().filter(|n| n.has_tag_name("PropertyRef")) {
        keys.push(key.attribute("Name").unwrap());
    }
    debug!("KEYS {:#?}", keys);
    Ok(())
}

async fn generate(name: &'static str) ->
        Result<(), Box<dyn std::error::Error + 'static>> {
    let xml: String = fs::read_to_string("odata_metadata.xml")?.parse()?;
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let entity = doc.descendants().find(by_name(name));
    debug!("ENTITY: {:#?}", entity);
    generate_create_struct(entity.unwrap().clone()).ok();
    generate_update_struct(entity.unwrap().clone()).ok();
    generate_id_impl(entity.unwrap().clone()).ok();
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

async fn download_metadata() ->
        Result<(), Box<dyn std::error::Error + 'static>> {
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
    generate("Alt_Address_Card").await?;
    Ok(())
}