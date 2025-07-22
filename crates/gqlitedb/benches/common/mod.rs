pub(crate) mod pokec;

pub(crate) fn get_bench_data()
{
  if std::fs::exists("gqlite_bench_data").unwrap()
  {
    std::process::Command::new("git")
      .arg("pull")
      .spawn()
      .unwrap()
      .wait()
      .unwrap();
  }
  else
  {
    std::process::Command::new("git")
      .arg("clone")
      .arg("https://gitlab.com/auksys/data/gqlite_bench.git")
      .arg("gqlite_bench_data")
      .spawn()
      .unwrap()
      .wait()
      .unwrap();
  }
}
