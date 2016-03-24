extern crate csv;
extern crate getopts;
extern crate rustc_serialize;

use getopts::Options;
use std::error::Error;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, RustcDecodable)]
struct Row {
    country: String,
    city: String,
    accent_city: String,
    region: String,
    population: Option<u64>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

struct PopulationCount {
    city: String,
    country: String,
    count: u64,
}

#[derive(Debug)]
enum CliError {
    Io(io::Error),
    Csv(csv::Error),
    NotFound,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Io(ref err) => err.fmt(f),
            CliError::Csv(ref err) => err.fmt(f),
            CliError::NotFound => write!(f, "No matching cities with a \
                                             population were found."),
        }
    }
}

impl Error for CliError {
    fn description(&self) -> &str {
        match *self {
            CliError::Io(ref err) => err.description(),
            CliError::Csv(ref err) => err.description(),
            CliError::NotFound => "not found",
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}

impl From<csv::Error> for CliError {
    fn from(err: csv::Error) -> CliError {
        CliError::Csv(err)
    }
}

fn print_usage(program: &str, opts: Options) {
    println!("{}", opts.usage(&format!("Usage: {} [options] <city>", program)));
}

fn search<P: AsRef<Path>>(file_path: &Option<P>, city: &str)
                          -> Result<Vec<PopulationCount>, CliError> {
    let mut found = vec![];
    let input: Box<io::Read> = match *file_path {
        None => Box::new(io::stdin()),
        Some(ref file_path) => Box::new(try!(fs::File::open(file_path))),
    };
    let mut rdr = csv::Reader::from_reader(input);

    for row in rdr.decode::<Row>() {
        let row = try!(row);
        match row.population {
            None => {},
            Some(count) => if row.city == city {
                found.push(PopulationCount {
                    city: row.city,
                    country: row.country,
                    count: count,
                });
            },
        }
    }

    if found.is_empty() {
        Err(CliError::NotFound)
    } else {
        Ok(found)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("f", "file", "Choose an input file, instead of STDIN.", "NAME");
    opts.optflag("h", "help", "Show this usage message.");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => { panic!(e.to_string()) },
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let file = matches.opt_str("f");
    let data_file = file.as_ref().map(Path::new);
    let city = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    match search(&data_file, &city) {
        Ok(pops) => {
            for pop in pops {
                println!("{}, {}: {:?}", pop.city, pop.country, pop.count);
            }
        },
        Err(err) => println!("{}", err),
    }
}
