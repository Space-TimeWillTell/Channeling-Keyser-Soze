#![feature(futures_api)]

extern crate clap;
extern crate itertools;

use std::collections::HashSet;
use std::process::Command;

use clap::{ Arg, App };
use itertools::Itertools;

const NUMBER_OF_CARDS: usize = 22;
const INTERMEDIATE_FORMAT: &'static str = "jpg";
const FINAL_FORMAT: &'static str = "png";

const TMP_MAX_QUALITY_TWO_CARDS_PREFIX: &'static str = "/tmp/max-quality-two-cards";
const TMP_HIGH_QUALITY_HORIZONTAL_CARD_PREFIX: &'static str = "/tmp/horizontal_card_";

const FINAL_HIGH_QUALITY_UP_CARD_PREFIX: &'static str = "card_";
const FINAL_HIGH_QUALITY_DOWN_CARD_PREFIX: &'static str = "reversed_card_";
const FINAL_THUMBNAIL_UP_CARD_PREFIX: &'static str = "small_card_";
const FINAL_THUMBNAIL_DOWN_CARD_PREFIX: &'static str = "small_reversed_card_";
const FINAL_THUMBNAIL_HORIZONTAL_CARD_NAME: &'static str = "small_horizontal_card_";


const GEOMETRY_TOP: &'static str = "0x2117+0+0";
const GEOMETRY_BOT: &'static str = "0x2117+0+2117";

struct Options {
    /// Source pdf.
    source: String,
    cards: HashSet<usize>,
    /// Destination directory.
    dest: String,
}

fn parse_cli() -> Options {
    let matches = App::new("Deck rebuilder")
        .author("David Teller")
        .arg(Arg::with_name("cards")
            .help("Add one card or a range of cards between 0 and 21. If unspecified, all.")
            .multiple(true)
            .takes_value(true)
            .use_delimiter(true)
        )
        .arg(Arg::with_name("input")
            .help("Source pdf file, two horizontal cards per page.")
            .short("i")
            .long("in")
            .takes_value(true)
            .required(true)
        )
        .get_matches();

    // Cards
    let mut cards = HashSet::new();
    if let Some(values) = matches.values_of("cards") {
        for range in values {
            // Single card number.
            if let Ok(index) = str::parse::<usize>(range) {
                cards.insert(index);
                continue;
            }
            // x-y
            if let Some(index) = range.find('-') {
                let start = str::parse::<usize>(&range[0..index])
                    .unwrap_or_else(|_| panic!("Invalid range format {} (range start)", range));
                let stop = str::parse::<usize>(&range[index + 1 .. range.len()])
                    .unwrap_or_else(|_| panic!("Invalid range format {} (range end)", range));
                for _ in start..stop + 1 {
                    cards.insert(index);
                }
                continue;
            }
            panic!("Invalid range format {}", range);
        }
    } else {
        // All cards.
        for i in 0..NUMBER_OF_CARDS {
            cards.insert(i);
        }
    }

    // Source
    let source = matches.value_of("input")
        .unwrap()
        .to_string();

    let dest = matches.value_of("dest")
        .map(str::to_string)
        .unwrap_or_else(|| "assets".to_string());

    Options {
        cards,
        source,
        dest,
    }
}

fn batch(commands: Vec<std::process::Child>) {
    for mut command in commands.into_iter() {
        print!(".");
        command.wait()
            .expect("Command failed to execute"); // Note: We could parallelize more.
    }
    println!("   DONE");
}

fn main() {
    let Options { cards, source, dest } = parse_cli();
    let cards = cards.into_iter()
        .sorted();
    println!("Regenerating cards {cards} from {source}",
        cards = cards.iter().format(","),
        source = source);


    // Extracting max quality pages from source pdf.
    let pages : Vec<_> = cards.iter()
        .map(|i| i / 2)
        .dedup()
        .collect();
    print!("Regenerating pages {}", pages.iter().format(","));

    batch(pages.into_iter()
            .map(|i| {
                Command::new("convert")
                    .args(&["-density", "300"])
                    .args(&["-quality", "90"])
                    .arg("-trim")
                    .arg(&format!("{source}[{index}]",
                        source = source,
                        index = i))
                    .arg(&format!("{prefix}{index}.{ext}",
                        prefix = TMP_MAX_QUALITY_TWO_CARDS_PREFIX,
                        index = i,
                        ext = INTERMEDIATE_FORMAT))
                    .spawn()
                    .expect("Command failed to start")
            })
            .collect());

    print!("Extracting high quality horizontal cards");
    batch(cards.iter()
            .map(|i| {
                let geometry =
                    if i % 2 == 0 {
                        // Even: top of the page.
                        GEOMETRY_TOP
                    } else {
                        // Odd: bottom of the page.
                        GEOMETRY_BOT
                    };
                Command::new("convert")
                    .arg(&format!("{prefix}{index}.{ext}[{geometry}]",
                        geometry = geometry,
                        prefix = TMP_MAX_QUALITY_TWO_CARDS_PREFIX,
                        index = i / 2,
                        ext = INTERMEDIATE_FORMAT))
                    .arg("-trim")
                    .args(&["-resize", "25%"])
                    .arg(&format!("{prefix}{index}.{ext}",
                        prefix = TMP_HIGH_QUALITY_HORIZONTAL_CARD_PREFIX,
                        index = i,
                        ext = INTERMEDIATE_FORMAT))
                    .spawn()
                    .expect("Command failed to start")
        })
        .collect());


    print!("Extracting high quality up cards");
    batch(cards.iter()
            .map(|i| {
                Command::new("convert")
                    .arg(&format!("{prefix}{index}.{ext}",
                        prefix = TMP_HIGH_QUALITY_HORIZONTAL_CARD_PREFIX,
                        index = i,
                        ext = INTERMEDIATE_FORMAT))
                    .args(&["-rotate", "90"])
                    .arg(&format!("{dest}/{prefix}{index}.{ext}",
                        dest = dest,
                        prefix = FINAL_HIGH_QUALITY_UP_CARD_PREFIX,
                        index = i,
                        ext = FINAL_FORMAT))
                    .spawn()
                    .expect("Command failed to start")
            })
        .collect());

    print!("Extracting high quality down cards");
    batch(cards.iter()
            .map(|i| {
                Command::new("convert")
                    .arg(&format!("{prefix}{index}.{ext}",
                        prefix = TMP_HIGH_QUALITY_HORIZONTAL_CARD_PREFIX,
                        index = i,
                        ext = INTERMEDIATE_FORMAT))
                    .args(&["-rotate", "-90"])
                    .arg(&format!("{dest}/{prefix}{index}.{ext}",
                        dest = dest,
                        prefix = FINAL_HIGH_QUALITY_DOWN_CARD_PREFIX,
                        index = i,
                        ext = FINAL_FORMAT))
                    .spawn()
                    .expect("Command failed to start")
            })
        .collect());

    print!("Extracting horizontal thumbnails");
    batch(cards.iter()
        .map(|i| {
            Command::new("convert")
                .arg(&format!("{prefix}{index}.{ext}",
                        prefix = TMP_HIGH_QUALITY_HORIZONTAL_CARD_PREFIX,
                        index = i,
                        ext = INTERMEDIATE_FORMAT))
                .args(&["-resize", "50%"])
                .arg(&format!("{dest}/{name}{index}.{ext}",
                    dest = dest,
                    name = FINAL_THUMBNAIL_HORIZONTAL_CARD_NAME,
                    index = i,
                    ext = FINAL_FORMAT))
                .spawn()
                .expect("Command failed to start")
        })
        .collect());

    print!("Extracting up thumbnails");
    batch(cards.iter()
        .map(|i| {
            Command::new("convert")
                .arg(&format!("{dest}/{prefix}{index}.{ext}",
                        dest = dest,
                        prefix = FINAL_HIGH_QUALITY_UP_CARD_PREFIX,
                        index = i,
                        ext = FINAL_FORMAT))
                .args(&["-resize", "50%"])
                .arg(&format!("{dest}/{name}{index}.{ext}",
                    dest = dest,
                    name = FINAL_THUMBNAIL_UP_CARD_PREFIX,
                    index = i,
                    ext = FINAL_FORMAT))
                .spawn()
                .expect("Command failed to start")
        })
        .collect());

    print!("Extracting down thumbnails");
    batch(cards.iter()
        .map(|i| {
            Command::new("convert")
                .arg(&format!("{dest}/{prefix}{index}.{ext}",
                        dest = dest,
                        prefix = FINAL_HIGH_QUALITY_DOWN_CARD_PREFIX,
                        index = i,
                        ext = FINAL_FORMAT))
                .args(&["-resize", "50%"])
                .arg(&format!("{dest}/{name}{index}.{ext}",
                    dest = dest,
                    name = FINAL_THUMBNAIL_DOWN_CARD_PREFIX,
                    index = i,
                    ext = FINAL_FORMAT))
                .spawn()
                .expect("Command failed to start")
        })
        .collect());
}
