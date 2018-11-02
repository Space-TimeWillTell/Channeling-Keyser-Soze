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

/// Size of the "safety zone" in the cards document.
/// That's the place where we don't put text or images
/// to avoid it being cut.
const SAFETY_ZONE_IN_SOURCE_DOCUMENT_CM: f64 = 2.54;
const SOURCE_DOCUMENT_WIDTH_CM: f64 = 38.;
const SOURCE_DOCUMENT_HEIGHT_CM: f64 = 28.;
const DESIRED_PADDING_FACTOR: f64   = 1.1;
const INITIAL_PADDING_BOTTOM: usize = (SAFETY_ZONE_IN_SOURCE_DOCUMENT_CM * PIXEL_HEIGHT_INITIAL as f64 / SOURCE_DOCUMENT_HEIGHT_CM) as usize;
const INITIAL_PADDING_TOP:    usize = (SAFETY_ZONE_IN_SOURCE_DOCUMENT_CM * PIXEL_HEIGHT_INITIAL as f64 / SOURCE_DOCUMENT_HEIGHT_CM) as usize;
const INITIAL_PADDING_LEFT:   usize = (SAFETY_ZONE_IN_SOURCE_DOCUMENT_CM * PIXEL_WIDTH_INITIAL as f64 / SOURCE_DOCUMENT_WIDTH_CM) as usize;
const INITIAL_PADDING_RIGHT:  usize = (SAFETY_ZONE_IN_SOURCE_DOCUMENT_CM * PIXEL_WIDTH_INITIAL as f64 / SOURCE_DOCUMENT_WIDTH_CM) as usize;

enum Format {
    PDF,
    JPG,
//    PNG,
}

struct PageFormat {
    name: &'static str,
    width_pixels: usize,
    height_pixels: usize,
    lines: usize,
    rows: usize,
    format: Format,
}

const PAGE_FORMAT_A4 : PageFormat = {
    const WIDTH_CM: f64 = 21.;
    const HEIGHT_CM: f64 = 29.7;
    const WIDTH_PIXELS: f64 = PIXEL_WIDTH_INITIAL as f64 * 2.15; // Arbitrary width
    PageFormat {
        name: "a4",
        width_pixels: WIDTH_PIXELS as usize,
        height_pixels: (WIDTH_PIXELS * HEIGHT_CM / WIDTH_CM) as usize,
        lines: 4,
        rows: 2,
        format: Format::PDF,
    }
};

const PAGE_FORMAT_LETTER : PageFormat = {
    const WIDTH_CM: f64 = 21.59;
    const HEIGHT_CM: f64 = 27.94;
    const WIDTH_PIXELS: f64 = PIXEL_WIDTH_INITIAL as f64 * 2.35; // Arbitrary width
    PageFormat {
        name: "us_letter",
        width_pixels: WIDTH_PIXELS as usize,
        height_pixels: (WIDTH_PIXELS * HEIGHT_CM / WIDTH_CM) as usize,
        lines: 4,
        rows: 2,
        format: Format::PDF,
    }
};

const PAGE_FORMAT_10_15 : PageFormat = {
    const WIDTH_CM: f64 = 10.;
    const HEIGHT_CM: f64 = 15.;
    const WIDTH_PIXELS: f64 = PIXEL_WIDTH_INITIAL as f64 * 1.1; // Arbitrary width
    PageFormat {
        name: "10x15",
        width_pixels: WIDTH_PIXELS as usize,
        height_pixels: (WIDTH_PIXELS * HEIGHT_CM / WIDTH_CM) as usize,
        lines: 2,
        rows: 1,
        format: Format::JPG,
    }
};

const PAGE_FORMAT_11_15 : PageFormat = {
    const WIDTH_CM: f64 = 11.;
    const HEIGHT_CM: f64 = 15.;
    const WIDTH_PIXELS: f64 = PIXEL_WIDTH_INITIAL as f64 * 1.1; // Arbitrary width
    PageFormat {
        name: "11x15",
        width_pixels: WIDTH_PIXELS as usize,
        height_pixels: (WIDTH_PIXELS * HEIGHT_CM / WIDTH_CM) as usize,
        lines: 2,
        rows: 1,
        format: Format::JPG,
    }
};

struct Options {
    /// Source pdf.
    source_front: PathBuf,
    source_back: PathBuf,


    /// The indices of cards, starting at 0.
    cards: HashSet<usize>,

    regenerate_pdfs: bool,
    regenerate_pngs: bool,

    formats: Vec<PageFormat>,

    /// Destination directories.
    dest_max_quality_horizontal_cards: PathBuf,
    dest_book_quality_cards: Option<PathBuf>,
    dest_printer_cards: Option<PathBuf>,
    dest_print_at_home_cards: Option<PathBuf>,

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
        .arg(source);                               // Input file
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
        .arg(Arg::with_name("format")
            .long("format")
            .takes_value(true)
            .multiple(true)
            .use_delimiter(true)
            .default_value("a4")
            .possible_values(&["a4", "usletter", "10x15", "11x15"])
            .help("Page size")
        )
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

    // Page sizes
    let formats = matches.values_of("format")
        .unwrap()
        .map(|name| {
            match name {
                "a4" => PAGE_FORMAT_A4,
                "usletter" => PAGE_FORMAT_LETTER,
                "10x15" => PAGE_FORMAT_10_15,
                "11x15" => PAGE_FORMAT_11_15,
                _ => panic!("Invalid format {}", name)
            }
        }).collect_vec();

    // Cards
    let mut cards = HashSet::new();
    if let Some(values) = matches.values_of("cards") {
        for range in values {
            debug!(target: "generate", "Adding cards: \"{}\"", range);
            // Single card number.
            if let Ok(index) = str::parse::<usize>(range) {
                debug!(target: "generate", "Adding single card {}", index);
                cards.insert(index);
                continue;
            }
            // x-y
            if let Some(index) = range.find('-') {
                let start = str::parse::<usize>(&range[0..index])
                    .unwrap_or_else(|_| panic!("Invalid range format {} (range start)", range));
                let stop = str::parse::<usize>(&range[index + 1 .. range.len()])
                    .unwrap_or_else(|_| panic!("Invalid range format {} (range end)", range));
                debug!(target: "generate", "Adding card in [{}, {}]", start, stop);
                for index in start..stop + 1 {
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

    debug!(target: "generate", "Eding up with cards: {:?}", cards);
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

    let color_profile_path = matches.value_of("colorprofile")
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
        color_profile_path,
        formats,
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

    println!("Regenerating card(s) {cards} from {source}, to print-at-home formats [{formats}]",
        cards = cards.iter().format(","),
        source = options.source_front.as_os_str().to_string_lossy(),
        formats = options.formats.iter().map(|format| format.name).format(", "));

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
        std::fs::create_dir_all(dest_book_quality_cards)
            .expect("Could not create directory");
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
                let mut command = Command::new("sips");
                command
                    .stdout(std::process::Stdio::null())
                    .args(&["--cropToHeightWidth", &pixel_height_without_bleed, &pixel_width_without_bleed])
                    .args(&["--resampleWidth", "800"])
                    .args(&["-s", "format", "png"])             // Output format
                    .arg(&source)
                    .args(&["--out", &dest]);
                debug!(target: "generate", "Generating book files: {:?}", command);
                command
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

        print!("Extracting entire pages for print-at-home");

        for format in &options.formats {
            let images_per_page = format.lines * format.rows;

            let groups = options.cards.iter()
                .sorted()
                .into_iter()
                .chunks(images_per_page);

            // Width and height of the page, in pixels, including bleed.
            let padding_extension = DESIRED_PADDING_FACTOR;
            let padding_left   = (INITIAL_PADDING_LEFT   as f64 / padding_extension) as usize;
            let padding_right  = (INITIAL_PADDING_RIGHT  as f64 / padding_extension) as usize;
            let padding_top    = (INITIAL_PADDING_TOP    as f64 / padding_extension) as usize;
            let padding_bottom = (INITIAL_PADDING_BOTTOM as f64 / padding_extension) as usize;

            // Width and height of the image, in pixels.
            let image_width = PIXEL_WIDTH_INITIAL as usize;
            let image_height = PIXEL_HEIGHT_INITIAL as usize;

            // Compute margins that will let us put all the images on the document.
            assert!(format.width_pixels >= image_width * format.rows,
                "In format {name}, I need a width of at least {need} pixels, got {got}",
                    name = format.name,
                    need = image_width * format.rows,
                    got = format.width_pixels
            );
            let margin_width = (format.width_pixels / format.rows - image_width) / 2;

            assert!(format.height_pixels >= image_height * format.lines,
                "In format {name}, I need a height of at least {need} pixels, got {got}",
                    name = format.name,
                    need = image_height * format.lines,
                    got = format.height_pixels
            );
            let margin_height = (format.height_pixels / format.lines - image_height) / 2;

            let image_plus_margin_width = image_width + margin_width * 2;

            let image_plus_margin_height = image_height + margin_height * 2;

            assert!((image_plus_margin_width as isize - format.width_pixels as isize / format.rows as isize).abs() <= 1, "Images per line");
            assert!((image_plus_margin_height as isize - format.height_pixels as isize / format.lines as isize).abs() <= 1, "Images per row");

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
                        .join(format!("{format}-{start}-{stop}.tiff",
                            format = format.name,
                            start = group[0],
                            stop = group[group.len() - 1]));
                    let dest = dest.to_str()
                        .unwrap();

                    let mut command = Command::new("convert");
                    command
                        // Page size, in pixels.
                        .args(&["-size", &format!("{width}x{height}",
                            width = format.width_pixels,
                            height = format.height_pixels,
                        )])
                        .args(&["-density", "900"])
                        // Page background.
                        .arg("xc:white");

                    // Prepare drawing lines.
                    command
                        .args(&["-stroke", "black"])
                        .args(&["-strokewidth", "3"])
                        .args(&["-fill", "none"]);

                    // Draw horizontal lines
                    for i in 0..format.lines {
                        let x0 = 0;
                        let x1 = format.width_pixels;
                        // Draw horizontal line for the top of the image
                        let y = i * image_plus_margin_height +
                            margin_height + padding_top;
                        command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                            y0 = y,
                            y1 = y,
                            x0 = x0,
                            x1 = x1)]);


                        // Draw horizontal line for the bottom of the image
                        let y = (i + 1) * image_plus_margin_height - margin_height - padding_bottom;
                        command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                            y0 = y,
                            y1 = y,
                            x0 = x0,
                            x1 = x1)]);

                    }
                    // Draw vertical lines
                    for i in 0..format.rows {
                        let y0 = 0;
                        let y1 = format.height_pixels;

                        // Draw vertical line for the left of the image
                        let x = i * image_plus_margin_width +
                            margin_width + padding_left;
                        command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                            y0 = y0,
                            y1 = y1,
                            x0 = x,
                            x1 = x)]);

                        // Draw vertical line for the right of the image
                        let x = (i + 1) * image_plus_margin_width - margin_width - padding_right;
                        command.args(&["-draw", &format!("line {x0},{y0} {x1},{y1}",
                            y0 = y0,
                            y1 = y1,
                            x0 = x,
                            x1 = x)]);
                    }

                    let mut sources = group.iter();
                    'grid: for i in 0..format.rows {
                        for j in 0..format.lines {
                            let source = if let Some(source) = sources.next() {
                                source
                            } else {
                                // We've run out of sources.
                                break 'grid
                            };
                            let path = options.dest_max_quality_horizontal_cards
                                .join(format!("{source}.png",
                                    source = source));
                            command.arg(path.into_os_string());

                            let geometry = format!("+{x}+{y}",
                                x = i * image_plus_margin_width + margin_width,
                                y = j * image_plus_margin_height + margin_height);
                            command.args(&["-geometry", &geometry])
                                .arg("-composite");
                        }
                    }
                    command.args(&["-flatten"]);
                    command.args(&["-alpha", "remove"]);
                    command.args(&["-alpha", "off"]);
                    command.arg(&dest);
                    debug!(target: "generate", "Generating print-at-home: {:?}", command);
                    command.spawn()
                        .expect("Could not launch command")
                })
                .collect();
            batch(tasks);
        }

        // Convert into jpg, if necessary
        print!("Converting print-at-home horizontal cards into jpg (if necessary)");
        let tasks = options.formats.iter()
            // Establish list of files to convert
            .flat_map(|format| {
                if let Format::JPG = format.format {
                    // Ok, we need to generate a jpg.
                } else {
                    // Skip this format
                    return vec![].into_iter();
                }
                let images_per_page = format.lines * format.rows;
                let groups = options.cards.iter()
                    .sorted()
                    .into_iter()
                    .chunks(images_per_page);
                let sources : Vec<(_, _)> = groups.into_iter()
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
                        let source = dest_print_at_home_cards
                            .join(format!("{format}-{start}-{stop}.tiff",
                                format = format.name,
                                start = group[0],
                                stop = group[group.len() - 1]));
                        let dest = dest_print_at_home_cards
                            .join(format!("{format}-{start}-{stop}.jpg",
                                format = format.name,
                                start = group[0],
                                stop = group[group.len() - 1]));
                        (
                            source.to_str()
                                .unwrap()
                                .to_string(),
                            dest.to_str()
                                .unwrap()
                                .to_string()
                        )
                    })
                    .collect();
                sources.into_iter()
            })
            .map(|(source, dest)| {
                let mut command = Command::new("convert");
                command.arg(source);
                command.arg(dest);
                command.spawn()
                    .expect("Could not launch command")
            })
            .collect_vec();
        batch(tasks);

        // Convert into a single pdf
        print!("Converting print-at-home horizontal cards into a single high-res pdf (if necessary)");
        let tasks = options.formats.iter()
            .filter_map(|format| {
                if let Format::PDF = format.format {
                    // Ok, we need to generate a pdf.
                } else {
                    // Skip this format
                    return None;
                }
                let images_per_page = format.lines * format.rows;
                let groups = options.cards.iter()
                    .sorted()
                    .into_iter()
                    .chunks(images_per_page);
                let sources = groups.into_iter()
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
                        let source = dest_print_at_home_cards
                            .join(format!("{format}-{start}-{stop}.tiff",
                                format = format.name,
                                start = group[0],
                                stop = group[group.len() - 1]));
                        source.to_str()
                            .unwrap()
                            .to_string()
                    });
                let mut command = Command::new("convert");
                for source in sources {
                    command.arg(source);
                }

                let dest = dest_print_at_home_cards
                    .join(format!("{format}-cards-highres.pdf",
                        format = format.name));
                let dest = dest.to_str()
                    .unwrap();
                command.arg(dest);
                debug!(target: "generate", "Generating high-res print-at-home: {:?}", command);
                Some(command.spawn()
                    .expect("Could not launch command"))
            }).collect();
        batch(tasks);

        print!("Converting print-at-home horizontal cards into a single high-res pdf (if necessary)");
        let tasks = options.formats.iter()
            .filter_map(|format|{
                if let Format::PDF = format.format {
                    // Ok, we need to generate a pdf.
                } else {
                    // Skip this format
                    return None;
                }
                let source = dest_print_at_home_cards
                    .join(format!("{format}-cards-highres.pdf",
                        format = format.name));
                let source = source.to_str()
                    .unwrap();
                let dest = dest_print_at_home_cards
                    .join(format!("{format}-cards.pdf",
                        format = format.name));
                let dest = dest.to_str()
                    .unwrap();
                let mut command = Command::new("gs");
                command.args(&[
                    "-dAutoRotatePages=/None",
                    "-r300",
                    "-sDEVICE=pdfwrite",
                    "-dBATCH",
                    "-dNOPAUSE",
                "-quit"]);
                command.arg(format!("-sOutputFile={}", dest))
                    .arg(source);
                debug!(target: "generate", "Generating destination print-at-home: {:?}", command);
                Some(command.spawn()
                    .expect("Could not launch command"))
            }).collect();
        batch(tasks);
    }
}
