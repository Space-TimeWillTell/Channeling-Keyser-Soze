extern crate clap;
extern crate env_logger;
extern crate itertools;
#[macro_use]
extern crate log;

use std::path::PathBuf;
use std::collections::HashSet;
use std::process::Command;

use clap::{ Arg, App };
use itertools::Itertools;

const NUMBER_OF_CARDS: usize = 87;

const TMP_DIR : &'static str = "/tmp/spacetimedeck/";
const TMP_MAX_QUALITY_HORIZONTAL_PDF_PREFIX : &'static str = "/tmp/spacetimedeck/pdf/";
const TMP_MAX_QUALITY_HORIZONTAL_PNG_PREFIX : &'static str = "/tmp/spacetimedeck/png/";

/// The dimensions of images when extracted from the pdf
const PIXEL_WIDTH_INITIAL: isize = 2250;
const PIXEL_HEIGHT_INITIAL: isize = 1650;

const PIXEL_WIDTH_WITHOUT_BLEED: isize = 1964;
const PIXEL_HEIGHT_WITHOUT_BLEED: isize = 1360;

const PIXEL_WIDTH_WITH_PHOTO_BLEED: isize = 2000;
const PIXEL_HEIGHT_WITH_PHOTO_BLEED: isize = 1370;

/// Size of the "safety zone" in the cards document.
/// That's the place where we don't put text or images
/// to avoid it being cut.
const SAFETY_ZONE_IN_SOURCE_DOCUMENT_CM: f64 = 2.54;
const SOURCE_DOCUMENT_WIDTH_CM: f64 = 38.;
const SOURCE_DOCUMENT_HEIGHT_CM: f64 = 28.;
const INITIAL_PADDING_BOTTOM: usize = (SAFETY_ZONE_IN_SOURCE_DOCUMENT_CM * PIXEL_HEIGHT_INITIAL as f64 / SOURCE_DOCUMENT_HEIGHT_CM) as usize;
const INITIAL_PADDING_TOP:    usize = (SAFETY_ZONE_IN_SOURCE_DOCUMENT_CM * PIXEL_HEIGHT_INITIAL as f64 / SOURCE_DOCUMENT_HEIGHT_CM) as usize;
const INITIAL_PADDING_LEFT:   usize = (SAFETY_ZONE_IN_SOURCE_DOCUMENT_CM * PIXEL_WIDTH_INITIAL as f64 / SOURCE_DOCUMENT_WIDTH_CM) as usize;
const INITIAL_PADDING_RIGHT:  usize = (SAFETY_ZONE_IN_SOURCE_DOCUMENT_CM * PIXEL_WIDTH_INITIAL as f64 / SOURCE_DOCUMENT_WIDTH_CM) as usize;

const PAGE_WIDTH_A4_PIXELS: usize = (PIXEL_WIDTH_INITIAL as f64 * 2.5) as usize; // Arbitrary width.
const PAGE_HEIGHT_A4_PIXELS: usize = ((PAGE_WIDTH_A4_PIXELS as f64) * 29.7 / 21.) as usize;

struct Dimensions {
    lines: usize,
    rows: usize,
}

struct Options {
    /// Source pdf.
    source_front: PathBuf,
    source_back: PathBuf,


    /// The indices of cards, starting at 0.
    cards: HashSet<usize>,

    regenerate_pdfs: bool,
    regenerate_pngs: bool,

    vignette_dimensions: Dimensions,
    print_at_home_dimensions: Dimensions,

    /// Destination directory.
    dest_montage: Option<PathBuf>,
    dest_max_quality_horizontal_cards: PathBuf,
    dest_book_quality_cards: Option<PathBuf>,
    dest_printer_cards: Option<PathBuf>,
    dest_print_at_home_cards: Option<PathBuf>,
    dest_print_as_photos: Option<PathBuf>,

    color_profile_path: Option<PathBuf>,
}

fn command_high_quality_horizontal_with_bleed(options: &Options, source: &PathBuf, name: &str) -> Command {
    let dest = options.dest_max_quality_horizontal_cards
        .join(format!("{}.png", name));
    let dest = dest
        .to_str()
        .expect("Path String error");
    let mut command = Command::new("sips");
    command
        .stdout(std::process::Stdio::null())        // Mute sips
        .args(&["-s", "format", "png"])             // Output format
        .args(&["--out", dest])                     // Output file
        .arg(source);                                // Input file
    command
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
        .arg(Arg::with_name("back")
            .help("Source pdf file for the back")
            .short("b")
            .long("back")
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
/*
        .arg(Arg::with_name("vignettes_per_line")
            .long("vignettes_per_line")
            .takes_value(true)
            .default_value("2")
            .help("Number of cards in each vignette line.")
        )
        .arg(Arg::with_name("vignettes_per_row")
            .long("vignettes_per_row")
            .takes_value(true)
            .default_value("4")
            .help("Number of cards in each vignette row.")
        )
*/
        .arg(Arg::with_name("regenpdf")
            .long("regenpdf")
            .conflicts_with("no-regenpdf")
            .help("Default behavior. If specified, regenerate max quality pdf-per-page.")
        )
        .arg(Arg::with_name("no-regenpdf")
            .long("no-regenpdf")
            .help("If specified, do not regenerate max quality pdf-per-page.")
        )
        .arg(Arg::with_name("regenpng")
            .long("regenpng")
            .conflicts_with("no-regenpng")
            .help("Default behavior. If specified, regenerate max quality png-per-page.")
        )
        .arg(Arg::with_name("no-regenpng")
            .long("no-regenpng")
            .help("If specified, do not regenerate max quality png-per-page.")
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
    let source_front = PathBuf::from(matches.value_of("input")
        .unwrap());

    let source_back = PathBuf::from(matches.value_of("back")
        .unwrap());

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

    Options {
        cards,
        source_front,
        source_back,
        regenerate_pdfs: !matches.is_present("no-regenpdf"),
        regenerate_pngs: !matches.is_present("no-regenpng"),
        dest_max_quality_horizontal_cards,
        dest_book_quality_cards,
        dest_print_at_home_cards,
        dest_printer_cards,
        dest_print_as_photos,
        dest_montage,
        color_profile_path,
        vignette_dimensions: Dimensions {
            lines: 4,
            rows: 2,
        },
        print_at_home_dimensions: Dimensions {
            lines: 4,
            rows: 2,
        }
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
        source = options.source_front.as_os_str().to_string_lossy());

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
            .arg(&options.source_front);
        debug!(target: "generate", "High Quality PDFs: Starting command {:?}", command);
        command
            .spawn()
            .expect("Command failed to start")
            .wait()
            .expect("Could not extract high quality pdfs");
    } else {
        println!("Skipping pdf regeneration");
    }

    if options.regenerate_pngs {
        print!("Extracting high quality horizontal card(s) (with bleed)");

        batch(cards.iter()
                .map(|i| {
                    let source = PathBuf::from(TMP_MAX_QUALITY_HORIZONTAL_PDF_PREFIX)
                        .join(format!("{index}.{ext}",
                            index = i,
                            ext = "pdf"));
                    let mut command = command_high_quality_horizontal_with_bleed(&options, &source, &format!("{}", i));
                    debug!(target: "generate", "High Quality PNGs with bleed: Starting command {:?}", command);
                    command
                        .spawn()
                        .expect("Command failed to start")
                })
                .chain(vec![
                    {
                        // Rotate the back.
                        let dest = options.dest_max_quality_horizontal_cards
                            .join(format!("{}.png", "back"));
                        let dest = dest
                            .to_str()
                            .expect("Path String error");

                        let mut command = Command::new("sips");
                        command
                            .stdout(std::process::Stdio::null())        // Mute sips
                            .args(&["-s", "format", "png"])             // Output format
                            .arg(&options.source_back)
                            .args(&["--rotate", "90"])
                            .args(&["--resampleHeightWidth",
                                &format!("{}", PIXEL_WIDTH_INITIAL),
                                &format!("{}", PIXEL_HEIGHT_INITIAL),
                            ])
                            .args(&["--out", &dest]);
                        debug!(target: "generate", "High Quality PNGs with bleed (back): Starting command {:?}", command);
                        command
                            .spawn()
                            .expect("Command failed to start")
                    }
                ].into_iter())
            .collect());
    } else {
        println!("Skipping high quality horizontal cards regeneration");
    }

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
        std::fs::create_dir_all(dest_printer_cards)
            .expect("Could not create directory");

        let tasks = cards.iter()
            .map(|i| format!("{}", i))
            .chain(Some("back".to_string()).into_iter())
            .map(|i| {
                let source = options.dest_max_quality_horizontal_cards
                    .join(format!("{}.png", i));
                let source = source
                    .to_str()
                    .expect("Path String error");
                let dest = dest_printer_cards
                    .join(format!("{index}.tiff",
                        index = i));
                let dest = dest
                    .to_str()
                    .expect("Path String error");

                let mut command = Command::new("sips");
                command
                    .stdout(std::process::Stdio::null())
                    .args(&["-s", "format", "tiff"])             // Output format
                    .arg(&source)
                    .args(&["--rotate", "90"])                    // Need vertical format
                    .args(&["--out", &dest]);
                if let Some(ref path) = options.color_profile_path {
                    let path = path.to_str()
                        .expect("Path String error");
                    command.args(&["-m", path]);
                }

                debug!(target: "generate", "Generating printer file: {:?}", command);
                command
                    .spawn()
                    .expect("Command failed to start")
            }).collect();
        batch(tasks);
    }


    if let Some(ref dest_print_at_home_cards) = options.dest_print_at_home_cards {
        std::fs::create_dir_all(dest_print_at_home_cards)
            .expect("Could not create directory");

/*

//        let dx = ((PIXEL_WIDTH_INITIAL as f64 - PIXEL_WIDTH_WITHOUT_BLEED as f64 / BLEED_FACTOR_WIDTH_PRINT_YOURSELF) / 2.) as usize;
//        let dy = ((PIXEL_HEIGHT_INITIAL as f64 - PIXEL_HEIGHT_WITHOUT_BLEED as f64 / BLEED_FACTOR_HEIGHT_PRINT_YOURSELF) / 2.) as usize;
        let dx = 0;
        let dy = 0;

        let w  = PIXEL_WIDTH_INITIAL as isize - dx as isize;
        let h  = PIXEL_HEIGHT_INITIAL as isize - dy as isize;
*/
/*
        print!("Extracting single horizontal cards for print-at-home");



        let tasks = cards.iter()
            .map(|name| {
                let source = options.dest_max_quality_horizontal_cards
                    .join(format!("{source}.png[{w}x{h}+{dx}+{dy}]",
                        source = name,
                        dx = dx,
                        dy = dy,
                        w = w,
                        h = h));
                let dest = dest_print_at_home_cards.join(format!("{}.png", name));
                let mut command = Command::new("convert");
                command.arg(source.into_os_string())
                    .arg(dest);

                command.spawn()
                    .expect("Coud not launch command")
            })
            .collect();
        batch(tasks);
*/
        print!("Extracting entire pages for print-at-home");

        let images_per_page = options.print_at_home_dimensions.lines as usize
                * options.print_at_home_dimensions.rows as usize;

        let groups = (1..NUMBER_OF_CARDS+1)
            .chunks(images_per_page);


        // Width and height of the page, in pixels, including bleed.
        let page_width = PAGE_WIDTH_A4_PIXELS;
        let page_height = PAGE_HEIGHT_A4_PIXELS;

        // Width and height of the image, in pixels.
        let image_width = PIXEL_WIDTH_INITIAL as usize;
        let image_height = PIXEL_HEIGHT_INITIAL as usize;

        // Compute margins that will let us put all the images on the document.
        let margin_width = (page_width / options.print_at_home_dimensions.rows - image_width) / 2;
        let margin_height = (page_height / options.print_at_home_dimensions.lines - image_height) / 2;

        let image_plus_margin_width = image_width + margin_width * 2;

        let image_plus_margin_height = image_height + margin_height * 2;

        assert_eq!(image_plus_margin_width, page_width / options.print_at_home_dimensions.rows, "Images per line");
        assert_eq!(image_plus_margin_height, page_height / options.print_at_home_dimensions.lines, "Images per row");

        let tasks = groups.into_iter()
            .map(|group| {
                group
                    .map(|i| format!("{}", i))
                    .collect_vec()
            })
            .chain({
                let vec = itertools::repeat_n("back".to_string(), images_per_page)
                    .collect_vec();
                Some(vec).into_iter()
            })
            .map(|group| {
                let dest = dest_print_at_home_cards
                    .join(format!("{}-{}.png", group[0], group[group.len() - 1]));
                let dest = dest.to_str()
                    .unwrap();

                let mut command = Command::new("convert");
                command
                    // Page size, in pixels.
                    .args(&["-size", &format!("{width}x{height}",
                        width = page_width,
                        height = page_height,
                    )])
                    // Page background.
                    .arg("xc:transparent");

                // Prepare drawing lines.
                command
                    .args(&["-stroke", "black"])
                    .args(&["-strokewidth", "3"])
                    .args(&["-fill", "none"]);

                // Draw horizontal lines
                for i in 0..options.print_at_home_dimensions.lines {
                    let x0 = 0;
                    let x1 = page_width;
                    // Draw horizontal line for the top of the image
                    let y = i * image_plus_margin_height +
                        margin_height + INITIAL_PADDING_TOP;
                    command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                        y0 = y,
                        y1 = y,
                        x0 = x0,
                        x1 = x1)]);


                    // Draw horizontal line for the bottom of the image
                    let y = (i + 1) * image_plus_margin_height - margin_height - INITIAL_PADDING_BOTTOM;
                    command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                        y0 = y,
                        y1 = y,
                        x0 = x0,
                        x1 = x1)]);

                }
                // Draw vertical lines
                for i in 0..options.print_at_home_dimensions.rows {
                    let y0 = 0;
                    let y1 = page_height;

                    // Draw vertical line for the left of the image
                    let x = i * image_plus_margin_width +
                        margin_width + INITIAL_PADDING_LEFT;
                    command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                        y0 = y0,
                        y1 = y1,
                        x0 = x,
                        x1 = x)]);

                    // Draw vertical line for the right of the image
                    let x = (i + 1) * image_plus_margin_width - margin_width - INITIAL_PADDING_RIGHT;
                    command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                        y0 = y0,
                        y1 = y1,
                        x0 = x,
                        x1 = x)]);
                }

                let mut sources = group.iter();
                'grid: for i in 0..options.print_at_home_dimensions.rows {
                    for j in 0..options.print_at_home_dimensions.lines {
                        let source = if let Some(source) = sources.next() {
                            source
                        } else {
                            // We've run out of sources.
                            break 'grid
                        };
                        let path = options.dest_max_quality_horizontal_cards
                            .join(format!("{source}.png",
                                source = source));
/*                            .join(format!("{source}.png[{w}x{h}+{dx}+{dy}]",
                                source = source,
                                dx = 0, // Full image
                                dy = 0, // Full image
                                w = w,
                                h = h));*/
                        command.arg(path.into_os_string());

                        let geometry = format!("+{x}+{y}",
                            x = i * image_plus_margin_width + margin_width,
                            y = j * image_plus_margin_height + margin_height);
                        command.args(&["-geometry", &geometry])
                            .arg("-composite");
                    }
                }
                debug!(target: "generate", "Generating print-at-home: {:?}", command);
                command.arg(&dest);
                command.spawn()
                    .expect("Could not launch command")
            })
/*
            .chain(vec![
                {
                    let mut command = Command::new("convert");
                    command
                        // Page size, in pixels.
                        .args(&["-size", &format!("{width}x{height}",
                            width = page_width,
                            height = page_height)])
                        // Page background
                        .arg("xc:white");

                    command
                        .args(&["-stroke", "black"])
                        .args(&["-fill", "none"]);

                    // Draw horizontal lines
                    for i in 0..options.print_at_home_dimensions.lines {
                        let x0 = 0;
                        let x1 = page_width;
                        // Draw horizontal line for the top of the image
                        let y = i * image_plus_margin_height + margin_height;
                        command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                            y0 = y,
                            y1 = y,
                            x0 = x0,
                            x1 = x1)]);
                        // Draw horizontal line for the bottom of the image
                        let y = (i + 1 ) * image_plus_margin_height - margin_height;
                        command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                            y0 = y,
                            y1 = y,
                            x0 = x0,
                            x1 = x1)]);
                    }

                    // Draw vertical lines
                    for i in 0..options.print_at_home_dimensions.rows {
                        let y0 = 0;
                        let y1 = page_height;

                        // Draw vertical line for the left of the image
                        let x = i * image_plus_margin_width + margin_width;
                        command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                            y0 = y0,
                            y1 = y1,
                            x0 = x,
                            x1 = x)]);

                        // Draw vertical line for the right of the image
                        let x = (i + 1) * image_plus_margin_width - margin_width;
                        command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                            y0 = y0,
                            y1 = y1,
                            x0 = x,
                            x1 = x)]);
                    }

                    let mut source = options.dest_max_quality_horizontal_cards
                        .join("back.png");
                    source.set_extension(format!("png[{w}x{h}+{dx}+{dy}]",
                        dx = dx,
                        dy = dy,
                        w = w,
                        h = h));
                    let source = source.into_os_string();
                    let dest = dest_print_at_home_cards.join("back.png");
                    let dest = dest.to_str()
                        .unwrap();

                    for i in 0..options.print_at_home_dimensions.rows {
                        for j in 0..options.print_at_home_dimensions.lines {
                            command.arg(&source);

                            let geometry = format!("+{x}+{y}",
                                x = i * image_plus_margin_width + margin_width / 2,
                                y = j * image_plus_margin_height + margin_height / 2);
                            command.args(&["-geometry", &geometry])
                                .arg("-composite");
                        }
                    }
                    command.arg(&dest);
                    debug!(target: "generate", "Generating print-at-home: {:?}", command);
                    command.spawn()
                        .expect("Could not launch command")
                }
            ].into_iter())
*/
            .collect();
        batch(tasks);

        // Convert into a single pdf
        println!("Converting print-at-home horizontal cards into a single pdf");
        let groups = (1..NUMBER_OF_CARDS+1)
            .chunks(options.print_at_home_dimensions.lines as usize
                * options.print_at_home_dimensions.rows as usize);
        let sources = groups.into_iter()
            .map(|group| {
                let group : Vec<_> = group.collect();
                let source = dest_print_at_home_cards
                    .join(format!("{}-{}.png", group[0], group[group.len() - 1]));
                source.to_str()
                    .unwrap()
                    .to_string()
            })
            .chain(Some( {
                let source = dest_print_at_home_cards
                    .join("back.png");
                source.to_str()
                    .unwrap()
                    .to_string()
            }).into_iter());
        let mut command = Command::new("convert");
        for source in sources {
            command.arg(source);
        }
//        command.args(&["-resize", &format!("{:0}%", RESAMPLE_FACTOR_PRINT_YOURSELF * 100.)]);
        command.args(&["-density", "300"]);
        let dest = dest_print_at_home_cards
            .join("cards.pdf");
        let dest = dest.to_str()
            .unwrap();
        command.arg(dest);
        debug!(target: "generate", "Generating single print-at-home: {:?}", command);
        command.spawn()
            .expect("Could not launch command")
            .wait()
            .expect("Error executing command");
    }



/*
    if let Some(ref dest_montage) = options.dest_montage {
        std::fs::create_dir_all(dest_montage)
            .expect("Could not create directory");

        print!("Exporting cards for self-printing (front)");
        let groups = (1..NUMBER_OF_CARDS+1)
            .chunks(options.vignettes_per_montage_line as usize * options.vignettes_per_montage_row as usize);
        let tile = format!("{}x{}",
            options.vignettes_per_montage_line,
            options.vignettes_per_montage_row);
        let geometry = format!("30+30+{}x{}",
            PIXEL_WIDTH_WITH_PRINT_YOURSELF_BLEED,
            PIXEL_HEIGHT_WITH_PRINT_YOURSELF_BLEED
        );

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
                    // FIXME: We probably don't need the entire bleed.
                Command::new("montage")
                    .args(&["-tile", &tile])
                    .args(&["-geometry", &geometry])
                    .args(&sources)
                    .arg(dest)
                    .spawn()
                    .expect("Command failed to start")
            })
            .collect()
        );

        print!("Exporting cards for self-printing (back) FIXME");
    }
*/

    if let Some(ref dest_print_as_photos) = options.dest_print_as_photos {
        std::fs::create_dir_all(dest_print_as_photos)
            .expect("Could not create directory");
        let pixel_width_with_photo_bleed = format!("{}", PIXEL_WIDTH_WITH_PHOTO_BLEED);

        let desired_height = (1.5 * (PIXEL_WIDTH_WITH_PHOTO_BLEED as f64)) as isize;
        let dx = (PIXEL_WIDTH_INITIAL - PIXEL_WIDTH_WITH_PHOTO_BLEED) / 2;
        let dy = (PIXEL_HEIGHT_INITIAL - PIXEL_HEIGHT_WITH_PHOTO_BLEED) / 2;

        let pos_top = "+0+0".to_string();
        let pos_bottom = format!("+0+{}",
            desired_height - PIXEL_HEIGHT_WITH_PHOTO_BLEED);
        let groups = (1..NUMBER_OF_CARDS+1)
            .chunks(2);

        print!("Assembling 2 images into one 10x15 photos");
        let tasks = groups.into_iter()
            .map(|group| {
                let group : Vec<_> = group.collect();
                let dest = dest_print_as_photos
                    .join(format!("{}-{}.jpg", group[0], group[group.len() - 1]));
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
            }).collect();
        batch(tasks);
    }
}
