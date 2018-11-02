extern crate plotonomicon;

extern crate clap;
extern crate rand;

use plotonomicon::*;

use clap::{ Arg, App };

fn wait_for_keypress() {
    use std::io::Write;

    print!("\nPress RETURN to continue...");
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    stdout.flush()
        .expect("Can't flush");
    stdin.read_line(&mut String::new())
        .expect("Could not read");
}

enum Kind {
    Triumph,
    Figure,
    Any
}

impl Kind {
    fn contains(&self, card: &Card) -> bool {
        match (self, card) {
            (&Kind::Any, _) => true,
            (&Kind::Triumph, &Card::Trump(_)) => true,
            (&Kind::Figure, &Card::Color(_, _)) => true,
            _ => false,
        }
    }
}

fn main() {
    let matches = App::new("Draw cards")
        .author("David Teller")
        .arg(Arg::with_name("kind")
            .long("kind")
            .takes_value(true)
            .possible_values(&["triumph", "figure", "any"])
            .default_value("any")
        )
        .get_matches();

    let kind = match matches.value_of("kind").unwrap() {
        "any" => Kind::Any,
        "triumph" => Kind::Triumph,
        "figure" => Kind::Figure,
        _ => unreachable!()
    };

    let mut rng = rand::thread_rng();
    let mut deck = Deck::shuffle(&mut rng);
    loop {
        let draw = deck.next(&mut rng);
        if !kind.contains(draw.card()) {
            continue;
        }
        println!(">>> {} (card {})", draw, draw.index());
        wait_for_keypress();
    }
}