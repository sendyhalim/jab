use std::fs;

use clap::App as Cli;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use log;

use env_logger;
use lib::config;
use lib::config::CidConfig;
use lib::config::ProjectConfig;
use lib::db::postgresql::client as pg;
use lib::project;
use lib::project::Project;
use lib::project_manager::CreateProjectInput;
use lib::project_manager::OpenProjectInput;
use lib::project_manager::ProjectManager;
use lib::types::ResultDynError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  env_logger::init();

  log::debug!("Preparing cid..");
  MainProjectManager::bootstrap()?;

  let cli = Cli::new("CID")
    .version("0.0.1")
    .author("Sendy Halim <sendyhalim93@gmail.com>")
    .setting(clap::AppSettings::ArgRequiredElseHelp)
    .about(
      "\
       CID is a database state management tool, think of it as git but for\
       database. You can commit your current db state and checkout to\
       your previous db state.\
       ",
    )
    .subcommand(project_cmd())
    .get_matches();

  if let Some(project_cli) = cli.subcommand_matches("project") {
    handle_project_cli(project_cli)?;
  }

  return Ok(());
}

fn project_cmd<'a, 'b>() -> Cli<'a, 'b> {
  let project_name_arg = Arg::with_name("project")
    .takes_value(true)
    .required(true)
    .help("Project name");

  return SubCommand::with_name("project")
    .setting(clap::AppSettings::ArgRequiredElseHelp)
    .about("Project cli")
    .subcommand(
      SubCommand::with_name("create")
        .about("Create a project")
        .arg(Arg::with_name("name").takes_value(true))
        .arg(
          Arg::with_name("database-uri")
            .long("database-uri")
            .takes_value(true)
            .required(true)
            .help(r#"Database uri, for example: --database-uri="user:secret@localhost/mydb""#),
        ),
    )
    .subcommand(SubCommand::with_name("list").about("List projects"))
    .subcommand(
      SubCommand::with_name("commit")
        .about("Commit current db state")
        .arg(project_name_arg.clone())
        .arg(
          Arg::with_name("message")
            .long("message")
            .short("m")
            .takes_value(true)
            .required(true)
            .help("Commit message"),
        ),
    )
    .subcommand(
      SubCommand::with_name("log")
        .arg(project_name_arg.clone())
        .about("Show list of changes log"),
    )
    .subcommand(
      SubCommand::with_name("show")
        .about("Show dump for a specific commit")
        .arg(project_name_arg.clone())
        .arg(Arg::with_name("commit-hash").takes_value(true)),
    )
    .subcommand(
      SubCommand::with_name("restore")
        .about("Restore dump for a specific commit")
        .arg(project_name_arg.clone())
        .arg(
          Arg::with_name("commit-hash")
            .required(true)
            .takes_value(true),
        ),
    );
}

fn handle_project_cli(cli: &ArgMatches) -> ResultDynError<()> {
  log::debug!("Reading cid config");

  let cid_config = CidConfig::read()?;
  let mut project_manager: MainProjectManager = MainProjectManager::new(cid_config);

  if let Some(create_cli) = cli.subcommand_matches("create") {
    log::debug!("Creating project...");

    let project_name = create_cli.value_of("name").unwrap();
    let db_uri = create_cli.value_of("database-uri").unwrap();

    let project = project_manager.create_project(&CreateProjectInput {
      project_dir: config::get_cid_dir().as_ref(),
      project_name,
      db_uri,
    })?;

    println!("Done creating {}", project.name());
  } else if let Some(_) = cli.subcommand_matches("list") {
    println!("Available projects:");

    project_manager
      .get_project_names()?
      .iter()
      .for_each(|name| {
        println!("* {}", name);
      });
  } else if let Some(commit_cli) = cli.subcommand_matches("commit") {
    let project = project_manager.open_project_from_args(commit_cli)?;

    let message = commit_cli.value_of("message").unwrap();
    let dump_output = pg::dump(pg::DumpInput {
      db_uri: project.db_uri(),
    })?;

    project.commit_dump(message, dump_output)?;
  } else if let Some(log_cli) = cli.subcommand_matches("log") {
    let project = project_manager.open_project_from_args(log_cli)?;

    log::debug!("Running log");

    let commit_iterator = project.commit_iterator()?;

    for commit in commit_iterator {
      let commit = commit?;
      println!("* {} {}", commit.hash, commit.message);
    }
  } else if let Some(show_cli) = cli.subcommand_matches("show") {
    let project = project_manager.open_project_from_args(show_cli)?;

    let commit_hash = show_cli.value_of("commit-hash").unwrap();

    log::debug!("Reading commit {}...", commit_hash);

  // TODO: Show commit details!
  // let dump = repo.get_dump_at_commit(String::from(commit_hash))?;
  } else if let Some(restore_cli) = cli.subcommand_matches("restore") {
    let project = project_manager.open_project_from_args(restore_cli)?;

    let commit_hash = restore_cli.value_of("commit-hash").unwrap();

    log::debug!("Reading commit...");

    // TODO: This is impractical because it will unnecessarily increase the memory usage.
    // but let's stick with this to target the functional feature first.
    let dump = project.get_dump_at_commit(commit_hash)?;
    let result = pg::restore(pg::RestoreInput {
      db_uri: project.db_uri(),
      sql: dump,
    })?;

    log::debug!("Result {}", result);
    println!("Restore {} done!", commit_hash);
  }

  return Ok(());
}

struct MainProjectManager {
  cid_config: CidConfig,
}

impl MainProjectManager {
  fn open_project_from_args(&self, matches: &ArgMatches) -> ResultDynError<Project> {
    let project_name = matches.value_of("project").unwrap();
    let project_config = self.cid_config.project_config(project_name)?;

    return self.open_project(&OpenProjectInput {
      project_dir: config::get_cid_dir().as_ref(),
      project_name,
      db_uri: &project_config.db_uri,
    });
  }
}

impl ProjectManager for MainProjectManager {
  fn bootstrap() -> ResultDynError<()> {
    let cid_dir = config::get_cid_dir();

    if !cid_dir.exists() {
      fs::create_dir(cid_dir)?;
    }

    let config_path = CidConfig::get_path();

    if !config_path.exists() {
      fs::write(config_path, CidConfig::empty_config_str())?;
    }

    return Ok(());
  }

  fn new(cid_config: CidConfig) -> MainProjectManager {
    return MainProjectManager { cid_config };
  }

  fn create_project(&mut self, input: &CreateProjectInput) -> ResultDynError<Project> {
    let project = Project::create(&project::CreateInput {
      project_dir: input.project_dir,
      project_name: input.project_name,
      db_uri: input.db_uri,
    })?;

    self.cid_config.register_project_config(ProjectConfig {
      name: String::from(project.name()),
      db_uri: String::from(project.db_uri()),
    });

    CidConfig::persist(&self.cid_config)?;

    return Ok(project);
  }

  fn open_project(&self, input: &OpenProjectInput) -> ResultDynError<Project> {
    return Project::open(&project::OpenInput {
      project_dir: input.project_dir,
      project_name: input.project_name,
      db_uri: input.db_uri,
    });
  }

  fn get_project_names(&self) -> ResultDynError<Vec<&str>> {
    let project_names: Vec<&str> = self
      .cid_config
      .projects
      .values()
      .map(|config| return config.name.as_ref())
      .collect();

    return Ok(project_names);
  }
}
