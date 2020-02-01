use clap::App as Cli;
use clap::Arg;
use clap::SubCommand;
use log;

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

  let log = SubCommand::with_name("log").about("Show list of changes log");

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
    .subcommand(log)
    .get_matches();

  if let Some(commit_cli) = cli.subcommand_matches("commit") {
    let db_uri = commit_cli
      .value_of("database-uri")
      .expect("db_uri shouldn't be null");

    let dump_output = pg::dump(pg::DumpInput { db_uri })?;

    log::debug!("Creating project...");
    let repo = GitRepo::upsert(config::get_project_dir(), "lol-meh")?;

    log::debug!("Reading db...");
    repo.sync_dump(dump_output.clone())?;

    log::debug!("Writing state changes...");
    repo.commit_dump("Update dump")?;
  } else if let Some(log_cli) = cli.subcommand_matches("log") {
    log::debug!("Running log");
    let repo = GitRepo::upsert(config::get_project_dir(), "lol-meh")?;

    let mut commit_iterator = repo.commit_iterator()?;

    let commit = commit_iterator.nth(0).unwrap()?;
    log::debug!("* {} {}", commit.hash, commit.message);
  }

  Ok(())
}
