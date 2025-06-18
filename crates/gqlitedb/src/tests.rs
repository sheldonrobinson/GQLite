mod compiler;
mod evaluators;
mod parser;
mod templates;

pub(crate) fn get_tmp_file() -> Result<std::path::PathBuf, std::io::Error>
{
  use rand::Rng;
  let mut rng = rand::rng();
  loop
  {
    let rand: u32 = rng.random();
    let path = std::path::PathBuf::from(format!(
      "{}/tmp_gqlite_{}",
      std::env::temp_dir().to_str().unwrap(),
      rand
    ));
    if !path.exists()
    {
      return Ok(path);
    }
  }
}
