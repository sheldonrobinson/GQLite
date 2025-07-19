use std::{collections::HashSet, fs};

use ccutils::temporary::TemporaryFile;
use divan::Bencher;
use gqlitedb::{map, Connection};
use rand::{rngs::StdRng, seq::IndexedRandom, SeedableRng};
use regex::Regex;

const FRIEND_OF_COUNT: u32 = 100;
const SINGLE_VERTEX_COUNT: u32 = 100;
const IMPORT_COUNT: u32 = 30;

fn main()
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
  // Run registered benchmarks.
  divan::main();
}

fn load_pokec(backend: &str) -> (TemporaryFile, Connection)
{
  let temporary_file = TemporaryFile::builder()
    .should_create_file(false)
    .label("gqlite_bench")
    .create();

  let connection = Connection::open(temporary_file.path(), map!("backend" => backend)).unwrap();

  let import_query = fs::read_to_string("gqlite_bench_data/pokec_tiny_import.cypher").unwrap();

  connection
    .execute_query(import_query, Default::default())
    .unwrap();
  (temporary_file, connection)
}

fn read_pokec_ids() -> Vec<i64>
{
  let re = Regex::new(r"id:\s*(\d+)").unwrap();
  let mut ids = HashSet::new();
  let content = fs::read_to_string("gqlite_bench_data/pokec_tiny_import.cypher")
    .expect("Failed to read the file");
  for cap in re.captures_iter(&content)
  {
    if let Some(id_match) = cap.get(1)
    {
      let id = id_match.as_str().parse::<i64>().unwrap();
      ids.insert(id);
    }
  }
  ids.into_iter().collect()
}

// Import the tiny `pokec` dataset.
#[divan::bench(args = ["sqlite", "redb"], sample_count = IMPORT_COUNT)]
fn import_tiny_pokec(backend: &str)
{
  let (_, _) = load_pokec(backend);
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = SINGLE_VERTEX_COUNT)]
fn single_vertex(bencher: Bencher, backend: &str)
{
  let (_, connection) = load_pokec(backend);
  let ids = read_pokec_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    let random_id = ids.choose(&mut rng).unwrap();
    connection
      .execute_query(
        "MATCH (n:User {id: $id}) RETURN n",
        map!("$id" => *random_id),
      )
      .unwrap();
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = SINGLE_VERTEX_COUNT)]
fn single_vertex_where(bencher: Bencher, backend: &str)
{
  let (_, connection) = load_pokec(backend);
  let ids = read_pokec_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    let random_id = ids.choose(&mut rng).unwrap();
    connection
      .execute_query(
        "MATCH (n:User) WHERE n.id = $id RETURN n",
        map!("$id" => *random_id),
      )
      .unwrap();
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = FRIEND_OF_COUNT)]
fn friend_of(bencher: Bencher, backend: &str)
{
  let (_, connection) = load_pokec(backend);
  let ids = read_pokec_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    let random_id = ids.choose(&mut rng).unwrap();
    connection
      .execute_query(
        "MATCH (s:User {id: $id})-->(n:User) RETURN n.id",
        map!("$id" => *random_id),
      )
      .unwrap();
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = FRIEND_OF_COUNT)]
fn friend_of_filter(bencher: Bencher, backend: &str)
{
  let (_, connection) = load_pokec(backend);
  let ids = read_pokec_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    let random_id = ids.choose(&mut rng).unwrap();
    connection
      .execute_query(
        "MATCH (s:User {id: $id})-->(n:User) WHERE n.age >= 18 RETURN n.id",
        map!("$id" => *random_id),
      )
      .unwrap();
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = FRIEND_OF_COUNT)]
fn friend_of_friend_of(bencher: Bencher, backend: &str)
{
  let (_, connection) = load_pokec(backend);
  let ids = read_pokec_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    let random_id = ids.choose(&mut rng).unwrap();
    connection
      .execute_query(
        "MATCH (s:User {id: $id})-->()-->(n:User) RETURN n.id",
        map!("$id" => *random_id),
      )
      .unwrap();
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = FRIEND_OF_COUNT)]
fn friend_of_friend_of_filter(bencher: Bencher, backend: &str)
{
  let (_, connection) = load_pokec(backend);
  let ids = read_pokec_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    let random_id = ids.choose(&mut rng).unwrap();
    connection
      .execute_query(
        "MATCH (s:User {id: $id})-->()-->(n:User) WHERE n.age >= 18 RETURN n.id",
        map!("$id" => *random_id),
      )
      .unwrap();
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = FRIEND_OF_COUNT)]
fn reciprocal_friends(bencher: Bencher, backend: &str)
{
  let (_, connection) = load_pokec(backend);
  let ids = read_pokec_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    let random_id = ids.choose(&mut rng).unwrap();
    connection
      .execute_query(
        "MATCH (n:User {id: $id})-[e1]->(m)-[e2]->(n) RETURN e1, m, e2",
        map!("$id" => *random_id),
      )
      .unwrap();
  });
}

#[divan::bench(args = ["sqlite", "redb"])]
fn aggregate_count(bencher: Bencher, backend: &str)
{
  let (_, connection) = load_pokec(backend);

  bencher.bench_local(move || {
    connection
      .execute_query("MATCH (n:User) RETURN n.age, count(*)", Default::default())
      .unwrap();
  });
}

#[divan::bench(args = ["sqlite", "redb"])]
fn aggregate_count_filter(bencher: Bencher, backend: &str)
{
  let (_, connection) = load_pokec(backend);

  bencher.bench_local(move || {
    connection
      .execute_query(
        "MATCH (n:User) WHERE n.age >= 18 RETURN n.age, count(*)",
        Default::default(),
      )
      .unwrap();
  });
}

#[divan::bench(args = ["sqlite", "redb"])]
fn aggregate_min_max_avg(bencher: Bencher, backend: &str)
{
  let (_, connection) = load_pokec(backend);

  bencher.bench_local(move || {
    connection
      .execute_query(
        "MATCH (n) RETURN min(n.age), max(n.age), avg(n.age)",
        Default::default(),
      )
      .unwrap();
  });
}
