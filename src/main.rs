use roxmltree::Node;
use std::collections::HashMap;
use std::cell::Cell;
use std::fs::File;
use std::fs;

fn editable(elem: Option<Node>) -> bool {
    match elem {
        Some(el) => {
            match el.attribute("Bool") {
                Some("false") => { false },
                _ => { true }
            }
        },
        None => {
            true
        }
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

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    two().ok();
    Ok(())
}
