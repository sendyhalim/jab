use std::fs;
use std::io;
use std::process::Command;

use failure::Fail;

use crate::types::ResultDynError;

pub struct DumpInput<'a> {
  pub db_uri: &'a str,
}

pub fn dump(input: DumpInput) -> ResultDynError<Vec<u8>> {
  let output = Command::new("pg_dump")
    .arg(format!("postgres://{}", input.db_uri))
    .arg("-Fc")
    .output()?;

  let err = String::from_utf8(output.stderr)?;

  if err.len() > 0 {
    let err = io::Error::new(io::ErrorKind::Other, err);

    return Err(err.into());
  }

  return Ok(output.stdout);
}

pub struct RestoreInput<'a> {
  pub db_uri: &'a str,
  pub sql: Vec<u8>,
}

#[derive(Debug)]
struct DbConnectionConfig {
  db_name: String,
  host: String,
  username: String,
  password: Option<String>,
}

#[derive(Debug, Clone, Fail)]
pub enum DbConnectionError {
  #[fail(display = "Invalid DB URI format: {}", db_uri)]
  InvalidDbUriError { db_uri: String },
}

impl DbConnectionConfig {
  fn from(db_uri: &str) -> ResultDynError<DbConnectionConfig> {
    let invalid_db_uri_error = DbConnectionError::InvalidDbUriError {
      db_uri: String::from(db_uri),
    };

    let parts: Vec<&str> = db_uri.split("/").collect();
    let db_name = parts.get(1).ok_or(invalid_db_uri_error.clone())?;

    let target_str = String::from(*parts.get(0).ok_or(invalid_db_uri_error.clone())?);
    let parts: Vec<&str> = target_str.split("@").collect();
    let credential_str = String::from(*parts.get(0).ok_or(invalid_db_uri_error.clone())?);
    let host = parts.get(1).ok_or(invalid_db_uri_error.clone())?;

    let parts: Vec<&str> = credential_str.split("@").collect();
    let username = parts.get(0).ok_or(invalid_db_uri_error.clone())?;
    let password = parts.get(1).map(|pass| (*pass).into());

    return Ok(DbConnectionConfig {
      db_name: (*db_name).into(),
      username: (*username).into(),
      password,
      host: (*host).into(),
    });
  }
}

pub fn restore(input: RestoreInput) -> ResultDynError<String> {
  let db_connection_config = DbConnectionConfig::from(input.db_uri)?;
  let temp_file_path = "/tmp/cid.sql";

  log::debug!("Parsed config {:?}", db_connection_config);
  log::debug!("Writing dump to a temp file");
  fs::write(temp_file_path, input.sql)?;

  let password = db_connection_config.password.or(Some("".into())).unwrap();

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

    return Err(err.into());
  }

  let output = String::from_utf8(output.stdout)?;

  return Ok(output);
}
