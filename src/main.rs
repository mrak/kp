extern crate clipboard;
extern crate getopts;
extern crate keepass;
extern crate rpassword;
extern crate cursive;

use clipboard::{ClipboardContext,ClipboardProvider};
use cursive::Cursive;
use cursive::align::Align;
use cursive::direction::Orientation;
use cursive::views::{Dialog,BoxView,TextView,ListView,LinearLayout,Panel};
use getopts::{Matches,Options};
use keepass::{Database,Node,OpenDBError};
use rpassword::prompt_password_stderr;
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
        None => { password = prompt_password_stderr("Password: ").unwrap() },
    }

    let db = File::open(std::path::Path::new(&db_file))
        .map_err(|e| OpenDBError::from(e))
        .and_then(|mut db_handle| Database::open(&mut db_handle, &password))
        .unwrap();

    run(db_file, db);
    //dump_contents(db);
}

fn run(file: String, db: Database) {
    let mut ui = Cursive::new();
    let entries_view = BoxView::with_full_screen(
        Dialog::new()
        .title("Entries")
        .content(TextView::new("entries view").align(Align::center()))
    );
    let sidebar_view = BoxView::with_full_height(
        Dialog::new()
        .title("Groups")
        .content(ListView::new())
    );
    let info_view = BoxView::with_full_width(
        Dialog::new()
        .title("Details")
        .content(TextView::new("detail view").align(Align::center()))
    );

    ui.add_global_callback('q', |ui| ui.quit());
    ui.add_fullscreen_layer(Dialog::new()
        .title(format!("Database: {}", file))
        .content(LinearLayout::new(Orientation::Horizontal)
            .child(sidebar_view)
            .child(LinearLayout::new(Orientation::Vertical)
                .child(entries_view)
                .child(info_view)
            )
        )
    );

    ui.run();
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
