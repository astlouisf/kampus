extern crate csv;
extern crate docopt;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod derangement;

use derangement::random_derangement;
use docopt::Docopt;
use rand::{thread_rng, Rng};
use std::fs::File;
use std::io::{BufReader};
use std::io::prelude::*;


const USAGE: &'static str = "
A secret santa program that assign unique themes to each participant.

In the spirit of Krampus, participants are encouraged to follows those simple rules:

  1) Get drunk and generate the yearly list of surreal and absurd themes
  2) Make a gift fitting the themes they received

Usage:
  krampus [--test] <participants-csv> <theme-file>
  krampus (-h | --help)
  krampus --version

Options:
  --test        Does not send anything. Displays on console instead
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_participants_csv: String,
    arg_theme_file: String,
    flag_test: bool
}

#[derive(Debug, Deserialize)]
struct Participant {
    name: String,
    email: String
}


fn main() {

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let mut rng = thread_rng();

    let themes_file = File::open(args.arg_theme_file).unwrap();
    let mut themes: Vec<String> = BufReader::new(themes_file).lines().map(|l| l.unwrap()).collect();

    rng.shuffle(&mut themes);

    let participants_file = File::open(args.arg_participants_csv).unwrap();
    let participants: Vec<Participant> =
        csv::Reader::from_reader(participants_file)
            .deserialize()
            .map(|result| result.unwrap())
            .collect()
            ;

    let derangement = random_derangement(participants.len());

    let matches : Vec<((&Participant, &Participant), &[String])> = derangement
        .into_iter()
        .enumerate()
        .map(|(from, to)| (&participants[from], &participants[to]))
        .zip(themes.chunks(themes.len() / participants.len()))
        .collect()
        ;

    if args.flag_test {
        for m in matches {
            println!("{} -> {}", (m.0).0.name, (m.0).1.name);
            for t in m.1 {
                println!("  {}",t);
            }
        }
    } else {
        println!("To do");
    }

}
