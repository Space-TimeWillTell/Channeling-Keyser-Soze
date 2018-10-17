#![feature(option_replace)]

extern crate plotonomicon;

extern crate clap;
extern crate itertools;
extern crate rand;

use std::collections::HashMap;

use clap::{ Arg, App };
use plotonomicon::*;


fn main() {
    let matches = App::new("Conflict resolution simulator")
        .author("David Teller")
        .arg(Arg::with_name("rounds")
            .long("rounds")
            .required(true)
            .takes_value(true)
            .validator(|s| match s.parse::<usize>() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e))
            })
            .help("Number of rounds"))
        .arg(Arg::with_name("samples")
            .long("samples")
            .required(true)
            .takes_value(true)
            .validator(|s| match s.parse::<usize>() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e))
            })
            .help("Number of samples"))
        .get_matches();

    let rounds = matches.value_of("rounds")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let samples = matches.value_of("samples")
        .unwrap()
        .parse::<usize>()
        .unwrap();


    let mut rng = rand::thread_rng();

    let mut results = HashMap::new();

    for _ in 0..samples {
        for initial_state in &[Balance::Balanced, Balance::Unbalanced(Side::A), Balance::Overwhelming(Side::A)] {
            let state = State::new(&mut rng, initial_state.clone());
            let mut balance = initial_state.clone();

            for (i, step) in state.enumerate() {
                balance = step.stop().clone();
                if i > rounds {
                    break;
                }
            }

            results.entry((initial_state.clone(), balance))
                .and_modify(|instances| *instances += 1)
                .or_insert(0);
        }
    }

    for initial_state in &[Balance::Balanced, Balance::Unbalanced(Side::A), Balance::Overwhelming(Side::A)] {
        for final_state in &[
            Balance::Victory(Side::A), Balance::Overwhelming(Side::A), Balance::Unbalanced(Side::A),
            Balance::Balanced,
            Balance::Unbalanced(Side::B), Balance::Overwhelming(Side::B), Balance::Victory(Side::B)]
        {
            let count = results.get(&(initial_state.clone(), final_state.clone()))
                .cloned()
                .unwrap_or(0);

            println!("{initial} => {terminal}: {count:.1}%",
                initial = initial_state,
                terminal = final_state,
                count = ((count as f64) / (samples as f64)) * 100.);
        }
    }
}