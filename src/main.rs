extern crate csv;
extern crate docopt;
extern crate lettre;
extern crate handlebars;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod derangement;

use derangement::random_derangement;
use docopt::Docopt;

use lettre::file::FileEmailTransport;
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::SUBMISSION_PORT;
use lettre::smtp::extension::ClientId;
use lettre::smtp::ConnectionReuseParameters;
use lettre::{SimpleSendableEmail, EmailTransport, EmailAddress, SendableEmail, SmtpTransport};

use handlebars::Handlebars;

use rand::{thread_rng, Rng};
use std::env::temp_dir;
use std::fs::File;
use std::io::{BufReader};
use std::io::prelude::*;
use std::path::Path;
use std::clone::Clone;

const USAGE: &'static str = "
A secret santa program that assign unique themes to each participant.

In the spirit of Krampus, participants are encouraged to follows those simple rules:

  1) Get drunk and generate the yearly list of surreal and absurd themes
  2) Make a gift fitting the themes they received



Usage:
  krampus [--test] [--template=<t>] [--ntheme=<n>] <participants-csv> <theme-file>
  krampus (-h | --help)
  krampus --version

Options:
  --ntheme=<n>      Set number of theme per participant to `<n>`.
  --template=<t>    Use email template <t> [default: templates/en.hbs]
  --test            Does not send anything. Displays on console instead
  -h --help         Show this screen.
  --version         Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_participants_csv: String,
    arg_theme_file: String,
    flag_template: String,
    flag_test: bool,
    flag_ntheme: Option<usize>
}

#[derive(Debug, Deserialize, Serialize)]
struct Participant {
    name: String,
    email: String,
    except: String
}

#[derive(Debug, Serialize)]
struct Match<'a> {
    from: &'a Participant,
    to: &'a Participant,
    themes: &'a [String]
}

fn print_email(email: &SimpleSendableEmail) {
    println!(
        "from: {:?}\nto: {:?}\nsubject: {:?}\nmessage:\n{}",
        (&email).from(),
        (&email).to(),
        (&email).message_id(),
        String::from_utf8((*(&email).message()).to_vec()).unwrap()
    );
}

fn main() {

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let mut rng = thread_rng();

    let participants_file = File::open(args.arg_participants_csv).unwrap();
    let participants: Vec<Participant> =
        csv::ReaderBuilder::new()
            .delimiter(b',')
            .double_quote(false)
            .escape(Some(b'\\'))
            .flexible(true)
            .comment(Some(b'#'))
            .from_reader(participants_file)
            .deserialize()
            .map(|result| result.unwrap())
            .collect()
            ;

    let themes_file = File::open(args.arg_theme_file).unwrap();
    let mut themes: Vec<String> = BufReader::new(themes_file).lines().map(|l| l.unwrap()).collect();
    let themes_per_participants = themes.len() / participants.len();
    let themes_per_participants = match args.flag_ntheme {
        Some(n) if n <= themes_per_participants => n,
        Some(n) => panic!("Not enough themes"),
        None => themes_per_participants
    };

    rng.shuffle(&mut themes);
    let themes = themes.chunks(themes_per_participants);

    let mut handlebars = Handlebars::new();
    let canonical_path = Path::new(&args.flag_template).canonicalize().unwrap();
    let template_file = canonical_path.as_path();
    let template_name = template_file.file_name().unwrap()
                                     .to_str().unwrap();
    handlebars.register_template_file(template_name, template_file);

    let mut matches : Vec<(&Participant, &Participant)>; 
    
    loop {
        matches = random_derangement(participants.len())
            .into_iter()
            .enumerate()
            .map(|(from, to)| (&participants[from], &participants[to]))
            .collect()
        ;

        let mut matched_except = false;
        for &(ref from, ref to) in matches.iter() {
            if from.name == to.except || to.name == from.except {
                matched_except = true;
            }
        }
        if !matched_except { break; }
    }
    let mut emails : Vec<SimpleSendableEmail> = matches
        .into_iter()
        .zip(themes)
        .map(|((from, to), themes)| Match { from: from, to: to, themes: themes })
        .map(|m: Match| SimpleSendableEmail::new(
            // TODO : Allow Sender configuration
            EmailAddress::new("".to_string()),
            vec![ EmailAddress::new(m.from.email.clone()) ],
            "Pige de noel".to_string(),
            handlebars.render(template_name, &m).unwrap()
        ))
        .collect();

    if args.flag_test {
        let dir = temp_dir();
        let mut sender = FileEmailTransport::new(dir);
        for email in emails {
            print_email(&email);
        };
    } else {
        // TODO : Allow SMTP configuration
        let username : String = "".to_string();
        let password : String = "".to_string();
        let mut mailer = SmtpTransport::simple_builder("smtp.gmail.com".to_string()).unwrap()
            .hello_name(ClientId::Domain("cobalt".to_string()))
            .credentials(Credentials::new(username, password))
            .smtp_utf8(true)
            .authentication_mechanism(Mechanism::Plain)
            .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
            .build();
        for email in emails {
            mailer.send(&email);
        };
    };
}
