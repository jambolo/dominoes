//! Randomly generates and prints a dominoes layout

use clap::{Arg, Command as ClapCommand};
use rules::{Boneyard, Configuration, Layout, Tile, Variation};

const VARIATION_NAMES: &[(&str, Variation)] = &[
    ("traditional", Variation::Traditional),
    ("allfives", Variation::AllFives),
    ("allsevens", Variation::AllSevens),
    ("bergen", Variation::Bergen),
    ("blind", Variation::Blind),
    ("fiveup", Variation::FiveUp),
];

fn main() {
    // Parse command line arguments
    // The command line consists of an optional string argument specifying the max_size of the layout to generate. If not provided,
    // it defaults to the full set max_size.
    let matches = ClapCommand::new("Layout Generator")
        .version("1.0")
        .author("Jambolo <jambolo@users.noreply.github.com>")
        .about("Randomly generates and prints a dominoes layout for a given set and variation.")
        .arg(Arg::new("size")
            .help("Maximum size of the layout (number of tiles)")
            .required(false)
            .index(1)
            .value_parser(clap::value_parser!(usize)))
        .arg(Arg::new("set")
            .long("set")
            .short('s')
            .help("Domino set to use (e.g., 6 for double-six, 9 for double-nine)")
            .required(false)
            .value_parser(clap::value_parser!(u8)))
        .arg(Arg::new("json")
            .long("json")
            .short('j')
            .help("Output in JSON format")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("variation")
            .long("variation")
            .short('v')
            .help("Game variation to use")
            .required(false)
            .value_parser(VARIATION_NAMES.iter().map(|(name, _)| *name).collect::<Vec<_>>()))
        .arg(Arg::new("doubles")
            .long("doubles")
            .short('d')
            .help("Prioritize laying doubles when building the layout")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    let mut max_size = matches.get_one::<usize>("size").copied() .unwrap_or(usize::MAX);
    let mut set_id = matches.get_one::<u8>("set").copied().unwrap_or(u8::MAX);
    let json_output = matches.get_flag("json");
    let variation_str = matches.get_one::<String>("variation").map(|s| s.as_str());
    let prioritize_doubles = matches.get_flag("doubles");

    // Build the configuration
    let num_players = 2;
    let variation = match variation_str {
        None => Variation::Traditional,
        Some(name) => VARIATION_NAMES
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
            .unwrap_or_else(|| {
                let valid = VARIATION_NAMES.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", ");
                eprintln!("Error: Unknown variation '{}'. Valid options are: {}.", name, valid);
                std::process::exit(1);
            }),
    };
    let starting_hand_size = 7; // Doesn't matter for layout generation

    if set_id == u8::MAX {
        set_id = Configuration::DEFAULT_SET_ID;
    } else if set_id > rules::MAX_PIPS {
        eprintln!("Error: set must be between 0 and {} (inclusive)", rules::MAX_PIPS);
        std::process::exit(1);
    }

    let configuration = Configuration::new(num_players, variation, set_id as u8, starting_hand_size);

    // Get the maximum size of the layout to generate
    if max_size == usize::MAX {
        max_size = configuration.set_size();
    } else if max_size > configuration.set_size() {
        eprintln!("Error: The maximum size of the layout ({}) cannot be greater than the number of tiles in the set ({})",
            max_size,
            configuration.set_size());
        std::process::exit(1);
    }

    // Generate the random layout
    let layout = generate_random_layout(&configuration, max_size, prioritize_doubles);

    // Print the layout
    if json_output {
        // Output the layout as JSON using serde_json
        match serde_json::to_string(&layout) {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("Error serializing layout to JSON: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("{}", layout.to_string());
    }
}

fn generate_random_layout(configuration: &Configuration, max_size: usize, prioritize_doubles: bool) -> Layout {
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
            let found = if prioritize_doubles {
                find_playable_tile(&layout, &hand, true)
                    .or_else(|| find_playable_tile(&layout, &hand, false))
            } else {
                find_playable_tile(&layout, &hand, false)
            };

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

/// Find a tile in the hand the index of a node in the layout where the tile can be attached, if any.
fn find_playable_tile(layout: &Layout, hand: &Vec<Tile>, doubles_only: bool) -> Option<(Tile, usize)> {
    for tile in hand {
        if !doubles_only || tile.is_double() {
            if let Some(index) = can_attach(layout, tile) {
                return Some((*tile, index));
            }
        }
    }
    None
}

fn find_and_remove_tile(hand: &mut Vec<Tile>, tile: Tile) {
    if let Some(pos) = hand.iter().position(|&t| t == tile) {
        hand.swap_remove(pos);
    }
}

/// Returns the index of a node in the layout where the tile can be attached, if any.
/// Checks both ends of the tile for a match with any open end in the layout.
fn can_attach(layout: &Layout, tile: &Tile) -> Option<usize> {
    let (a, b) = tile.into();
    layout.get_nodes_with_open_end(a).first().copied()
        .or_else(|| layout.get_nodes_with_open_end(b).first().copied())
}
