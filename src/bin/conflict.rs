#![feature(option_replace)]

extern crate plotonomicon;

extern crate clap;
extern crate itertools;
extern crate rand;

use clap::{ Arg, App };
use itertools::Itertools;
use plotonomicon::*;

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
fn main() {
    let matches = App::new("Conflict draw")
        .author("David Teller")
        .arg(Arg::with_name("balance")
            .long("balance")
            .required(true)
            .takes_value(true)
            .possible_values(&["=", "A", "AA", "B", "BB"])
            .help("Starting situation"))
        .get_matches();

    let mut rng = rand::thread_rng();

    let start = match matches.value_of("balance").unwrap() {
        "=" => ThreeStateBalance::Balanced,
        "A" => ThreeStateBalance::Unbalanced(Side::A),
        "B" => ThreeStateBalance::Unbalanced(Side::B),
        "AA" => ThreeStateBalance::Overwhelming(Side::A),
        "BB" => ThreeStateBalance::Overwhelming(Side::B),
        _ => unreachable!()
    };
    println!("Start: {}", start);
    let state = State::new(&mut rng, start);

    for step in state {
        wait_for_keypress();

        println!("A draws: {}", step.a().iter().format(", "));
        println!("B draw: {}", step.b().iter().format(", "));
        // FIXME: Visually display cards for A and B.

        match step.winner() {
            Some(side) => println!("Round winner: {}", side),
            _ => println!("Draw!")
        }
        println!("{}", step.stop());
    }
}