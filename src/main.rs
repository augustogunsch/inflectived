#![feature(slice_group_by)]

//mod database;
use rocket::routes;
use rocket::fs::FileServer;
use clap::{App, AppSettings, Arg, SubCommand};
//use database::WordDb;
mod database;
mod language;
mod entry;
mod routes;

use database::WordDb;
use language::Language;

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

    let mut db = WordDb::new("test.db");

    let lang = Language::new("polish",
                             vec![String::from("adj"),
                                  String::from("noun"),
                                  String::from("verb"),
                                  String::from("character"),
                                  String::from("suffix"),
                                  String::from("prefix"),
                                  String::from("conj"),
                                  String::from("adv"),
                                  String::from("infix"),
                                  String::from("name"),
                                  String::from("phrase"),
                                  String::from("prep_phrase"),
                                  String::from("intj"),
                                  String::from("det"),
                                  String::from("prep"),
                                  String::from("proverb"),
                                  String::from("abbrev"),
                                  String::from("num"),
                                  String::from("pron"),
                                  String::from("punct"),
                                  String::from("interfix"),
                                  String::from("particle")]);

    match matches.subcommand() {
        ("upgrade", _) => { db.upgrade_lang(&lang).await; },
        ("run", _) => {
            let figment = rocket::Config::figment()
                                         .merge(("address", "0.0.0.0"));

            rocket::custom(figment)
                   .manage(db)
                   .mount("/static", FileServer::from("static/"))
                   .mount("/", routes![routes::get_entries,
                                       routes::get_entries_like,
                                       routes::frontend])
                   .launch()
                   .await.unwrap();
        },
        _ => {}
    }
}
