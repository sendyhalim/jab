use std::io;
use std::process::Command;

pub struct DumpInput<'a> {
  pub db_uri: &'a str,
}

pub fn dump(input: DumpInput) -> Result<String, Box<dyn std::error::Error>> {
  let output = Command::new("pg_dump")
    .arg(format!("postgres://{}", input.db_uri))
    .output()?;

  let err = String::from_utf8(output.stderr)?;

  if err.len() > 0 {
    let err = io::Error::new(io::ErrorKind::Other, err);

    return Err(Box::new(err));
  }

  let output = String::from_utf8(output.stdout)?;

  return Ok(output);
}

pub struct RestoreInput<'a> {
  pub db_uri: &'a str,
  pub sql_str: &'a str,
}

pub fn restore(input: RestoreInput) {
  // Drop db
  // Restore from sql_str
}
