#![feature(option_replace)]

extern crate clap;
extern crate itertools;
extern crate rand;
#[macro_use]
extern crate smallvec;

use std::collections::HashMap;

use clap::{ Arg, App };
use itertools::Itertools;
use rand::Rng;
use smallvec::SmallVec;

struct Options {
    draw_rebalaces: bool,
}

trait Beats {
    fn beats(&self, other: &Self) -> Comparison;
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Suit {
    Rock,
    Paper,
    Blades,
    Lizard,
    OutThere,
}
impl Suit {
    #[cfg(test)]
    fn suits() -> [Suit; 5] {
        use Suit::*;
        [Rock, Paper, Blades, Lizard, OutThere]
    }
}
impl Beats for Suit {
    fn beats(&self, other: &Self) -> Comparison {
        use Suit::*;
        match (*self, *other) {
            (Rock, Rock) => Comparison::Draw,
            (Rock, Paper) | (Rock, OutThere) => Comparison::Lose,
            (Rock, Blades) | (Rock, Lizard )=> Comparison::Win,
            (Paper, Paper) => Comparison::Draw,
            (Paper, Blades) | (Paper, Lizard) => Comparison::Lose,
            (Paper, Rock) | (Paper, OutThere) => Comparison::Win,
            (Blades, Blades) => Comparison::Draw,
            (Blades, Rock) | (Blades, OutThere) => Comparison::Lose,
            (Blades, Paper) | (Blades, Lizard) => Comparison::Win,
            (Lizard, Lizard) => Comparison::Draw,
            (Lizard, Rock) | (Lizard, Blades) => Comparison::Lose,
            (Lizard, OutThere) | (Lizard, Paper) => Comparison::Win,
            (OutThere, OutThere) => Comparison::Draw,
            (OutThere, Paper) | (OutThere, Lizard) => Comparison::Lose,
            (OutThere, Rock) | (OutThere, Blades) => Comparison::Win,
        }
    }
}

#[test]
fn suit_beats() {
    for left in &Suit::suits() {
        for right in &Suit::suits() {
            assert_eq!(left.beats(right), right.beats(left).rev());
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Role {
    Ruler,
    Dragon,
    Treasure,
    Builder,
    Seeker,
    Soldier,
    Lover,
    Servant,
    Home,
}

#[derive(PartialEq, Eq)]
enum Card {
    Excuse,
    Trump(u8 /* 1 - 21 */),
    Color(Suit, Role),
}

#[derive(PartialEq, Eq, Debug)]
enum Comparison {
    Excuse,
    Win,
    Lose,
    Draw,
}
impl Comparison {
    #[cfg(test)]
    fn rev(self) -> Self {
        use Comparison::*;
        match self {
            Excuse => Excuse,
            Win => Lose,
            Lose => Win,
            Draw => Draw,
        }
    }
}

impl Beats for Card {
    fn beats(&self, other: &Card) -> Comparison {
        debug_assert!(self != other);
        match (self, other) {
            (&Card::Excuse, _) | (_, &Card::Excuse) => Comparison::Excuse,
            (&Card::Trump(ref me), &Card::Trump(ref them)) => {
                if *me < *them {
                    Comparison::Lose
                } else if *me > *them {
                    Comparison::Win
                } else {
                    Comparison::Draw
                }
            }
            (&Card::Trump(_), _) => Comparison::Win,
            (_, &Card::Trump(_)) => Comparison::Lose,
            (&Card::Color(ref me, _), &Card::Color(ref them, _)) =>
                me.beats(them)
        }
    }
}

impl Beats for SmallVec<[Card; 2]> {
    // This assumes that `self` has the advantage.
    fn beats(&self, other: &Self) -> Comparison {
        let mut best = None; // Placeholder.
        let product = self.iter()
            .cartesian_product(other.iter());
        for (mine, theirs) in product {
            match mine.beats(theirs) {
                Comparison::Excuse => return Comparison::Excuse,
                Comparison::Win => return Comparison::Win,
                Comparison::Lose => {
                    if best.is_none() {
                        best.replace(Comparison::Lose);
                    }
                }
                Comparison::Draw => {
                    best.replace(Comparison::Draw);
                }
            }
        }
        best.unwrap()
    }
}

struct Deck(Vec<Card>);

impl Deck {
    pub fn shuffle<R: Rng>(rng: &mut R) -> Self {
        let mut deck = vec![];
        // Excuse
        deck.push(Card::Excuse);
        // Trump
        for i in 1..22 {
            deck.push(Card::Trump(i));
        }
        // Each suit
        for suit in [Suit::Rock, Suit::Paper, Suit::Blades, Suit::Lizard, Suit::OutThere].iter() {
            for role in [Role::Ruler, Role::Dragon, Role::Treasure,
                Role::Builder, Role::Seeker, Role::Soldier,
                Role::Lover, Role::Servant, Role::Home].iter()
            {
                deck.push(Card::Color(*suit, *role));
            }
        }
        assert_eq!(deck.len(), 67);

        // Now shuffle
        rng.shuffle(deck.as_mut_slice());
        Deck(deck)
    }

    pub fn next<R: Rng>(&mut self, rng: &mut R) -> Card {
        if let Some(result) = self.0.pop() {
            return result
        }
        // Otherwise, the deck is empty. Reshuffle.
        *self = Self::shuffle(rng);
        self.0.pop()
            .unwrap() // We just reshuffled, it can't be empty.
    }
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