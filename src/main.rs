#![feature(slice_group_by, path_try_exists)]

use std::process::exit;
use std::fs;

//mod database;
use rocket::routes;
use rocket::fs::FileServer;
use clap::{App, AppSettings, Arg, SubCommand};

//use database::WordDb;
mod database;
mod language;
mod entry;
mod views;
mod version;
mod util;

use database::{WordDb, DbError};

const DB_DIR: &str = "/usr/share/inflectived";
const CACHE_DIR: &str = "/var/cache/inflectived";
//const FRONTEND_DIR: &str = "/opt/inflectived";
const FRONTEND_DIR: &str = "static";

const MAJOR: i32 = 0;
const MINOR: i32 = 1;
const PATCH: i32 = 0;

#[rocket::main]
async fn main() {
    let matches = App::new("inflectived")
        .version("0.1")
        .author("Augusto Gunsch <augustogunsch@tutanota.com>")
        .about("inflective daemon")
        .subcommands(vec![
            SubCommand::with_name("upgrade")
                .about("Upgrade or install a language database")
                .arg(
                    Arg::with_name("LANG")
                        .required(true)
                        .index(1)
                        .help("Language database to upgrade"),
                ),
            SubCommand::with_name("run").about("Run the daemon").arg(
                Arg::with_name("port")
                    .short("p")
                    .long("port")
                    .value_name("PORT")
                    .help("Port to run the server on")
                    .takes_value(true),
            ),
            SubCommand::with_name("list")
                .about("List language databases")
                .arg(
                    Arg::with_name("installed")
                        .short("i")
                        .long("installed")
                        .help("List only installed databases"),
                ),
            SubCommand::with_name("passwd").about("Set admin password for remote management"),
        ])
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let mut db = WordDb::new("inflectived.db");

    match matches.subcommand() {
        ("upgrade", matches) => {
            let lang = db.get_lang(matches.unwrap().value_of("LANG").unwrap());

            if let None = lang {
                eprintln!("The requested language is not available.");
                eprintln!("Available languages:");
                eprint!("{}", db.list_available());
                exit(1);
            }

            if let Err(e) = db.upgrade_lang(&lang.unwrap()).await {
                match e {
                    DbError::AccessDenied => {
                        eprintln!("Permission denied. Please run as root.");
                        exit(1);
                    }
                }
            }
        },
        ("run", matches) => {
            let figment = rocket::Config::figment()
                                         .merge(("address", "0.0.0.0"));

            let mut app = rocket::custom(figment)
                                 .manage(db)
                                 .mount("/", routes![views::get_entries,
                                                     views::get_entries_like,
                                                     views::get_langs,
                                                     views::frontend]);

            if let Ok(_) = fs::try_exists(FRONTEND_DIR) {
                app = app.mount("/static", FileServer::from(FRONTEND_DIR));
            }

            app.launch().await.unwrap();
        },
        _ => {}
    }
}
