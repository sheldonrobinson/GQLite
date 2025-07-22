use divan::Bencher;
use rand::{rngs::StdRng, SeedableRng};

mod common;

use common::pokec::{Pokec, PokecSize};

const FRIEND_OF_COUNT: u32 = 100;
const SINGLE_VERTEX_COUNT: u32 = 100;
const IMPORT_COUNT: u32 = 30;

fn main()
{
  common::get_bench_data();

  // Run registered benchmarks.
  divan::main();
}

fn load_pokec(backend: &str) -> Pokec
{
  Pokec::load(backend, PokecSize::Tiny)
}

// Import the tiny `pokec` dataset.
#[divan::bench(args = ["sqlite", "redb"], sample_count = IMPORT_COUNT)]
fn import_tiny_pokec(backend: &str)
{
  let _ = load_pokec(backend);
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = SINGLE_VERTEX_COUNT)]
fn single_vertex(bencher: Bencher, backend: &str)
{
  let tiny_pokec = load_pokec(backend).read_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    tiny_pokec.single_vertex(&mut rng);
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = SINGLE_VERTEX_COUNT)]
fn single_vertex_where(bencher: Bencher, backend: &str)
{
  let tiny_pokec = load_pokec(backend).read_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    tiny_pokec.single_vertex_where(&mut rng);
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = FRIEND_OF_COUNT)]
fn friend_of(bencher: Bencher, backend: &str)
{
  let tiny_pokec = load_pokec(backend).read_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    tiny_pokec.friend_of(&mut rng);
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = FRIEND_OF_COUNT)]
fn friend_of_filter(bencher: Bencher, backend: &str)
{
  let tiny_pokec = load_pokec(backend).read_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    tiny_pokec.friend_of_filter(&mut rng);
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = FRIEND_OF_COUNT)]
fn friend_of_friend_of(bencher: Bencher, backend: &str)
{
  let tiny_pokec = load_pokec(backend).read_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    tiny_pokec.friend_of_friend_of(&mut rng);
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = FRIEND_OF_COUNT)]
fn friend_of_friend_of_filter(bencher: Bencher, backend: &str)
{
  let tiny_pokec = load_pokec(backend).read_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    tiny_pokec.friend_of_friend_of_filter(&mut rng);
  });
}

#[divan::bench(args = ["sqlite", "redb"], sample_count = FRIEND_OF_COUNT)]
fn reciprocal_friends(bencher: Bencher, backend: &str)
{
  let tiny_pokec = load_pokec(backend).read_ids();
  let mut rng = StdRng::seed_from_u64(991173);

  bencher.bench_local(move || {
    tiny_pokec.reciprocal_friends(&mut rng);
  });
}

#[divan::bench(args = ["sqlite", "redb"])]
fn aggregate_count(bencher: Bencher, backend: &str)
{
  let tiny_pokec = load_pokec(backend);

  bencher.bench_local(move || {
    tiny_pokec.aggregate_count();
  });
}

#[divan::bench(args = ["sqlite", "redb"])]
fn aggregate_count_filter(bencher: Bencher, backend: &str)
{
  let tiny_pokec = load_pokec(backend);

  bencher.bench_local(move || {
    tiny_pokec.aggregate_count_filter();
  });
}

#[divan::bench(args = ["sqlite", "redb"])]
fn aggregate_min_max_avg(bencher: Bencher, backend: &str)
{
  let tiny_pokec = load_pokec(backend);

  bencher.bench_local(move || {
    tiny_pokec.aggregate_min_max_avg();
  });
}
