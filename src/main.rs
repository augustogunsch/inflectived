#![feature(slice_group_by)]

//mod database;
use rocket::routes;
use rocket::fs::FileServer;
use clap::{App, AppSettings, Arg, SubCommand};
//use database::WordDb;
mod database;
mod language;
mod entry;
mod views;

use database::WordDb;
use language::Language;


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

    let lang = Language::new("pl", "Polish");

    match matches.subcommand() {
        ("upgrade", _) => { db.upgrade_lang(&lang).await; },
        ("run", _) => {
            let figment = rocket::Config::figment()
                                         .merge(("address", "0.0.0.0"));

            rocket::custom(figment)
                   .manage(db)
                   .mount("/static", FileServer::from("static/"))
                   .mount("/", routes![views::get_entries,
                                       views::get_entries_like,
                                       views::get_langs,
                                       views::frontend])
                   .launch()
                   .await.unwrap();
        },
        _ => {}
    }
}
