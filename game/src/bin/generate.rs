//! Randomly generates and prints a dominoes layout

use std::u8;

use clap::{Arg, Command as ClapCommand};
use rules::{self, Configuration, Tile};
use dominoes_state::{Boneyard, Layout};

fn main() {
    // Parse command line arguments
    // The command line consists of an optional string argument specifying the max_size of the layout to generate. If not provided,
    // it defaults to the full set max_size.
    let matches = ClapCommand::new("Layout Generator")
        .version("1.0")
        .author("Jambolo <jambolo@users.noreply.github.com>")
        .arg(Arg::new("max_size")
            .help("Size of the layout (number of tiles)")
            .required(false)
            .index(1))
        .arg(Arg::new("json")
            .long("json")
            .short('j')
            .help("Output in JSON format")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    let configuration = Configuration::default();
    let set_size = configuration.set_size();
    let max_size = parse_size_parameter(&matches, set_size);
    let json_output = matches.get_flag("json");

    if max_size > set_size {
        eprintln!("Error: The max_size of the layout ({}) cannot be greater than the number of tiles in the set ({})", max_size, set_size);
        std::process::exit(1);
    }

    let layout = generate_random_layout(&configuration, max_size);

    if json_output {
        unimplemented!();
    } else {
        println!("{}", layout.to_string());
    }
}

fn parse_size_parameter(matches: &clap::ArgMatches, default_size: usize) -> usize {
    if let Some(size_str) = matches.get_one::<String>("max_size") {
        size_str.parse()
            .unwrap_or_else(|_| {
                eprintln!("Error: Size must be a valid number");
                std::process::exit(1);
            })
    } else {
        default_size
    }
}

fn generate_random_layout(configuration: &Configuration, max_size: usize) -> Layout {
    let mut boneyard = Boneyard::new(&configuration);

    let mut layout = Layout::new(&configuration);
    let mut hand = Vec::new();
    let mut size = 0;

    // Draw from the boneyard until a double is found and play it.
    while let Some(tile) = boneyard.draw() {
        if tile.is_double() {
            layout.attach(tile, None);
            size += 1;
            break;
        } else {
            hand.push(tile);
        }
    }

    // Continue drawing and playing tiles until the layout reaches the desired max_size or the boneyard is empty
    loop {
        // Repeatedly go through the hand and attach any tile to the layout until no more can be attached
        let mut made_progress = true;
        while made_progress {
            // Find the first tile in the hand that can be attached to the layout by iterating through the hand looking for a
            // tile that has values that match the layout's open ends
            let mut found = None;
            for tile in &hand {
                let optional_index = can_attach(&layout, tile);
                if let Some(index) = optional_index {
                    found = Some((*tile, index));
                    break;
                }
            }
            if let Some((tile, index)) = found {
                // Attach the tile to the layout at the found index
                layout.attach(tile, Some(index));
                size += 1;
                // Remove the tile from the hand
                find_and_remove_tile(&mut hand, tile);

                if size >= max_size {
                    break;
                }
            } else {
                made_progress = false;
            }
        }

        if size >= max_size {
            break;
        }

        // Try to draw a tile from the boneyard and add it to the hand
        if let Some(tile) = boneyard.draw() {
            // Add the tile to the hand to the front since none of the other tiles could be played
            hand.insert(0, tile);
        } else {
            // No more tiles in boneyard, break early
            break;
        }
    }
    layout
}

fn find_and_remove_tile(hand: &mut Vec<Tile>, tile: Tile) {
    if let Some(pos) = hand.iter().position(|&t| t == tile) {
        hand.swap_remove(pos);
    }
}

fn can_attach(layout: &Layout, tile: &Tile) -> Option<usize> {
    let (a, b) = tile.into();
    any_node_with_open_end(&layout, a).or_else(|| any_node_with_open_end(layout, b))
}

fn any_node_with_open_end(layout: &Layout, end: u8) -> Option<usize> {
    layout.get_nodes_with_open_end(end).first().cloned()
}
