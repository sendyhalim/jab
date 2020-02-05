use std::path::PathBuf;

use clap::App as Cli;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use log;

use env_logger;
use lib::config;
use lib::db::postgresql::client as pg;
use lib::git::GitRepo;
use lib::project::Project;
use lib::project::ProjectConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  env_logger::init();

  let create = SubCommand::with_name("create")
    .about("Create a project")
    .arg(Arg::with_name("name").takes_value(true));

  let commit = SubCommand::with_name("commit")
    .about("Commit current db state")
    .arg(
      Arg::with_name("message")
        .long("message")
        .short("m")
        .takes_value(true)
        .required(true)
        .help("Commit message"),
    )
    .arg(
      Arg::with_name("database-uri")
        .long("database-uri")
        .takes_value(true)
        .required(true)
        .help(r#"Database uri, for example: --database-uri="user:secret@localhost/mydb""#),
    );

  let log = SubCommand::with_name("log").about("Show list of changes log");

  let show = SubCommand::with_name("show")
    .about("Show dump for a specific commit")
    .arg(Arg::with_name("commit-hash").takes_value(true));

  let restore = SubCommand::with_name("restore")
    .about("Restore dump for a specific commit")
    .arg(Arg::with_name("commit-hash").takes_value(true))
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
    .subcommand(create)
    .subcommand(commit)
    .subcommand(log)
    .subcommand(show)
    .subcommand(restore)
    .get_matches();

  let project_dir = config::get_project_dir();
  let repo_path = project_dir.join("woi");
  let project = Project {
    config: ProjectConfig {
      repo_path,
      sql_path: PathBuf::from("dump.sql"),
    },
  };

  if let Some(create_cli) = cli.subcommand_matches("create") {
    let project_name = create_cli.value_of("name").unwrap();

    log::debug!("Creating project...");

    let _repo = GitRepo::upsert(project_dir.join(project_name))?;

    println!("Done creating {}", project_name);
  } else if let Some(commit_cli) = cli.subcommand_matches("commit") {
    let db_uri = String::from(get_db_uri(commit_cli));
    let message = String::from(commit_cli.value_of("message").unwrap());
    let dump_output = pg::dump(pg::DumpInput { db_uri })?;

    log::debug!("Creating project...");
    let repo = GitRepo::new(&project.config.repo_path)?;

    log::debug!("Reading db...");
    project.sync_dump(dump_output)?;

    log::debug!("Writing state changes...");
    repo.commit_file(project.config.sql_path, &message)?;
  } else if let Some(_log_cli) = cli.subcommand_matches("log") {
    log::debug!("Running log");

    let repo = GitRepo::new(project.config.repo_path)?;
    let commit_iterator = repo.commit_iterator()?;

    for commit in commit_iterator {
      let commit = commit?;
      println!("* {} {}", commit.hash, commit.message);
    }
  } else if let Some(show_cli) = cli.subcommand_matches("show") {
    let commit_hash = show_cli.value_of("commit-hash").unwrap();
    let _repo = GitRepo::new(project.config.repo_path)?;

    log::debug!("Reading commit {}...", commit_hash);

  // let dump = repo.get_dump_at_commit(String::from(commit_hash))?;
  } else if let Some(restore_cli) = cli.subcommand_matches("restore") {
    let commit_hash = restore_cli.value_of("commit-hash").unwrap();
    let repo = GitRepo::new(project.config.repo_path)?;

    log::debug!("Reading commit...");

    // TODO: This is impractical because it will unnecessarily increase the memory usage.
    // but let's stick with this to target the functional feature first.
    let dump =
      repo.get_file_content_at_commit(project.config.sql_path, String::from(commit_hash))?;
    let db_uri = String::from(get_db_uri(restore_cli));
    let result = pg::restore(pg::RestoreInput { db_uri, sql: dump })?;

    log::debug!("Result {}", result);
    println!("Restore {} done!", commit_hash);
  }

  Ok(())
}

fn get_db_uri<'a>(matches: &'a ArgMatches) -> &'a str {
  return matches.value_of("database-uri").unwrap();
}
