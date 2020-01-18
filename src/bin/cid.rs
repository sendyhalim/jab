use clap::App as Cli;
use clap::Arg;
use clap::SubCommand;

use lib::db::postgresql::client as pg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    println!("HEYY {}", dump_output);
  }

  Ok(())
}
