#![allow(non_snake_case)]

extern crate clap;
extern crate pulldown_cmark;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;
extern crate serde;

use clap::{App, Arg};
use pulldown_cmark::{html, Parser};
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct TrelloBoard {
    cards: Vec<Card>,
    lists: Vec<List>,
}

#[derive(Serialize, Deserialize)]
struct List {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Card {
    closed: bool,
    idList: String,
    name: String,
}

fn main() { 
    let matches = App::new("trello-set-list")
       .version("0.1")
       .about("Creates a printable set list out of a Trello board")
       .author("Brandon H.")
       .arg(Arg::with_name("output")
            .short("o")
            .long("out")
            .help("Output filename")
            .takes_value(true)
       )
       .arg(Arg::with_name("input")
            .short("i")
            .long("in")
            .help("Trello JSON file to process")
            .takes_value(true)
       )
       .arg(Arg::with_name("list_name")
            .short("l")
            .long("list")
            .help("Trello list name")
            .takes_value(true)
       )
       .get_matches();

    let input_json = matches.value_of("input").unwrap_or("exported.json");
    let output_filename: &str = matches.value_of("output").unwrap_or("set_list.md");
    let set_list_name = matches.value_of("list_name").unwrap_or("Set List");

    // TODO: Get json from Trello

    // Read JSON
    let mut file = match File::open(input_json) {
        Ok(f) => f,
        Err(e) => {
            println!("Error opening JSON file '{}': {}", input_json, e);
            return;
        }
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents) {
        println!("Error reading JSON file '{}': {}", input_json, e);
        return;
    }

    let set_list = match get_set_list_from_json(&contents, &set_list_name) {
        Ok(list) => list,
        Err(e) => {
            println!("Error extracting the set list: {}", e);
            return;
        }
    };

    export_set_list(&set_list, output_filename).expect("Failed exporting set list");
}

fn export_set_list(set_list: &[String], output_filename: &str) -> Result<(), io::Error> {

    // Construct contents
    let mut markdown_contents = String::new();
    markdown_contents.extend("# Bookends Set List:\n\n".chars());
    for (index, item) in set_list.iter().enumerate() {

        let mut formatted_item = String::new();
        let items = item.split(" - ").collect::<Vec<&str>>();
        formatted_item.push_str(&format!("  {}. {}", index + 1, &items[0]));

        // Bold any metadata, such as capo and tuning info
        if items.len() == 2 {
            formatted_item.push_str(&format!(" - **{}**", &items[1]));
        }

        formatted_item.push_str("\n");

        markdown_contents.extend(formatted_item.chars());
    }
    
    // Export Markdown
    let markdown_path = Path::new(output_filename).with_extension("md");
    {
        let mut outfile = File::create(&markdown_path)?;
        outfile.write(markdown_contents.as_bytes())?;
    }

    // Export HTML
    let parser = Parser::new(&markdown_contents);
    let mut html_contents = String::new();
    html::push_html(&mut html_contents, parser);

    let html_path = markdown_path.with_extension("html");
    {
        let mut outfile = File::create(html_path)?;
        outfile.write(html_contents.as_bytes())?;
    }

    Ok(())
}

fn get_set_list_from_json(json: &str, set_list_name: &str) -> Result<Vec<String>, io::Error> {
    let data: TrelloBoard = serde_json::from_str(json)?;
    let set_list_id = get_set_list_id(&data, set_list_name)?;

    let set_list = get_card_names_on_list(&data, &set_list_id)?;

    Ok(set_list)
}

fn get_card_names_on_list(board_data: &TrelloBoard, list_id: &str) -> Result<Vec<String>, io::Error> {
    let mut cards_on_list = vec![];

    for card in board_data.cards.iter() {
        if card.idList == list_id && !card.closed {
            cards_on_list.push(card.name.to_string());
        }
    }

    Ok(cards_on_list)
}

fn get_set_list_id(board: &TrelloBoard, set_list_name: &str) -> Result<String, io::Error> {
    println!("Lists:");
    for list in board.lists.iter() {
        println!("   {}", list.name);
        if list.name == set_list_name {
            return Ok(list.id.to_string());
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, format!("Couldn't find list named '{}'", set_list_name)))
}
