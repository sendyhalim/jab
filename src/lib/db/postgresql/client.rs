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
  port: Option<String>,
  username: String,
  password: Option<String>,
}

#[derive(Debug, Clone, Fail)]
pub enum DbConnectionError {
  #[fail(display = "Error when parsing db uri {}, parsing step: ", db_uri)]
  DbUriParseError { parsing_step: String, db_uri: String },
}

impl DbConnectionError {
  fn parse_error(db_uri: &str, step: DbUriParsingStep) -> DbConnectionError {
    return DbConnectionError::DbUriParseError {
      db_uri: String::from(db_uri),
      parsing_step: step.to_string(),
    };
  }
}

#[derive(Debug)]
enum DbUriParsingStep {
  ParseDbName,
  ParseCredentialAndHostCandidate,
  ParseHostAndPort,
  ParseHost,
  ParseCredential,
  ParseCredentialUsername,
}

impl ToString for DbUriParsingStep {
  fn to_string(&self) -> String {
    return format!("{:?}", self);
  }
}

impl DbConnectionConfig {
  fn from(db_uri: &str) -> ResultDynError<DbConnectionConfig> {
    let parts: Vec<&str> = db_uri.split("/").collect();
    let db_name = parts.get(1)
      .ok_or(DbConnectionError::parse_error(db_uri, DbUriParsingStep::ParseDbName))?;

    let target_str = String::from(
      *parts.get(0).ok_or(DbConnectionError::parse_error(db_uri, DbUriParsingStep::ParseCredentialAndHostCandidate))?
    );
    let parts: Vec<&str> = target_str.split("@").collect();
    let credential_str = String::from(
      *parts.get(0).ok_or(DbConnectionError::parse_error(db_uri, DbUriParsingStep::ParseCredential))?
    );
    let host_and_port = parts.get(1)
      .ok_or(DbConnectionError::parse_error(db_uri, DbUriParsingStep::ParseHostAndPort))?;

    let host_and_port_parts: Vec<&str> = host_and_port.split(":").collect();
    let host = host_and_port_parts.get(0)
      .ok_or(DbConnectionError::parse_error(db_uri, DbUriParsingStep::ParseHost))?;
    let port = host_and_port_parts.get(1).map(|port|(*port).into());

    let parts: Vec<&str> = credential_str.split(":").collect();
    let username = parts.get(0).ok_or(DbConnectionError::parse_error(db_uri, DbUriParsingStep::ParseCredentialUsername))?;
    let password = parts.get(1).map(|pass| (*pass).into());

    return Ok(DbConnectionConfig {
      db_name: (*db_name).into(),
      username: (*username).into(),
      password,
      host: (*host).into(),
      port,
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
  let port = db_connection_config.port.or(Some("".into())).unwrap();

  log::debug!("Running pg_restore");
  let mut command = Command::new("pg_restore");

  command
    .env("PGPASSWORD", password)
    .arg("--clean")
    .arg(format!("--username={}", db_connection_config.username))
    .arg(format!("--dbname={}", db_connection_config.db_name))
    .arg(format!("--host={}", db_connection_config.host))
    .arg(format!("--port={}", port))
    .arg("--single-transaction")
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
