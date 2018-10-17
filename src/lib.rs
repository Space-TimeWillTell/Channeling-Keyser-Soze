#![feature(option_replace)]

extern crate itertools;
extern crate rand;
extern crate smallvec;


use itertools::Itertools;
use rand::Rng;
use smallvec::SmallVec;

pub trait Beats {
    fn beats(&self, other: &Self) -> Comparison;
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Suit {
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
pub enum Role {
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
pub enum Card {
    Excuse,
    Trump(u8 /* 1 - 21 */),
    Color(Suit, Role),
}

#[derive(PartialEq, Eq, Debug)]
pub enum Comparison {
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

pub struct Deck(Vec<Card>);

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
