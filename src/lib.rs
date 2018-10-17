#![feature(option_replace)]

extern crate xvii;
extern crate itertools;
extern crate rand;
#[macro_use]
extern crate smallvec;

use std::fmt::{ Display, Error as FmtError, Formatter };
use itertools::Itertools;
use rand::Rng;
use xvii::Roman;

use smallvec::SmallVec;

pub type Draw = SmallVec<[DrawnCard;2]>;

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
impl Display for Suit {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        use Suit::*;
        let name = match *self {
            Rock => "Rock",
            Paper => "Paper",
            Blades => "Blade",
            Lizard => "Serpent",
            OutThere => "OutThere",
        };
        name.fmt(fmt)
    }
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
impl Display for Role {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        use Role::*;
        let name = match *self {
            Ruler => "Ruler",
            Dragon => "Dragon",
            Treasure => "Treasure",
            Builder => "Builder",
            Seeker => "Seeker",
            Soldier => "Soldier",
            Servant => "Servant",
            Lover => "Lover",
            Home => "Home",
        };
        name.fmt(fmt)
    }
}
#[derive(PartialEq, Eq)]
pub enum Card {
    Excuse,
    Trump(u8 /* 1 - 21 */),
    Color(Suit, Role),
}

/*
const TRUMPS : [&'static str; 21] = [

];

impl Display for Card {
    fn fmt<F: Formatter>(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        use Card::*;
        match *self {
            Excuse => write!(fmt, "Excuse"),
            Trump(n) => write!(fmt, TRUMPS[i]),
        }
    }
}
*/
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
impl Display for Card {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        use Card::*;
        match *self {
            Excuse => write!(fmt, "Excuse"),
            Trump(i) => write!(fmt, "{} of Triumph", Roman::from(i as i32).unwrap()),
            Color(suit, role) => write!(fmt, "{role} of {suit}",
                role = role,
                suit = suit,
            )
        }
    }
}

impl Beats for Draw {
    fn beats(&self, other: &Self) -> Comparison {
        let mut best = None; // Placeholder.
        let product = self.iter()
            .cartesian_product(other.iter());
        for (mine, theirs) in product {
            match mine.card.beats(&theirs.card) {
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

    fn next_card<R: Rng>(&mut self, rng: &mut R) -> Card {
        if let Some(result) = self.0.pop() {
            return result
        }
        // Otherwise, the deck is empty. Reshuffle.
        *self = Self::shuffle(rng);
        self.0.pop()
            .unwrap() // We just reshuffled, it can't be empty.
    }

    pub fn next<R: Rng>(&mut self, rng: &mut R) -> DrawnCard {
        let orientation = rng.choose(&[Orientation::Up, Orientation::Reversed])
            .unwrap() // The arra is not empty, so can't be None.
            .clone();
        let card = self.next_card(rng);
        DrawnCard {
            orientation,
            card
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Orientation {
    Up,
    Reversed
}
impl Display for Orientation {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), FmtError> {
        use Orientation::*;
        match *self {
            Up => write!(fmt, "Up"),
            Reversed => write!(fmt, "Reversed"),
        }
    }
}


#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum Side {
    A,
    B
}
impl Display for Side {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), FmtError> {
        use Side::*;
        match *self {
            A => write!(fmt, "A"),
            B => write!(fmt, "B"),
        }
    }
}

impl Side {
    pub fn rev(self) -> Self {
        match self {
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum Balance {
    Balanced,
    Unbalanced(Side),
    Overwhelming(Side),
    Victory(Side),
}
impl Display for Balance {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), FmtError> {
        use Balance::*;
        match *self {
            Balanced => write!(fmt, "Balanced"),
            Unbalanced(side) => write!(fmt, "Advantage: {}", side),
            Overwhelming(side) => write!(fmt, "Overwhelming: {}", side),
            Victory(side) => write!(fmt, "Victory: {}", side),
        }
    }
}

pub struct DrawnCard {
    orientation: Orientation,
    card: Card,
}
impl DrawnCard {
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
    pub fn card(&self) -> &Card {
        &self.card
    }
}
impl Display for DrawnCard {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        match self.orientation() {
            Orientation::Up => self.card.fmt(fmt),
            Orientation::Reversed => {
                write!(fmt, "{} (Reversed)", self.card)
            }
        }
    }
}

pub struct State<'a, R: Rng> {
    balance: Balance,
    deck: Deck,
    rng: &'a mut R,
}
impl<'a, R> State<'a, R> where R: Rng {
    pub fn new(rng: &'a mut R, balance: Balance) -> Self {
        let deck = Deck::shuffle(rng);
        State {
            balance,
            deck,
            rng
        }
    }
}

pub struct Step {
    start: Balance,
    stop: Balance,
    a: Draw,
    b: Draw,
    winner: Option<Side>,
}
impl Step {
    pub fn start(&self) -> &Balance {
        &self.start
    }
    pub fn stop(&self) -> &Balance {
        &self.stop
    }
    pub fn a(&self) -> &Draw {
        &self.a
    }
    pub fn b(&self) -> &Draw {
        &self.b
    }
    pub fn winner(&self) -> Option<Side> {
        self.winner
    }
}

impl<'a, R> Iterator for State<'a, R> where R: Rng {
    type Item = Step;
    fn next(&mut self) -> Option<Self::Item> {
        let balance = self.balance;
        let (over, under, who) = match balance {
            Balance::Balanced => (smallvec![self.deck.next(self.rng)], smallvec![self.deck.next(self.rng)], Side::A /* Arbitrary */),
            Balance::Unbalanced(side) => (smallvec![self.deck.next(self.rng), self.deck.next(self.rng)], smallvec![self.deck.next(self.rng)], side),
            Balance::Overwhelming(side) => (smallvec![self.deck.next(self.rng), self.deck.next(self.rng)], smallvec![self.deck.next(self.rng), self.deck.next(self.rng)], side),
            Balance::Victory(_) => return None
        };
        let (winner, new_balance) = match over.beats(&under) {
            Comparison::Excuse => (Some(who.rev()), balance.shift(None)),
            Comparison::Win => (Some(who), balance.shift(Some(who))),
            Comparison::Lose => (Some(who.rev()), balance.shift(Some(who.rev()))),
            Comparison::Draw => (None, balance),
        };
        self.balance = new_balance;
        let (a, b) = match who {
            Side::A => (over, under),
            Side::B => (under, over),
        };
        Some(Step {
            start: balance,
            stop: new_balance,
            a,
            b,
            winner
        })
    }
}

impl Balance {
    fn shift(self, towards: Option<Side>) -> Self {
        match (self, towards) {
            (Balance::Balanced, None) =>
                Balance::Balanced,
            (Balance::Balanced, Some(side)) =>
                Balance::Unbalanced(side),
            (Balance::Unbalanced(_), None) =>
                Balance::Balanced,
            (Balance::Unbalanced(side), Some(side_)) =>
                if side == side_ {
                    Balance::Overwhelming(side)
                } else {
                    Balance::Balanced
                }
            (Balance::Overwhelming(side), None) =>
                Balance::Unbalanced(side),
            (Balance::Overwhelming(side), Some(side_)) =>
                if side == side_ {
                    Balance::Victory(side)
                } else {
                    Balance::Unbalanced(side)
                }
            (Balance::Victory(side), _) =>
                Balance::Victory(side)
        }
    }
}

