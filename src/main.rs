#![allow(non_snake_case)]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

use clap::App;

use serde_json::{Value};
use std::fs::File;
use std::io::prelude::*;
use std::io;


fn main() { 
    App::new("trello-set-list")
       .version("0.1")
       .about("Creates a printable set list out of a Trello board")
       .author("Brandon H.")
       .get_matches();

    // TODO: Get json from Trello

    // Read JSON
    let json_path = "D:/Git/trello-set-list/exported.json";
    let mut file = match File::open(json_path) {
        Ok(f) => f,
        Err(e) => {
            println!("Error opening JSON file: {}", e);
            return;
        }
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents) {
        println!("Error reading JSON file: {}", e);
        return;
    }

    let set_list = match get_set_list_from_json(&contents) {
        Ok(list) => list,
        Err(e) => {
            println!("Error extracting the set list: {}", e);
            return;
        }
    };

    println!("Set list:");
    for name in set_list {
        println!("   {}", name);
    }

    // Export set list to HTML
    // match set_list.len() {
    //     0 => println!("No songs found in the set list"),
    //     _ => {
    //         println!("Set list:");
    //         for (index, song_name) in set_list.iter().enumerate() {
    //             println!("   {}. {}", index + 1, song_name);
    //         }
    //     }
    // }
}

fn get_set_list_from_json(json: &str) -> Result<Vec<String>, io::Error> {
    let data: TrelloBoard = serde_json::from_str(json)?;
    let set_list_id = get_set_list_id(&data, "Songs")?;

    let set_list = get_card_names_on_list(&data, &set_list_id)?;

    Ok(set_list)
}

fn get_card_names_on_list(board_data: &TrelloBoard, list_id: &str) -> Result<Vec<String>, io::Error> {
    let mut cards_on_list = vec![];

    match board_data.cards {
        Value::Array(ref cards) => {
            for card in cards {
                // TODO: Filter out archived cards "closed"
                if card["idList"].as_str().unwrap() == list_id {
                    cards_on_list.push(card["name"].as_str().unwrap().to_string());
                }
            }
        },
        _ => return Err(io::Error::new(io::ErrorKind::NotFound, "'lists' was the wrong JSON type. Expected array."))
    }

    Ok(cards_on_list)
}

fn get_set_list_id(board_data: &TrelloBoard, set_list_name: &str) -> Result<String, io::Error> {
    match board_data.lists {
        Value::Array(ref lists) => {
            for list in lists {
                match list {
                    &Value::Object(ref list_data) => {
                        let list_name = list_data["name"].as_str().unwrap();
                        println!("   {}", list_name);
                        if list_name == set_list_name {
                            let id = list_data["id"].as_str().unwrap().to_string();
                            println!("Found set list: {}", id);
                            return Ok(id);
                        }
                    }
                    _ => continue
                }
            }
        },
        _ => return Err(io::Error::new(io::ErrorKind::NotFound, "'lists' was the wrong JSON type. Expected array."))
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "Couldn't find 'lists'"))
}

#[derive(Serialize, Deserialize)]
struct TrelloBoard {
    cards: Value,
    lists: Value,
}