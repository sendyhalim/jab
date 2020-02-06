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
    .arg(Arg::with_name("name").takes_value(true))
    .arg(
      Arg::with_name("database-uri")
        .long("database-uri")
        .takes_value(true)
        .required(true)
        .help(r#"Database uri, for example: --database-uri="user:secret@localhost/mydb""#),
    );

  let project_name_arg = Arg::with_name("project")
    .long("project")
    .short("p")
    .takes_value(true)
    .required(true)
    .help("Project name");

  let commit = SubCommand::with_name("commit")
    .about("Commit current db state")
    .arg(project_name_arg.clone())
    .arg(
      Arg::with_name("message")
        .long("message")
        .short("m")
        .takes_value(true)
        .required(true)
        .help("Commit message"),
    );

  let log = SubCommand::with_name("log")
    .arg(project_name_arg.clone())
    .about("Show list of changes log");

  let show = SubCommand::with_name("show")
    .about("Show dump for a specific commit")
    .arg(project_name_arg.clone())
    .arg(Arg::with_name("commit-hash").takes_value(true));

  let restore = SubCommand::with_name("restore")
    .about("Restore dump for a specific commit")
    .arg(project_name_arg.clone())
    .arg(Arg::with_name("commit-hash").takes_value(true));

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

  if let Some(create_cli) = cli.subcommand_matches("create") {
    let project_name = create_cli.value_of("name").unwrap();
    let project = open_project(project_name);

    log::debug!("Creating project...");

    let _repo = GitRepo::upsert(project.project_dir())?;

    println!("Done creating {}", project_name);
  } else if let Some(commit_cli) = cli.subcommand_matches("commit") {
    let project = open_project_from_args(commit_cli);
    let db_uri = String::from(get_db_uri(commit_cli));
    let message = String::from(commit_cli.value_of("message").unwrap());
    let dump_output = pg::dump(pg::DumpInput { db_uri })?;

    log::debug!("Creating project...");
    let repo = GitRepo::new(project.repo_path())?;

    log::debug!("Reading db...");
    project.sync_dump(dump_output)?;

    log::debug!("Writing state changes...");
    repo.commit_file(project.sql_path(), &message)?;
  } else if let Some(log_cli) = cli.subcommand_matches("log") {
    let project = open_project_from_args(log_cli);

    log::debug!("Running log");

    let repo = GitRepo::new(project.repo_path())?;
    let commit_iterator = repo.commit_iterator()?;

    for commit in commit_iterator {
      let commit = commit?;
      println!("* {} {}", commit.hash, commit.message);
    }
  } else if let Some(show_cli) = cli.subcommand_matches("show") {
    let project = open_project_from_args(show_cli);
    let commit_hash = show_cli.value_of("commit-hash").unwrap();
    let _repo = GitRepo::new(project.repo_path())?;

    log::debug!("Reading commit {}...", commit_hash);

  // let dump = repo.get_dump_at_commit(String::from(commit_hash))?;
  } else if let Some(restore_cli) = cli.subcommand_matches("restore") {
    let project = open_project_from_args(restore_cli);
    let commit_hash = restore_cli.value_of("commit-hash").unwrap();
    let repo = GitRepo::new(project.repo_path())?;

    log::debug!("Reading commit...");

    // TODO: This is impractical because it will unnecessarily increase the memory usage.
    // but let's stick with this to target the functional feature first.
    let dump = repo.get_file_content_at_commit(project.sql_path(), String::from(commit_hash))?;
    let db_uri = String::from(get_db_uri(restore_cli));
    let result = pg::restore(pg::RestoreInput { db_uri, sql: dump })?;

    log::debug!("Result {}", result);
    println!("Restore {} done!", commit_hash);
  }

  Ok(())
}

fn open_project_from_args(matches: &ArgMatches) -> Project {
  let project_name = matches.value_of("project").unwrap();

  return open_project(project_name);
}

fn open_project(name: &str) -> Project {
  let project_dir = config::get_project_dir();

  return Project::open(ProjectConfig {
    project_dir,
    name: String::from(name),
  });
}

fn get_db_uri<'a>(matches: &'a ArgMatches) -> &'a str {
  return matches.value_of("database-uri").unwrap();
}
