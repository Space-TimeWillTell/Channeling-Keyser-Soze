#![feature(option_replace)]

extern crate plotonomicon;

extern crate clap;
extern crate itertools;
extern crate rand;
#[macro_use]
extern crate smallvec;

use std::collections::HashMap;

use clap::{ Arg, App };
use plotonomicon::*;
use rand::Rng;

struct Options {
    draw_rebalaces: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
enum Side {
    A,
    B
}
impl std::fmt::Display for Side {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Side::*;
        match *self {
            A => write!(fmt, "A"),
            B => write!(fmt, "B"),
        }
    }
}

impl Side {
    fn rev(self) -> Self {
        match self {
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
enum State {
    Balanced,
    Unbalanced(Side),
    Overwhelming(Side),
    Victory(Side),
}
impl std::fmt::Display for State {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use State::*;
        match *self {
            Balanced => write!(fmt, "Balanced"),
            Unbalanced(side) => write!(fmt, "Advantage: {}", side),
            Overwhelming(side) => write!(fmt, "Overwhelming: {}", side),
            Victory(side) => write!(fmt, "Victory: {}", side),
        }
    }
}

impl State {
    pub fn shift(self, towards: Option<Side>) -> Self {
        match (self, towards) {
            (State::Balanced, None) =>
                State::Balanced,
            (State::Balanced, Some(side)) =>
                State::Unbalanced(side),
            (State::Unbalanced(_), None) =>
                State::Balanced,
            (State::Unbalanced(side), Some(side_)) =>
                if side == side_ {
                    State::Overwhelming(side)
                } else {
                    State::Balanced
                }
            (State::Overwhelming(side), None) =>
                State::Unbalanced(side),
            (State::Overwhelming(side), Some(side_)) =>
                if side == side_ {
                    State::Victory(side)
                } else {
                    State::Unbalanced(side)
                }
            (State::Victory(side), _) =>
                State::Victory(side)
        }
    }
    pub fn next<R: Rng>(self, rng: &mut R, deck: &mut Deck, options: &Options) -> Self {
        let (over, under, who) = match self {
            State::Balanced => (smallvec![deck.next(rng)], smallvec![deck.next(rng)], Side::A /* Arbitrary */),
            State::Unbalanced(side) => (smallvec![deck.next(rng), deck.next(rng)], smallvec![deck.next(rng)], side),
            State::Overwhelming(side) => (smallvec![deck.next(rng), deck.next(rng)], smallvec![deck.next(rng), deck.next(rng)], side),
            State::Victory(side) => return State::Victory(side),
        };
        assert!(under.len() <= over.len());
        match over.beats(&under) {
            Comparison::Excuse => self.shift(None),
            Comparison::Win => self.shift(Some(who)),
            Comparison::Lose => self.shift(Some(who.rev())),
            Comparison::Draw if options.draw_rebalaces => self.shift(None),
            Comparison::Draw => self,
        }
    }
}

fn main() {
    let matches = App::new("Conflict resolution simulator")
        .author("David Teller")
        .arg(Arg::with_name("rounds")
            .long("rounds")
            .required(true)
            .takes_value(true)
            .validator(|s| match s.parse::<u32>() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e))
            })
            .help("Number of rounds"))
        .arg(Arg::with_name("samples")
            .long("samples")
            .required(true)
            .takes_value(true)
            .validator(|s| match s.parse::<u32>() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e))
            })
            .help("Number of samples"))
        .arg(Arg::with_name("draw_rebalances")
            .long("draw_rebalances")
            .required(true)
            .default_value("false")
            .takes_value(true)
            .validator(|s| match s.parse::<bool>() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e))
            })
            .help("If true, a Draw shifts towards balance. Otherwise, Draw is a draw"))
        .get_matches();

    let rounds = matches.value_of("rounds")
        .unwrap()
        .parse::<u32>()
        .unwrap();

    let samples = matches.value_of("samples")
        .unwrap()
        .parse::<u32>()
        .unwrap();

    let options = Options {
        draw_rebalaces: matches.value_of("draw_rebalances")
            .unwrap()
            .parse::<bool>()
            .unwrap(),
    };

    let mut rng = rand::thread_rng();

    let mut results = HashMap::new();

    for _ in 0..samples {
        for initial_state in &[State::Balanced, State::Unbalanced(Side::A), State::Overwhelming(Side::A)] {
            let mut state = initial_state.clone();

            let mut deck = Deck::shuffle(&mut rng);
            for _ in 0..rounds {
                state = state.next(&mut rng, &mut deck, &options);
                if let State::Victory(_) = state {
                    break;
                }
            }

            results.entry((initial_state.clone(), state))
                .and_modify(|instances| *instances += 1)
                .or_insert(0);
        }
    }

    for initial_state in &[State::Balanced, State::Unbalanced(Side::A), State::Overwhelming(Side::A)] {
        for final_state in &[
            State::Victory(Side::A), State::Overwhelming(Side::A), State::Unbalanced(Side::A),
            State::Balanced,
            State::Unbalanced(Side::B), State::Overwhelming(Side::B), State::Victory(Side::B)]
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