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

fn el(name: &'static str) -> Box<dyn FnMut(&Node) -> bool + 'static> {
    Box::new(move |n| n.has_tag_name(name))
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

fn with_maxlength(prop: Node) -> bool {
    return !prop.attribute("MaxLength").is_none();
}

fn validation_str(prop: Node) -> Option<String> {
    if nullable(prop) && with_maxlength(prop) {
        return Some(
            format!(
                "\t#[validation(required,length={})]",
                prop.attribute("MaxLength").unwrap()
            )
        );
    } else if with_maxlength(prop) {
        return Some(
            format!(
                "\t#[validation(length={})]",
                prop.attribute("MaxLength").unwrap()
            )
        );
    } else if nullable(prop) {
        return Some("\t#[validation(required)]".to_string());
    } else {
        return None;
    }
}

fn convert_type(attr_type: String) -> String {
    match attr_type.as_ref() {
        "Edm.String" => return String::from("String"),
        "Edm.Boolean" => return String::from("bool"),
        "Edm.Date" => return String::from("DateTime"),
        "Edm.Number" => return String::from("u32"),
        _ => return String::from("?"),
    }
}

fn prop_str(prop: Node) -> Option<String> {
    let attr_name: Option<&str> = prop.attribute("Name");
    let attr_type: Option<&str> = prop.attribute("Type");
    if attr_name.is_none() || attr_type.is_none() {
        return None;
    } else {
        return Some(
            format!(
                "\tpub {}: {},\n",
                attr_name.unwrap(),
                convert_type(attr_type.unwrap().to_string())
            )
        );
    }
}

fn validatable_property_line(prop: Node) -> String {
    let mut line: String = String::new();
    match prop_str(prop) {
        Some(attr) => {
            match validation_str(prop) {
                Some(validation) => {
                    line.push_str(&(validation.to_string() + "\n"))
                },
                _ => { }
            }
            line.push_str(&attr);
        },
        _ => { }
    }
    return line;
}

fn property_line(prop: Node) -> String {
    match prop_str(prop) {
        Some(attr) => return String::from(attr),
        _ => return String::new(),
    }
}

fn write_struct(entity: Node) ->
        Result<Vec<String>, Box<dyn std::error::Error + 'static>> {
    let mut out: Vec<String> = Vec::new();
    out.push("#[derive(Debug, Deserialize)]".to_string());
    out.push(format!("struct {} {{", entity.attribute("Name").unwrap()));
    for prop in entity.descendants().filter(is_prop()) {
        out.push(format!("{}", property_line(prop)));
    }
    out.push("}}\n\n".to_string());
    return Ok(out);
}

fn write_create_struct(entity: Node) ->
        Result<Vec<String>, Box<dyn std::error::Error + 'static>> {
    let mut out: Vec<String> = Vec::new();
    out.push("#[derive(Debug, Validate, Deserialize)]".to_string());
    out.push(format!("struct {} {{", entity.attribute("Name").unwrap()));
    for prop in entity.descendants().filter(is_editable_prop()) {
        out.push(format!("{}", validatable_property_line(prop)));
    }
    out.push("}}\n\n".to_string());
    return Ok(out);
}

fn write_update_struct(entity: Node) ->
        Result<Vec<String>, Box<dyn std::error::Error + 'static>> {
    let mut out: Vec<String> = Vec::new();
    out.push("#[derive(Debug, Validate, Deserialize)]".to_string());
    out.push(format!("struct {} {{", entity.attribute("Name").unwrap()));
    for prop in entity.descendants().filter(is_editable_prop()) {
        out.push(format!("{}", validatable_property_line(prop)));
    }
    out.push("}}\n\n".to_string());
    return Ok(out);
}

fn write_id_impl(entity: Node) ->
        Result<Vec<String>, Box<dyn std::error::Error + 'static>> {
    let mut out: Vec<String> = Vec::new();
    let mut keys = Vec::new();
    for key in entity.descendants().filter(el("PropertyRef")) {
        keys.push(key.attribute("Name").unwrap());
    }
    debug!("KEYS {:#?}", keys);
    return Ok(out);
}

fn maybe_add(
        node: Node,
        func: &dyn Fn(Node) ->
            Result<Vec<String>, Box<dyn std::error::Error + 'static>>
) -> Vec<String> {
    match func(node) {
        Ok(output) => return output,
        _ => return Vec::<String>::new()
    }
}

pub async fn generate(name: &'static str) ->
        Result<(), Box<dyn std::error::Error + 'static>> {
    let mut out: Vec<String> = Vec::new();
    let xml: String = fs::read_to_string("odata_metadata.xml")?.parse()?;
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let entity = doc.descendants().find(by_name(name));
    out.push("use chrono::DateTime;".to_string());
    out.push("use serde::Deserialize;".to_string());
    out.push("use validator::{{Validate, ValidationError}};\n\n".to_string());
    out.append(&mut maybe_add(entity.unwrap(), &write_struct));
    out.append(&mut maybe_add(entity.unwrap(), &write_create_struct));
    out.append(&mut maybe_add(entity.unwrap(), &write_update_struct));
    out.append(&mut maybe_add(entity.unwrap(), &write_id_impl));
    debug!("{:#?}", out);
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
