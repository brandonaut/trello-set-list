#![allow(non_snake_case)]

#[macro_use]
extern crate clap;
extern crate pulldown_cmark;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate textwrap;
extern crate time;

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
    let matches = App::new(crate_name!())
       .version(crate_version!())
       .about("Creates a printable set list out of a Trello board")
       .author(crate_authors!())
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
    // let api_key = "063abb545583e20e7ab609ab534854e4";

    // Read JSON
    let json_path = if Path::new(&input_json).exists() {
        input_json.to_string()
    } else {
        get_json_path_from_user().unwrap()
    };

    println!("Loading Trello data from '{}'", &json_path);
    let mut file = match File::open(&json_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening JSON file: {}", e);
            return;
        },
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents) {
        eprintln!("Error reading JSON file: {}", e);
        return;
    }

    let set_list = match get_set_list_from_json(&contents, &set_list_name) {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Error extracting the set list: {}", e);
            return;
        }
    };

    export_set_list(&set_list, output_filename).expect("Failed exporting set list");
    
    println!("Done");
}

fn get_json_path_from_user() -> Result<String, String> {
    let mut got_it = false;
    let mut path = String::new();

    while !got_it {
        print!("Path to JSON file (Ctrl+C to quit): ");
        io::stdout().flush().unwrap();
        path = String::new();
        if io::stdin().read_line(&mut path).is_ok() {
            path = path.trim_right().to_string();
            if Path::new(&path).exists() {
                got_it = true;
            }
            else { 
                println!("{} not found", &path);
            }
        }
    }
    Ok(path)
}

fn export_set_list(set_list: &[String], output_filename: &str) -> Result<(), io::Error> {

    // Construct contents
    let now = time::now();
    let mut markdown_contents = String::new();
    markdown_contents.extend("## Bookends Set List\n\n".chars());
    markdown_contents.extend(
        format!("_Generated on {}-{:02}-{:02}_\n\n", now.tm_year + 1900, now.tm_mon + 1, now.tm_mday)
        .chars()
    );

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
    println!("Exported to {}", markdown_path.to_str().unwrap());

    // Export HTML
    let parser = Parser::new(&markdown_contents);
    let mut html_contents = String::new();
    html::push_html(&mut html_contents, parser);

    let html_path = markdown_path.with_extension("html");
    {
        let mut outfile = File::create(&html_path)?;
        // TODO: find better way to inject CSS
        outfile.write(textwrap::dedent("
            <head>
                <style>
                h2   {font-family:'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;}
                p    {font-family:'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;}
                li   {font-family:'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;}
                </style>
            </head>
        ").as_bytes())?;
        outfile.write(html_contents.as_bytes())?;
    }
    println!("Exported to {}", html_path.to_str().unwrap());

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
    for list in board.lists.iter() {
        if list.name == set_list_name {
            return Ok(list.id.to_string());
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, format!("Couldn't find list named '{}'", set_list_name)))
}
