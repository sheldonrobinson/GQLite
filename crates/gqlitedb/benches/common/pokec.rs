use std::{collections::HashSet, fs};

use ccutils::temporary::TemporaryFile;
use gqlitedb::{value_map, Connection};
use rand::{seq::IndexedRandom, Rng};
use regex::Regex;

pub(crate) struct Pokec
{
  #[allow(dead_code)]
  temporary_file: TemporaryFile,
  connection: Connection,
  ids: Vec<i64>,
}

#[allow(dead_code)]
pub(crate) enum PokecSize
{
  Micro,
  Tiny,
}

impl Pokec
{
  pub(crate) fn load(backend: &str, size: PokecSize) -> Pokec
  {
    let backend = match backend
    {
      "sqlite" => gqlitedb::Backend::SQLite,
      "redb" => gqlitedb::Backend::Redb,
      o => panic!("Unknown backend '{}'", o),
    };
    let temporary_file = TemporaryFile::builder()
      .should_create_file(false)
      .label("gqlite_bench")
      .create();
    let connection = Connection::builder()
      .path(temporary_file.path())
      .backend(backend)
      .create()
      .unwrap();

    let filename = match size
    {
      PokecSize::Micro => "gqlite_bench_data/pokec_micro_import.cypher",
      PokecSize::Tiny => "gqlite_bench_data/pokec_tiny_import.cypher",
    };

    let import_query = fs::read_to_string(filename).unwrap();

    connection
      .execute_oc_query(import_query, Default::default())
      .unwrap();
    Self {
      temporary_file,
      connection,
      ids: Default::default(),
    }
  }
  pub(crate) fn read_ids(mut self) -> Self
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
    self.ids = ids.into_iter().collect();
    self
  }
  pub(crate) fn single_vertex<R>(&self, rng: &mut R)
  where
    R: Rng + ?Sized,
  {
    let random_id = self.ids.choose(rng).unwrap();
    self
      .connection
      .execute_oc_query(
        "MATCH (n:User {id: $id}) RETURN n",
        value_map!("$id" => *random_id),
      )
      .unwrap();
  }
  pub(crate) fn single_vertex_where<R>(&self, rng: &mut R)
  where
    R: Rng + ?Sized,
  {
    let random_id = self.ids.choose(rng).unwrap();
    self
      .connection
      .execute_oc_query(
        "MATCH (n:User) WHERE n.id = $id RETURN n",
        value_map!("$id" => *random_id),
      )
      .unwrap();
  }
  pub(crate) fn friend_of<R>(&self, rng: &mut R)
  where
    R: Rng + ?Sized,
  {
    let random_id = self.ids.choose(rng).unwrap();
    self
      .connection
      .execute_oc_query(
        "MATCH (s:User {id: $id})-->(n:User) RETURN n.id",
        value_map!("$id" => *random_id),
      )
      .unwrap();
  }
  pub(crate) fn friend_of_filter<R>(&self, rng: &mut R)
  where
    R: Rng + ?Sized,
  {
    let random_id = self.ids.choose(rng).unwrap();
    self
      .connection
      .execute_oc_query(
        "MATCH (s:User {id: $id})-->(n:User) WHERE n.age >= 18 RETURN n.id",
        value_map!("$id" => *random_id),
      )
      .unwrap();
  }
  pub(crate) fn friend_of_friend_of<R>(&self, rng: &mut R)
  where
    R: Rng + ?Sized,
  {
    let random_id = self.ids.choose(rng).unwrap();
    self
      .connection
      .execute_oc_query(
        "MATCH (s:User {id: $id})-->()-->(n:User) RETURN n.id",
        value_map!("$id" => *random_id),
      )
      .unwrap();
  }
  pub(crate) fn friend_of_friend_of_filter<R>(&self, rng: &mut R)
  where
    R: Rng + ?Sized,
  {
    let random_id = self.ids.choose(rng).unwrap();
    self
      .connection
      .execute_oc_query(
        "MATCH (s:User {id: $id})-->()-->(n:User) WHERE n.age >= 18 RETURN n.id",
        value_map!("$id" => *random_id),
      )
      .unwrap();
  }
  pub(crate) fn reciprocal_friends<R>(&self, rng: &mut R)
  where
    R: Rng + ?Sized,
  {
    let random_id = self.ids.choose(rng).unwrap();
    self
      .connection
      .execute_oc_query(
        "MATCH (n:User {id: $id})-[e1]->(m)-[e2]->(n) RETURN e1, m, e2",
        value_map!("$id" => *random_id),
      )
      .unwrap();
  }
  pub(crate) fn aggregate_count(&self)
  {
    self
      .connection
      .execute_oc_query("MATCH (n:User) RETURN n.age, count(*)", Default::default())
      .unwrap();
  }
  pub(crate) fn aggregate_count_filter(&self)
  {
    self
      .connection
      .execute_oc_query(
        "MATCH (n:User) WHERE n.age >= 18 RETURN n.age, count(*)",
        Default::default(),
      )
      .unwrap();
  }
  pub(crate) fn aggregate_min_max_avg(&self)
  {
    self
      .connection
      .execute_oc_query(
        "MATCH (n) RETURN min(n.age), max(n.age), avg(n.age)",
        Default::default(),
      )
      .unwrap();
  }
}
