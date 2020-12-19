#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_xml_rs;

use serde_xml_rs::from_reader;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct PropertyRef {
    #[serde(rename = "Name")]
    pub name: String
}

#[derive(Debug, Deserialize)]
struct Key {
    #[serde(alias = "PropertyRef")]
    pub refs: Vec<PropertyRef>
}

#[derive(Debug, Deserialize)]
struct Property {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type",default)]
    pub _type: String,
    #[serde(rename = "Nullable",default)]
    pub nullable: bool,
    #[serde(rename = "MaxLength",default)]
    pub length: u8
}

#[derive(Debug, Deserialize)]
struct Entity {
    #[serde(alias = "Name")]
    pub name: String,

    #[serde(alias = "Key")]
    pub keys: Vec<Key>,
    #[serde(alias = "Property")]
    pub props: Vec<Property>
}

#[derive(Debug, Deserialize)]
struct Schema {
    #[serde(alias = "EntityType")]
    pub entities: Vec<Entity>
}

#[derive(Debug, Deserialize)]
struct DataService {
    #[serde(alias = "Schema")]
    pub schema: Schema
}

#[derive(Debug, Deserialize)]
struct Edmx {
    #[serde(alias = "DataServices")]
    pub dataservices: Vec<DataService>
}

#[derive(Debug)]
struct OData {
    entities: Vec<DataService>
}

impl From<Edmx> for OData {
    fn from(e: Edmx) -> Self {
        Self {
            entities: e.dataservices
        }
    }
}

fn main() {
    let file = File::open("odata_metadata.xml").unwrap();
    let edmx: Edmx = from_reader(file).unwrap();
    let odata = OData::from(edmx);
    println!("{:#?}", odata);
}
