#![feature(option_replace)]

extern crate plotonomicon;

extern crate clap;
extern crate rand;
extern crate smallvec;

use std::collections::HashMap;

use clap::{ Arg, App };
use plotonomicon::*;
use rand::Rng;
use smallvec::*;

const MAX_DIFFICULTY: usize = 10;
const MAX_LOSSES: usize = 5;

struct Options {
    samples: usize,
    rounds: usize,
    difficulty: Option<usize>,
    skill: Option<usize>,
}

/// Test with the no-damage 3 state rules
fn test_nodamage<R: Rng>(rng: &mut R, options: Options) {
    use ThreeStateBalance as Balance;
    let mut results = HashMap::new();

    for _ in 0..options.samples {
        for initial_state in &[Balance::Balanced, Balance::Unbalanced(Side::A), Balance::Overwhelming(Side::A)] {
            let state = State::new(rng, initial_state.clone());
            let mut balance = initial_state.clone();

            for (i, step) in state.enumerate() {
                balance = step.stop().clone();
                if i > options.rounds {
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
                count = ((count as f64) / (options.samples as f64)) * 100.);
        }
    }
}

/// Test with the blackjack rules
fn test_blackjack<R: Rng>(_rng: &mut R, _options: Options) {
    unimplemented!()
}

/// Test Overcoming an Obstacle
fn test_overcome<R: Rng>(rng: &mut R, options: Options) {
    let mut deck = Deck::shuffle(rng);
    let mut results = HashMap::new();
    'sample: for _ in 0..options.samples {

        // Fill the hand of the Obstacle.
        let mut obstacle: SmallVec<[DrawnCard; MAX_DIFFICULTY]> = smallvec![];
        for _ in 0..options.difficulty.unwrap() {
            let card = deck.next(rng);
            if let Card::Excuse = card.card() {
                continue 'sample;
            }
            obstacle.push(card)
        }

        // Test results.
        let mut reversed = 0;
        for _ in 0..options.rounds {
            let draw = deck.next(rng);
            if let Orientation::Reversed = draw.orientation() {
                reversed += 1;
            }
            if reversed >= options.skill.unwrap() + MAX_LOSSES {
                // We have lost, badly.
                break;
            }
            if let &Card::Excuse = draw.card() {
                break;
            }
            obstacle.retain(|obstacle| {
                // Can't beat a card with a different orientation.
                if draw.orientation() != obstacle.orientation() {
                    return true;
                }

                // Keep cards that we can't beat.
                draw.card()
                    .beats(obstacle.card())
                    .loses()
            });
            if obstacle.len() == 0 {
                // We have won.
                break;
            }
        }
        let damage = if reversed >= options.skill.unwrap() {
            reversed - options.skill.unwrap()
        } else {
            0
        };
        let result = (obstacle.len() == 0, damage);
        results.entry(result)
            .and_modify(|instances| *instances += 1)
            .or_insert(1);
    }

    // Display results.
    for win in &[true, false] {
        for damage in 0..6 {
            let instances = results.get(&(*win, damage))
                .unwrap_or(&0);
            println!("{win} with {damage} consequences: {frequency:2}%",
                win = if *win { "win" } else { "lose" },
                damage = damage,
                frequency = (*instances as f64 * 100. )/ options.samples as f64);
        }
    }
}


/// Test Overcoming an Obstacle
fn test_overcome_2<R: Rng>(rng: &mut R, options: Options) {
    let mut deck = Deck::shuffle(rng);
    let mut results = HashMap::new();
    for _ in 0..options.samples {

        // Fill the hand of the Obstacle.
        let mut obstacle: SmallVec<[DrawnCard; MAX_DIFFICULTY]> = smallvec![];
        for _ in 0..options.difficulty.unwrap() {
            let card = deck.next(rng);
            if let Card::Excuse = card.card() {
                // The excuse showed up, stop drawing!
                break;
            }
            obstacle.push(card)
        }

        // Count the number of cards already reversed.
        let mut reversed = obstacle.iter()
            .filter(|card| card.orientation().is_reversed())
            .count();
        while obstacle.len() > 0 {
            let draw = deck.next(rng);
            if let &Card::Excuse = draw.card() {
                // The excuse showed up, you lost!
                break;
            }
            if let Orientation::Reversed = draw.orientation() {
                reversed += 1;
            }
            if reversed >= options.skill.unwrap() + MAX_LOSSES {
                // Wipe Out!
                break;
            }
            obstacle.retain(|obstacle| {
                // Keep cards that we can't beat.
                draw.card()
                    .beats(obstacle.card())
                    .loses()
            });
        }
        let damage = if reversed >= options.skill.unwrap() {
            usize::min(reversed - options.skill.unwrap(), MAX_LOSSES)
        } else {
            0
        };
        let result = (obstacle.len() == 0, damage);
        results.entry(result)
            .and_modify(|instances| *instances += 1)
            .or_insert(1);
    }

    // Display results.
    let mut wins_flawlessly = 0;
    let mut wins_with_consequences = 0;
    let mut losses = 0;
    let mut all = 0;
    for win in &[true, false] {
        for damage in 0..MAX_LOSSES+1 {
            let instances = results.get(&(*win, damage))
                .cloned()
                .unwrap_or(0);
            if instances == 0 {
                continue;
            }
            all += instances;
            if *win {
                if damage == 0 {
                    wins_flawlessly += instances;
                } else {
                    wins_with_consequences += instances;
                }
            } else {
                losses += instances;
            }
            println!("{win} with {damage} consequences: {instances} ({frequency:2}%)",
                win = if *win { "win" } else { "lose" },
                damage = damage,
                instances = instances,
                frequency = (instances as f64 * 100. )/ options.samples as f64);
        }
    }

    println!("Total: {wins_flawlessly:.2}% perfect wins vs {wins_with_consequences:.2}% imperfect wins vs {losses:.2}% losses",
        wins_flawlessly = (wins_flawlessly as f64 * 100.) / (all as f64),
        wins_with_consequences = (wins_with_consequences as f64 * 100.) / (all as f64),
        losses = (losses as f64 * 100.) / (all as f64),
    );
}

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
        .arg(Arg::with_name("rules")
            .long("rules")
            .required(true)
            .takes_value(true)
            .default_value("blackjack")
            .possible_values(&["blackjack", "nodamage", "overcome", "overcome2"])
            .help("Rules to test")
        )
        .arg(Arg::with_name("difficulty")
            .long("difficulty")
            .takes_value(true)
            .validator(|s| match s.parse::<usize>() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e))
            })
            .required_if("rules", "overcome")
            .help("Difficulty of the overcome")
        )
        .arg(Arg::with_name("skill")
            .long("skill")
            .takes_value(true)
            .validator(|s| match s.parse::<usize>() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e))
            })
            .required_if("rules", "overcome")
            .help("Skill when overcoming")
        )
        .get_matches();

    let rounds = matches.value_of("rounds")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let samples = matches.value_of("samples")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let difficulty = match matches.value_of("difficulty") {
        None => None,
        Some(s) => Some(s.parse().unwrap())
    };

    let skill = match matches.value_of("skill") {
        None => None,
        Some(s) => Some(s.parse().unwrap())
    };

    let options = Options {
        rounds,
        samples,
        difficulty,
        skill,
    };
    let mut rng = rand::thread_rng();

    match matches.value_of("rules").unwrap() {
        "blackjack" => test_blackjack(&mut rng, options),
        "nodamage"  => test_nodamage(&mut rng, options),
        "overcome" => test_overcome(&mut rng, options),
        "overcome2" => test_overcome_2(&mut rng, options),
        _ => unreachable!()
    }
}