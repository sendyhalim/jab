use std::fs;
use std::io;
use std::process::Command;

use crate::types::ResultDynError;

pub struct DumpInput {
  pub db_uri: String,
}

pub fn dump(input: DumpInput) -> ResultDynError<Vec<u8>> {
  let output = Command::new("pg_dump")
    .arg(format!("postgres://{}", input.db_uri))
    .arg("-Fc")
    .output()?;

  let err = String::from_utf8(output.stderr)?;

  if err.len() > 0 {
    let err = io::Error::new(io::ErrorKind::Other, err);

    return Err(Box::new(err));
  }

  return Ok(output.stdout);
}

pub struct RestoreInput {
  pub db_uri: String,
  pub sql: Vec<u8>,
}

#[derive(Debug)]
struct DbConnectionConfig {
  db_name: String,
  host: String,
  username: String,
  password: Option<String>,
}

impl DbConnectionConfig {
  fn from(db_uri: String) -> ResultDynError<DbConnectionConfig> {
    let parts: Vec<&str> = db_uri.split("/").collect();
    let db_name = String::from(*parts.get(1).unwrap());

    let target_str = String::from(*parts.get(0).unwrap());
    let parts: Vec<&str> = target_str.split("@").collect();
    let credential_str = String::from(*parts.get(0).unwrap());
    let host = String::from(*parts.get(1).unwrap());

    let parts: Vec<&str> = credential_str.split("@").collect();
    let username = String::from(*parts.get(0).unwrap());
    let password = parts.get(1).map(|pass| String::from(*pass));

    return Ok(DbConnectionConfig {
      db_name,
      username,
      password,
      host,
    });
  }
}

pub fn restore(input: RestoreInput) -> ResultDynError<String> {
  let db_connection_config = DbConnectionConfig::from(input.db_uri)?;
  let temp_file_path = "/tmp/cid.sql";

  log::debug!("Parsed config {:?}", db_connection_config);
  log::debug!("Writing dump to a temp file");
  fs::write(temp_file_path, input.sql)?;

  let password = db_connection_config
    .password
    .or(Some(String::from("")))
    .unwrap();

  log::debug!("Running pg_restore");
  let mut command = Command::new("pg_restore");

  command
    .env("PG_PASSWORD", password)
    .arg("--clean")
    .arg(format!("--username={}", db_connection_config.username))
    .arg(format!("--dbname={}", db_connection_config.db_name))
    .arg(format!("--host={}", db_connection_config.host))
    .arg(temp_file_path);

  log::debug!("Created command {:?}", command);

  let output = command.output()?;

  log::debug!("Cleaning out temp file");
  fs::remove_file(temp_file_path)?;

  let err = String::from_utf8(output.stderr)?;

  if err.len() > 0 {
    let err = io::Error::new(io::ErrorKind::Other, err);

    return Err(Box::new(err));
  }

  let output = String::from_utf8(output.stdout)?;

  return Ok(output);
}
