extern crate clipboard;
extern crate getopts;
extern crate keepass;
extern crate rpassword;

use clipboard::{ClipboardContext,ClipboardProvider};
use getopts::{Matches,Options};
use keepass::{Database,Node,OpenDBError};
use rpassword::prompt_password_stdout;
use std::env;
use std::fs::File;
use std::process::exit;

fn main() {
    let matches = get_options();
    let password: String;

    if matches.free.is_empty() {
        print!("Needs a .kbdx file");
        exit(1);
    }

    let db_file = matches.free[0].clone();

    match matches.opt_str("p") {
        Some(p) => { password = p },
        None => { password = prompt_password_stdout("Password: ").unwrap() },
    }

    let db = File::open(std::path::Path::new(&db_file))
        .map_err(|e| OpenDBError::from(e))
        .and_then(|mut db_handle| Database::open(&mut db_handle, &password))
        .unwrap();

    dump_contents(db);
}

fn dump_contents(db: Database) {
    for node in &db.root {
        match node {
            Node::Group(g) => {
                println!("Group '{}'", g.name)
            },
            Node::Entry(e) => {
                let title = e.get_title().unwrap();
                let username = e.get_username().unwrap();
                let password: String = e.get_password().unwrap().chars().map(|_| {
                    '*'
                }).collect();

                println!("Entry \"{}\":", title);
                println!("  username: \"{}\"", username);
                println!("  password: \"{}\"", password);
            },
        }
    }
}

fn get_options() -> Matches {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();

    opts.optflag("h", "help", "print this help menu");
    opts.optopt("p", "password", "taken from STDIN if omitted", "PASSWORD");

    match opts.parse(&args[1..]) {
        Ok(m) => {
            if m.opt_present("h") {
                usage(program, opts);
                exit(0);
            }

            return m;
        },
        Err(e) => {
            print!("{}", e.to_string());
            usage(program, opts);
            exit(1);
        }
    }
}

fn usage(program: String, opts: Options) {
    let firstline = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&firstline));
}
