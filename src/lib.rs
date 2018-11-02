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
    Serpent,
    Unknown,
}
impl Display for Suit {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        use Suit::*;
        let name = match *self {
            Rock => "Rock",
            Paper => "Paper",
            Blades => "Blade",
            Serpent => "Serpent",
            Unknown => "Unknown",
        };
        name.fmt(fmt)
    }
}

impl Suit {
    #[cfg(test)]
    fn suits() -> [Suit; 5] {
        use Suit::*;
        [Rock, Paper, Blades, Serpent, Unknown]
    }
}
impl Beats for Suit {
    fn beats(&self, other: &Self) -> Comparison {
        use Suit::*;
        match (*self, *other) {
            (Rock, Rock) => Comparison::Draw,
            (Rock, Paper) | (Rock, Unknown) => Comparison::Lose,
            (Rock, Blades) | (Rock, Serpent )=> Comparison::Win,
            (Paper, Paper) => Comparison::Draw,
            (Paper, Blades) | (Paper, Serpent) => Comparison::Lose,
            (Paper, Rock) | (Paper, Unknown) => Comparison::Win,
            (Blades, Blades) => Comparison::Draw,
            (Blades, Rock) | (Blades, Unknown) => Comparison::Lose,
            (Blades, Paper) | (Blades, Serpent) => Comparison::Win,
            (Serpent, Serpent) => Comparison::Draw,
            (Serpent, Rock) | (Serpent, Blades) => Comparison::Lose,
            (Serpent, Unknown) | (Serpent, Paper) => Comparison::Win,
            (Unknown, Unknown) => Comparison::Draw,
            (Unknown, Paper) | (Unknown, Serpent) => Comparison::Lose,
            (Unknown, Rock) | (Unknown, Blades) => Comparison::Win,
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


#[derive(PartialEq, Eq, Debug)]
pub enum Comparison {
    Excuse,
    Win,
    Lose,
    Draw,
}
impl Comparison {
    pub fn loses(&self) -> bool {
        match self {
            &Comparison::Lose => true,
            _ => false
        }
    }

    pub fn wins(&self) -> bool {
        match self {
            &Comparison::Win => true,
            _ => false
        }
    }

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
        for suit in [Suit::Rock, Suit::Paper, Suit::Blades, Suit::Serpent, Suit::Unknown].iter() {
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

    pub fn next_card<R: Rng>(&mut self, rng: &mut R) -> Card {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
impl Orientation {
    pub fn is_reversed(&self) -> bool {
        match *self {
            Orientation::Up => false,
            Orientation::Reversed => true,
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

/// An implementation of Balance for the three-state rules.
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum ThreeStateBalance {
    Balanced,
    Unbalanced(Side),
    Overwhelming(Side),
    Victory(Side),
}
impl Display for ThreeStateBalance {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), FmtError> {
        use ThreeStateBalance::*;
        match *self {
            Balanced => write!(fmt, "Balanced"),
            Unbalanced(side) => write!(fmt, "Advantage: {}", side),
            Overwhelming(side) => write!(fmt, "Overwhelming: {}", side),
            Victory(side) => write!(fmt, "Victory: {}", side),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
struct Losses {
    temporary: usize,
    serious: usize,
    left: usize,
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct BiddingBalance {
    losses: [Losses; 2],
    advantage: BiddingAdvantage,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
/// An inplementation of Balance for the Bidding rules.
pub enum BiddingAdvantage {
    Balanced,
    Unbalanced(Side, /*non-0*/usize),
    Victory(Side),
}
impl Display for BiddingAdvantage {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), FmtError> {
        use BiddingAdvantage::*;
        match *self {
            Balanced => write!(fmt, "Balanced"),
            Unbalanced(side, level) => write!(fmt, "Advantage {}: {}", level, side),
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
    pub fn index(&self) -> String {
        let num = match self.card {
            Card::Excuse => 5,
            Card::Trump(n) => 5 + n,
            Card::Color(suit, figure) => {
                let start = match suit {
                    Suit::Paper => 32,
                    Suit::Unknown => 43,
                    Suit::Rock => 54,
                    Suit::Blades => 65,
                    Suit::Serpent => 76,
                };
                let offset = match figure {
                    Role::Ruler => 0,
                    Role::Dragon => 1,
                    Role::Treasure => 2,
                    Role::Soldier => 3,
                    Role::Builder => 4,
                    Role::Seeker => 5,
                    Role::Lover => 6,
                    Role::Servant => 7,
                    Role::Home => 8,
                };
                start + offset
            }
        };
        let orientation = match self.orientation {
            Orientation::Up => "+",
            Orientation::Reversed => "-",
        };
        format!("{}{}", num, orientation)
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

pub struct State<'a, B, R: Rng> {
    balance: B,
    deck: Deck,
    rng: &'a mut R,
}
impl<'a, B, R> State<'a, B, R> where R: Rng {
    pub fn new(rng: &'a mut R, balance: B) -> Self {
        let deck = Deck::shuffle(rng);
        State {
            balance,
            deck,
            rng
        }
    }
}

pub struct Step<B> {
    start: B,
    stop: B,
    a: Draw,
    b: Draw,
    winner: Option<Side>,
}
impl<B> Step<B> {
    pub fn start(&self) -> &B {
        &self.start
    }
    pub fn stop(&self) -> &B {
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

impl<'a, R> Iterator for State<'a, ThreeStateBalance, R> where R: Rng {
    type Item = Step<ThreeStateBalance>;
    fn next(&mut self) -> Option<Self::Item> {
        let balance = self.balance;
        let (over, under, who) = match balance {
            ThreeStateBalance::Balanced => (smallvec![self.deck.next(self.rng)], smallvec![self.deck.next(self.rng)], Side::A /* Arbitrary */),
            ThreeStateBalance::Unbalanced(side) => (smallvec![self.deck.next(self.rng), self.deck.next(self.rng)], smallvec![self.deck.next(self.rng)], side),
            ThreeStateBalance::Overwhelming(side) => (smallvec![self.deck.next(self.rng), self.deck.next(self.rng)], smallvec![self.deck.next(self.rng), self.deck.next(self.rng)], side),
            ThreeStateBalance::Victory(_) => return None
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

impl ThreeStateBalance {
    fn shift(self, towards: Option<Side>) -> Self {
        match (self, towards) {
            (ThreeStateBalance::Balanced, None) =>
                ThreeStateBalance::Balanced,
            (ThreeStateBalance::Balanced, Some(side)) =>
                ThreeStateBalance::Unbalanced(side),
            (ThreeStateBalance::Unbalanced(_), None) =>
                ThreeStateBalance::Balanced,
            (ThreeStateBalance::Unbalanced(side), Some(side_)) =>
                if side == side_ {
                    ThreeStateBalance::Overwhelming(side)
                } else {
                    ThreeStateBalance::Balanced
                }
            (ThreeStateBalance::Overwhelming(side), None) =>
                ThreeStateBalance::Unbalanced(side),
            (ThreeStateBalance::Overwhelming(side), Some(side_)) =>
                if side == side_ {
                    ThreeStateBalance::Victory(side)
                } else {
                    ThreeStateBalance::Unbalanced(side)
                }
            (ThreeStateBalance::Victory(side), _) =>
                ThreeStateBalance::Victory(side)
        }
    }
}

impl<'a, R> Iterator for State<'a, BiddingBalance, R> where R: Rng {
    type Item = Step<BiddingBalance>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut cards_dominant: SmallVec<[Card; 10]> = smallvec![];
        let mut cards_dominated: SmallVec<[Card; 10]> = smallvec![];
        loop {

            // FIXME: Dominant draws until it beats Dominated or is about to be elimited.
            // FIXME: If Dominant is about to be eliminated, it Concedes the Bid.
            // FIXME: Dominated draws until it beats Dominant or is about to be eliminated.
            // FIXME: If Dominated is about to be eliminated, it Concedes the Bid.
            // FIXME: We should try several strategies. Maybe later :)
        }
        unimplemented!()
    }
}

impl BiddingAdvantage {
    fn shift(self, towards: Option<Side>) -> Self {
        match (self, towards) {
            (BiddingAdvantage::Balanced, None) =>
                BiddingAdvantage::Balanced,
            (BiddingAdvantage::Balanced, Some(side)) =>
                BiddingAdvantage::Unbalanced(side, 1),
            (BiddingAdvantage::Unbalanced(_, _), None) =>
                BiddingAdvantage::Balanced,
            (BiddingAdvantage::Unbalanced(side, level), Some(side_)) =>
                if side == side_ {
                    BiddingAdvantage::Unbalanced(side, level + 1)
                } else if level > 1 {
                    BiddingAdvantage::Unbalanced(side, level - 1)
                } else {
                    BiddingAdvantage::Balanced
                }
            (BiddingAdvantage::Victory(side), _) =>
                BiddingAdvantage::Victory(side)
        }
    }
}
