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
  DbUriParseError {
    parsing_step: String,
    db_uri: String,
  },
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
  DbName,
  CredentialAndHostCandidate,
  HostAndPort,
  Host,
  Credential,
  CredentialUsername,
}

impl ToString for DbUriParsingStep {
  fn to_string(&self) -> String {
    return format!("{:?}", self);
  }
}

impl DbConnectionConfig {
  fn from(db_uri: &str) -> ResultDynError<DbConnectionConfig> {
    let parts: Vec<&str> = db_uri.split('/').collect();
    let db_name = parts
      .get(1)
      .ok_or({ DbConnectionError::parse_error(db_uri, DbUriParsingStep::DbName) })?;

    let target_str = String::from(*parts.get(0).ok_or({
      DbConnectionError::parse_error(db_uri, DbUriParsingStep::CredentialAndHostCandidate)
    })?);

    let parts: Vec<&str> = target_str.split('@').collect();
    let credential_str = String::from(
      *parts
        .get(0)
        .ok_or({ DbConnectionError::parse_error(db_uri, DbUriParsingStep::Credential) })?,
    );
    let host_and_port = parts
      .get(1)
      .ok_or({ DbConnectionError::parse_error(db_uri, DbUriParsingStep::HostAndPort) })?;

    let host_and_port_parts: Vec<&str> = host_and_port.split(':').collect();
    let host = host_and_port_parts
      .get(0)
      .ok_or({ DbConnectionError::parse_error(db_uri, DbUriParsingStep::Host) })?;
    let port = host_and_port_parts.get(1).map(|port| (*port).into());

    let parts: Vec<&str> = credential_str.split(':').collect();
    let username = parts
      .get(0)
      .ok_or({ DbConnectionError::parse_error(db_uri, DbUriParsingStep::CredentialUsername) })?;
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

/// The cleaniest way to do clean restore is by doing below steps:
/// * Drop DB.
/// * Create DB.
/// * Run pg_restore.
///
/// Note that maybe this will change in the future, we would
/// probably need to write our own version of pg_restore and pg_dump.
pub fn restore(input: RestoreInput) -> ResultDynError<String> {
  let db_connection_config = DbConnectionConfig::from(input.db_uri)?;
  let temp_file_path = "/tmp/cid.sql";

  log::debug!("Parsed config {:?}", db_connection_config);
  log::debug!("Writing dump to a temp file");
  fs::write(temp_file_path, input.sql)?;

  let password = db_connection_config
    .password
    .or({ Some("".into()) })
    .unwrap();
  let port = db_connection_config.port.or({ Some("".into()) }).unwrap();
  let username = format!("--username={}", db_connection_config.username);
  let dbname = format!("--dbname={}", db_connection_config.db_name);
  let host = format!("--host={}", db_connection_config.host);
  let port = format!("--port={}", port);

  // Drop DB
  log::debug!("Dropping DB");

  let output = Command::new("dropdb")
    .env("PGPASSWORD", &password)
    .arg(&username)
    .arg(&host)
    .arg(&port)
    .arg(&db_connection_config.db_name)
    .output()?;

  log::debug!("Drop db result {:?}", output);

  // Create DB
  log::debug!("Recreating DB");

  let output = Command::new("createdb")
    .env("PGPASSWORD", &password)
    .arg(&username)
    .arg(&host)
    .arg(&port)
    .arg(&db_connection_config.db_name)
    .output()?;

  log::debug!("Create db result {:?}", output);

  // Run pg_restore
  log::debug!("Running pg_restore");
  let mut command = Command::new("pg_restore");

  command
    .env("PGPASSWORD", &password)
    .arg(&username)
    .arg(&dbname)
    .arg(&host)
    .arg(&port)
    .arg("--single-transaction")
    .arg(temp_file_path);

  log::debug!("Created command {:?}", command);

  let output = command.output()?;

  log::debug!("Cleaning out temp file");
  fs::remove_file(temp_file_path)?;

  let err = String::from_utf8(output.stderr)?;

  if !err.is_empty() {
    let err = io::Error::new(io::ErrorKind::Other, err);

    return Err(err.into());
  }

  let output = String::from_utf8(output.stdout)?;

  return Ok(output);
}

#[cfg(test)]
mod test {
  use super::*;

  mod DbConnectionConfigTest {
    use super::*;

    mod from {
      use super::*;

      #[test]
      fn it_should_return_valid_db_config_without_password() -> ResultDynError<()> {
        let config = DbConnectionConfig::from("yay@localhost/testdb")?;

        assert_eq!(config.username, "yay");
        assert!(config.password.is_none());
        assert_eq!(config.db_name, "testdb");
        assert_eq!(config.host, "localhost");
        assert!(config.port.is_none());

        return Ok(());
      }

      #[test]
      fn it_should_return_valid_db_config_with_password() -> ResultDynError<()> {
        let config = DbConnectionConfig::from("yay:hidden@localhost/testdb")?;

        assert_eq!(config.username, "yay");
        assert_eq!(config.password.unwrap(), "hidden");
        assert_eq!(config.db_name, "testdb");
        assert_eq!(config.host, "localhost");
        assert!(config.port.is_none());

        return Ok(());
      }

      #[test]
      fn it_should_return_valid_db_config_with_complete_uri() -> ResultDynError<()> {
        let config = DbConnectionConfig::from("yay:hidden@localhost:5544/testdb")?;

        assert_eq!(config.username, "yay");
        assert_eq!(config.password.unwrap(), "hidden");
        assert_eq!(config.db_name, "testdb");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port.unwrap(), "5544");

        return Ok(());
      }
    }
  }
}
