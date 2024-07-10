mod capi;
mod connection;
mod error;
mod graph;
mod interpreter;
mod parser;
mod store;

pub type Error = error::Error;
pub type Result<T> = std::result::Result<T, Error>;
pub type Connection = connection::Connection;


#[cfg(test)]
pub(crate) mod tests
{
  use rand::Rng;
  pub(crate) fn get_tmp_file() -> Result<std::path::PathBuf, std::io::Error> {
    let mut rng = rand::thread_rng();
    loop {
      let rand: u32 = rng.gen();
      let path = std::path::PathBuf::from(format!("{}/tmp_gqlite_{}", std::env::temp_dir().to_str().unwrap(), rand));
      return Ok(path)
    }
  }
}