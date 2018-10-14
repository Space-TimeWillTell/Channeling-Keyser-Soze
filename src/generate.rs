extern crate clap;
extern crate env_logger;
extern crate itertools;
#[macro_use]
extern crate log;
#[macro_use]
extern crate smallvec;

use std::path::PathBuf;
use std::collections::HashSet;
use std::process::Command;

use clap::{ Arg, App };
use itertools::Itertools;
use smallvec::SmallVec;

const NUMBER_OF_CARDS: usize = 87;

const TMP_DIR : &'static str = "/tmp/spacetimedeck/";
const TMP_MAX_QUALITY_HORIZONTAL_PDF_PREFIX : &'static str = "/tmp/spacetimedeck/pdf/";
const TMP_MAX_QUALITY_HORIZONTAL_PNG_PREFIX : &'static str = "/tmp/spacetimedeck/png/";

const PIXEL_WIDTH_TOTAL: isize = 2250;
const PIXEL_HEIGHT_TOTAL: isize = 1650;

const PIXEL_WIDTH_WITHOUT_BLEED: isize = 1964;
const PIXEL_HEIGHT_WITHOUT_BLEED: isize = 1360;

const PIXEL_WIDTH_WITH_PHOTO_BLEED: isize = 2000;
const PIXEL_HEIGHT_WITH_PHOTO_BLEED: isize = 1370;

struct Options {
    /// Source pdf.
    source: String,

    /// The indices of cards, starting at 0.
    cards: HashSet<usize>,

    regenerate_pdfs: bool,
    vignettes_per_montage_page: u32,

    /// Destination directory.
    dest_montage: Option<PathBuf>,
    dest_max_quality_horizontal_cards: PathBuf,
    dest_book_quality_cards: Option<PathBuf>,
    dest_printer_cards: Option<PathBuf>,
    dest_print_at_home_cards: Option<PathBuf>,
    dest_print_as_photos: Option<PathBuf>,

    color_profile_path: Option<PathBuf>,
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
        .arg(Arg::with_name("bookdir")
            .long("bookdir")
            .takes_value(true)
            .help("Destination up/down/horizontal cards, book quality.")
        )
        .arg(Arg::with_name("printerdir")
            .long("printerdir")
            .takes_value(true)
            .requires("colorprofile")
            .help("Destination up cards, printer quality.")
        )
        .arg(Arg::with_name("colorprofile")
            .long("colorprofile")
            .takes_value(true)
            .help("Color profile for printer quality files.")
        )
        .arg(Arg::with_name("printathomedir")
            .long("printathomedir")
            .takes_value(true)
            .help("Destination horizontal cards, print-at-home quality.")
        )
        .arg(Arg::with_name("photosdir")
            .long("photosdir")
            .takes_value(true)
            .help("Destination directory for print as 2 cards per 10x15 photo.")
        )
        .arg(Arg::with_name("vignettedir")
            .long("vignettedir")
            .takes_value(true)
            .help("Destination for a thumbnail montage.")
        )
        .arg(Arg::with_name("vignettes_per_page")
            .long("vignettes_per_page")
            .takes_value(true)
            .default_value("10")
            .help("Number of cards in each vignette page.")
        )
        .arg(Arg::with_name("noregenpdf")
            .long("noregenpdf")
            .help("If specified, do not regenerate pdf-per-page.")
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
        for i in 1..(NUMBER_OF_CARDS + 1) {
            cards.insert(i);
        }
    }

    // Source
    let source = matches.value_of("input")
        .unwrap()
        .to_string();

    let dest_max_quality_horizontal_cards = PathBuf::from(TMP_MAX_QUALITY_HORIZONTAL_PNG_PREFIX);

    let dest_book_quality_cards = matches.value_of("bookdir")
        .map(|value| PathBuf::from(value));

    let dest_print_at_home_cards = matches.value_of("printathomedir")
        .map(|value| PathBuf::from(value));

    let dest_printer_cards = matches.value_of("printerdir")
        .map(|value| PathBuf::from(value));

    let dest_print_as_photos = matches.value_of("photosdir")
        .map(|value| PathBuf::from(value));

    let color_profile_path = matches.value_of("colorprofile")
        .map(|value| PathBuf::from(value));

    let dest_montage = matches.value_of("vignettedir")
        .map(|value| PathBuf::from(value));

    let vignettes_per_montage_page = matches.value_of("vignettes_per_page")
        .unwrap()
        .parse::<u32>()
        .unwrap();

    Options {
        cards,
        source,
        regenerate_pdfs: !matches.is_present("noregenpdf"),
        dest_max_quality_horizontal_cards,
        dest_book_quality_cards,
        dest_print_at_home_cards,
        dest_printer_cards,
        dest_print_as_photos,
        dest_montage,
        color_profile_path,
        vignettes_per_montage_page,
    }
}

fn batch(commands: Vec<std::process::Child>) {
    use std::io::Write;
    std::io::stdout()
        .flush()
        .unwrap();
    for mut command in commands.into_iter() {
        print!(".");
        std::io::stdout()
            .flush()
            .unwrap();
        command.wait()
            .expect("Command failed to execute"); // Note: We could parallelize more.
    }
    println!("   DONE");
}

fn main() {
    env_logger::init();
    let options = parse_cli();
    let cards = options.cards.iter()
        .cloned()
        .sorted();

    println!("Regenerating card(s) {cards} from {source}",
        cards = cards.iter().format(","),
        source = options.source);

    for dir in &[TMP_DIR, TMP_MAX_QUALITY_HORIZONTAL_PDF_PREFIX, TMP_MAX_QUALITY_HORIZONTAL_PNG_PREFIX] {
        std::fs::create_dir_all(dir)
            .expect("Could not create directory");
    }

    if options.regenerate_pdfs {
        println!("Extracting high quality pdfs (slow)");
/* // For some reason, this attempt to batch doesn't seem to handle all pages.
        batch(cards.chunks(5)
            .into_iter()
            .map(|group| {
                assert!(group.len() != 0);
                Command::new("gs")
                    .arg(&format!("-dFirstPage={}", group[0]))
                    .arg(&format!("-dLastPage={}",  group[group.len() - 1]))
                    .arg("-dAutoRotatePages=/None") // No rotation
                    .arg("-r300")                   // Resolution: 300dpi
                    .arg("-sDEVICE=pdfwrite")       // Write to pdf (so no loss)
                    .arg("-dBATCH")
                    .arg("-dNOPAUSE")               // Don't wait for input
                    .arg(&format!("-sOutputFile={prefix}%d.{ext}",
                        prefix = TMP_MAX_QUALITY_HORIZONTAL_PDF_PREFIX,
                        ext = "pdf")) // Destination files
                    .arg("-quit")                   // Quit once done
                    .arg(&options.source)
                    .spawn()
                    .expect("Command failed to start")
            })
            .collect()
        )
*/

        let mut command = Command::new("gs");
        command
            .arg("-dAutoRotatePages=/None") // No rotation
            .arg("-r300")                   // Resolution: 300dpi
            .arg("-sDEVICE=pdfwrite")       // Write to pdf (so no loss)
            .arg("-dBATCH")
            .arg("-dNOPAUSE")               // Don't wait for input
            .arg(&format!("-sOutputFile={prefix}%d.{ext}",
                prefix = TMP_MAX_QUALITY_HORIZONTAL_PDF_PREFIX,
                ext = "pdf")) // Destination files
            .arg("-quit")                   // Quit once done
            .arg(&options.source);
        debug!(target: "generate", "High Quality PDFs: Starting command {:?}", command);
        command
            .spawn()
            .expect("Command failed to start")
            .wait()
            .expect("Could not extract high quality pdfs");
    } else {
        println!("Skipping pdf regeneration");
    }

    print!("Extracting high quality horizontal card(s) (with bleed)");
    batch(cards.iter()
            .map(|i| {
                let dest = options.dest_max_quality_horizontal_cards
                    .join(format!("{}.png", i));
                let dest = dest
                    .to_str()
                    .expect("Path String error");
                let mut command = Command::new("sips");
                command
                    .stdout(std::process::Stdio::null())
                    .args(&["-s", "format", "png"])             // Output format
                    .args(&["--out", dest])                     // Output file
                    .arg(&format!("{prefix}{index}.{ext}",
                        prefix = TMP_MAX_QUALITY_HORIZONTAL_PDF_PREFIX,
                        index = i,
                        ext = "pdf"))                           // Input file
                    .args(
                        &if let Some(ref path) = options.color_profile_path {
                            let path = path.to_str()
                                .expect("Path String error");
                            smallvec!["-m", path]
                        } else {
                            let result : SmallVec<[_; 2]> = smallvec![];
                            result
                        }
                    );
                debug!(target: "generate", "High Quality PNGs with bleed: Starting command {:?}", command);
                command
                    .spawn()
                    .expect("Command failed to start")
        })
        .collect());

    if let Some(ref dest_book_quality_cards) = options.dest_book_quality_cards {
        let pixel_height_without_bleed = format!("{}", PIXEL_HEIGHT_WITHOUT_BLEED);
        let pixel_width_without_bleed = format!("{}", PIXEL_WIDTH_WITHOUT_BLEED);
        print!("Extracting book high quality horizontal card(s)");
        batch(cards.iter()
            .map(|i| {
                let source = options.dest_max_quality_horizontal_cards
                    .join(format!("{}.png", i));
                let source = source
                    .to_str()
                    .expect("Path String error");
                let dest = dest_book_quality_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "horizontal_card_",
                        index = i));
                let dest = dest
                    .to_str()
                    .expect("Path String error");
                Command::new("sips")
                    .stdout(std::process::Stdio::null())
                    .args(&["--cropToHeightWidth", &pixel_height_without_bleed, &pixel_width_without_bleed])
                    .args(&["--resampleWidth", "800"])
                    .args(&["-s", "format", "png"])             // Output format
                    .arg(&source)
                    .args(&["--out", &dest])
                    .spawn()
                    .expect("Command failed to start")
            })
            .collect()
        );

        print!("Extracting book high quality up card(s)");
        batch(cards.iter()
            .map(|i| {
                let source = dest_book_quality_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "horizontal_card_",
                        index = i));
                let source = source
                    .to_str()
                    .expect("Path String error");
                let dest = dest_book_quality_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "card_",
                        index = i));
                let dest = dest
                    .to_str()
                    .expect("Path String error");

                Command::new("sips")
                    .stdout(std::process::Stdio::null())
                    .args(&["--rotate", "90"])
                    .args(&["-s", "format", "png"])             // Output format
                    .arg(&source)
                    .args(&["--out", &dest])
                    .spawn()
                    .expect("Command failed to start")
            })
            .collect()
        );

        print!("Extracting book high quality down card(s)");
        batch(cards.iter()
            .map(|i| {
                let source = dest_book_quality_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "horizontal_card_",
                        index = i));
                let source = source
                    .to_str()
                    .expect("Path String error");
                let dest = dest_book_quality_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "reversed_card_",
                        index = i));
                let dest = dest
                    .to_str()
                    .expect("Path String error");

                Command::new("sips")
                    .stdout(std::process::Stdio::null())
                    .args(&["--rotate", "-90"])
                    .args(&["-s", "format", "png"])             // Output format
                    .arg(&source)
                    .args(&["--out", &dest])
                    .spawn()
                    .expect("Command failed to start")
            })
            .collect()
        );

        print!("Extracting book thumbnail quality horizontal card(s)");
        batch(cards.iter()
            .map(|i| {
                let source = dest_book_quality_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "horizontal_card_",
                        index = i));
                let source = source
                    .to_str()
                    .expect("Path String error");

                let dest = dest_book_quality_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "small_horizontal_card_",
                        index = i));
                let dest = dest
                    .to_str()
                    .expect("Path String error");

                Command::new("sips")
                    .stdout(std::process::Stdio::null())
                    .args(&["-s", "format", "png"])             // Output format
                    .arg(&source)
                    .args(&["--resampleWidth", "350"])
                    .args(&["--out", &dest])
                    .spawn()
                    .expect("Command failed to start")
            })
            .collect()
        );

        print!("Extracting book thumbnail quality up card(s)");
        batch(cards.iter()
            .map(|i| {
                let source = dest_book_quality_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "card_",
                        index = i));
                let source = source
                    .to_str()
                    .expect("Path String error");

                let dest = dest_book_quality_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "small_card_",
                        index = i));
                let dest = dest
                    .to_str()
                    .expect("Path String error");

                Command::new("sips")
                    .stdout(std::process::Stdio::null())
                    .args(&["-s", "format", "png"])             // Output format
                    .arg(&source)
                    .args(&["--resampleHeight", "350"])
                    .args(&["--out", &dest])
                    .spawn()
                    .expect("Command failed to start")
            })
            .collect()
        );

        print!("Extracting book thumbnail quality down card(s)");
        batch(cards.iter()
            .map(|i| {
                let source = dest_book_quality_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "reversed_card_",
                        index = i));
                let source = source
                    .to_str()
                    .expect("Path String error");

                let dest = dest_book_quality_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "small_reversed_card_",
                        index = i));
                let dest = dest
                    .to_str()
                    .expect("Path String error");

                Command::new("sips")
                    .stdout(std::process::Stdio::null())
                    .args(&["-s", "format", "png"])             // Output format
                    .arg(&source)
                    .args(&["--resampleHeight", "350"])
                    .args(&["--out", &dest])
                    .spawn()
                    .expect("Command failed to start")
            })
            .collect()
        );
    }

    if let Some(ref dest_printer_cards) = options.dest_printer_cards {
        print!("Extracting printer quality horizontal card(s)");
        if dest_printer_cards == &options.dest_max_quality_horizontal_cards {
            println!("...skipped");
        } else {
            for i in 1..NUMBER_OF_CARDS + 1 {
                print!(".");
                std::fs::copy(
                        options.dest_max_quality_horizontal_cards.join(format!("{}.png", i)),
                        dest_printer_cards.join(format!("{}.png", i))
                ).expect("Copy failed");
            }
            println!("DONE");
        }
    }

    if let Some(ref dest_print_at_home_cards) = options.dest_print_at_home_cards {
        print!("Extracting print-at-home horizontal cards");
        batch(cards.iter()
            .map(|i| {
                let source = options.dest_max_quality_horizontal_cards
                    .join(format!("{}.png", i));
                let source = source
                    .to_str()
                    .expect("Path String error");
                let dest = dest_print_at_home_cards
                    .join(format!("{prefix}{index}.png",
                        prefix = "print_at_home_",
                        index = i));
                let dest = dest
                    .to_str()
                    .expect("Path String error");
                Command::new("sips")
                    .stdout(std::process::Stdio::null())
                    .args(&["--cropToHeightWidth", "2040", "1420"])
                    .args(&["-s", "format", "png"])             // Output format
                    .arg(&source)
                    .args(&["--out", &dest])
                    .spawn()
                    .expect("Command failed to start")
            })
            .collect()
        );
    }

    if let Some(ref dest_montage) = options.dest_montage {
        let groups = (1..NUMBER_OF_CARDS+1)
            .chunks(options.vignettes_per_montage_page as usize);
        batch(groups.into_iter()
            .enumerate()
            .map(|(group_index, group)| {
                let sources : Vec<_> = group
                    .map(|i| {
                        let path = options.dest_max_quality_horizontal_cards
                            .join(format!("{}.png", i));
                        path.into_os_string()
                    })
                    .collect();
                let dest = dest_montage
                    .join(format!("{}.png", group_index));
                Command::new("montage")
                    .args(&sources)
                    .arg(dest)
                    .spawn()
                    .expect("Command failed to start")
            })
            .collect()
        )
    }

    if let Some(ref dest_print_as_photos) = options.dest_print_as_photos {
        std::fs::create_dir_all(dest_print_as_photos)
            .expect("Could not create directory");
        let pixel_width_with_photo_bleed = format!("{}", PIXEL_WIDTH_WITH_PHOTO_BLEED);

        let desired_height = (1.5 * (PIXEL_WIDTH_WITH_PHOTO_BLEED as f64)) as isize;
        let dx = (PIXEL_WIDTH_TOTAL - PIXEL_WIDTH_WITH_PHOTO_BLEED) / 2;
        let dy = (PIXEL_HEIGHT_TOTAL - PIXEL_HEIGHT_WITH_PHOTO_BLEED) / 2;

        let pos_top = "+0+0".to_string();
        let pos_bottom = format!("+0+{}",
            desired_height - PIXEL_HEIGHT_WITH_PHOTO_BLEED);
        let groups = (1..NUMBER_OF_CARDS+1)
            .chunks(2);

        print!("Assembling 2 images into one 10x15 photos");
        batch(groups.into_iter()
            .map(|group| {
                let group : Vec<_> = group.collect();
                let dest = dest_print_as_photos
                    .join(format!("photo-{}-{}.jpg", group[0], group[group.len() - 1]));
                let dest = dest.to_str()
                    .unwrap();

                let mut command = Command::new("convert");
                command
                    .args(&["-size", &format!("{}x{}",
                        pixel_width_with_photo_bleed,
                        desired_height
                    )])
                    .arg("xc:white");
                for (i, source) in group.iter().enumerate() {
                    let path = options.dest_max_quality_horizontal_cards
                        .join(format!("{source}.png[{w}x{h}+{dx}+{dy}]",
                            source = source,
                            dx = dx,
                            dy = dy,
                            w = PIXEL_WIDTH_WITH_PHOTO_BLEED,
                            h = PIXEL_HEIGHT_WITH_PHOTO_BLEED));
                    command.arg(path.into_os_string());
                    let geometry = if i == 0 { &pos_top } else { &pos_bottom };
                    command.args(&["-geometry", geometry])
                        .arg("-composite");
                }
                command.arg(&dest);

                debug!(target: "generate", "Assembling photos: {:?}", command);
                command.spawn()
                    .expect("Could not launch command")
            }).collect()
        );
    }
}
