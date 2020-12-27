#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_xml_rs;

use serde_xml_rs::from_reader;
use std::collections::HashMap;
use std::cell::Cell;
use std::fs::File;
use std::fs;

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
    pub length: u8,
    #[serde(default)]
    pub key: Cell<bool>
}

impl Property {
    fn make_prim(&self) {
        self.key.set(true);
    }
}

#[derive(Debug, Deserialize)]
struct OEntity {
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
    pub entities: Vec<OEntity>
}

#[derive(Debug, Deserialize)]
struct DataService {
    #[serde(alias = "Schema")]
    pub schema: Schema
}

#[derive(Debug, Deserialize)]
struct Edmx {
    #[serde(alias = "DataServices")]
    pub dataservice: DataService
}


fn one() {
    let file = File::open("odata_metadata.xml").unwrap();
    let edmx: Edmx = from_reader(file).unwrap();
    let mut entities: HashMap<String,HashMap<String,&Property>> = HashMap::new();
    for entity in edmx.dataservice.schema.entities.iter() {
        let mut props: HashMap<String,_> = HashMap::new();
        for prop in entity.props.iter() {
            props.insert(prop.name.clone(), prop);
        }
        for key in &entity.keys {
            for prop_ref in &key.refs {
                props.get_mut(&prop_ref.name).unwrap().make_prim();
                println!("{:#?}", prop_ref.name);
            }
        }
        entities.insert(entity.name.clone(), props);
    }
    println!("{:#?}", entities);
}

fn two() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let xml: String = fs::read_to_string("odata_metadata.xml")?.parse()?;
    let doc = roxmltree::Document::parse(&xml).unwrap();
    for entity in doc.descendants().filter(|n| n.has_tag_name("EntityType")) {
        println!("ENTITY {:#?}", entity.attribute("Name"));
        for key in entity.descendants().filter(|n| n.has_tag_name("PropertyRef")) {
            println!("KEY {:#?}", key.attribute("Name"));
        }
        for prop in entity.descendants().filter(|n| n.has_tag_name("Property")) {
            println!("Property {:#?}", prop.attribute("Name"));
            println!("{:#?}", prop.attribute("Type"));
            println!("{:#?}", prop.attribute("Nullable"));
            println!("{:#?}", prop.attribute("MaxLength"));
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    one();
    two().ok();
    Ok(())
}
