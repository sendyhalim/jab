use clap::App as Cli;
use clap::Arg;
use clap::SubCommand;

use env_logger;
use lib::config;
use lib::db::postgresql::client as pg;
use lib::git::GitRepo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  env_logger::init();

  let commit = SubCommand::with_name("commit")
    .about("Commit current db state")
    .arg(
      Arg::with_name("database-uri")
        .long("database-uri")
        .takes_value(true)
        .required(true)
        .help(r#"Database uri, for example: --database-uri="user:secret@localhost/mydb""#),
    );

  let cli = Cli::new("CID")
    .version("0.0.1")
    .author("Sendy Halim <sendyhalim93@gmail.com>")
    .about(
      "\
       CID is a database state management tool, think of it as git but for\
       database. You can commit your current db state and checkout to\
       your previous db state.\
       ",
    )
    .subcommand(commit)
    .get_matches();

  if let Some(commit_cli) = cli.subcommand_matches("commit") {
    let db_uri = commit_cli
      .value_of("database-uri")
      .expect("db_uri shouldn't be null");

    let dump_output = pg::dump(pg::DumpInput { db_uri })?;

    println!("Creating project...");
    let repo = GitRepo::upsert(config::get_project_dir(), "lol-meh")?;

    println!("Reading db...");
    repo.sync_dump(dump_output.clone())?;

    println!("Writing state changes...");
    repo.commit_dump("Update dump")?;

    // println!("HEYY {}", dump_output);
  }

  Ok(())
}
