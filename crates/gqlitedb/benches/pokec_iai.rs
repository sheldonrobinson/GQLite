use std::hint::black_box;

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use rand::{rngs::StdRng, SeedableRng};

mod common;

use common::pokec::{Pokec, PokecSize};

fn load_pokec(backend: &str) -> Pokec
{
  Pokec::load(backend, PokecSize::Micro)
}

fn load_with_ids(backend: &str) -> Pokec
{
  load_pokec(backend).read_ids()
}

#[library_benchmark]
#[bench::redb("redb")]
#[bench::sqlite("sqlite")]
fn import_micro_pokec(backend: &str)
{
  black_box(load_pokec(backend));
}

#[library_benchmark(setup = load_with_ids)]
#[bench::redb("redb")]
#[bench::sqlite("sqlite")]
fn single_vertex(micro_pokec: Pokec)
{
  let mut rng = StdRng::seed_from_u64(991173);
  micro_pokec.single_vertex(&mut rng);
}

#[library_benchmark(setup = load_with_ids)]
#[bench::redb("redb")]
#[bench::sqlite("sqlite")]
fn single_vertex_where(micro_pokec: Pokec)
{
  let mut rng = StdRng::seed_from_u64(991173);
  micro_pokec.single_vertex_where(&mut rng);
}

#[library_benchmark(setup = load_with_ids)]
#[bench::redb("redb")]
#[bench::sqlite("sqlite")]
fn friend_of(micro_pokec: Pokec)
{
  let mut rng = StdRng::seed_from_u64(991173);
  micro_pokec.friend_of(&mut rng);
}

#[library_benchmark(setup = load_with_ids)]
#[bench::redb("redb")]
#[bench::sqlite("sqlite")]
fn friend_of_filter(micro_pokec: Pokec)
{
  let mut rng = StdRng::seed_from_u64(991173);
  micro_pokec.friend_of_filter(&mut rng);
}

#[library_benchmark(setup = load_with_ids)]
#[bench::redb("redb")]
#[bench::sqlite("sqlite")]
fn friend_of_friend_of(micro_pokec: Pokec)
{
  let mut rng = StdRng::seed_from_u64(991173);
  micro_pokec.friend_of_friend_of(&mut rng);
}

#[library_benchmark(setup = load_with_ids)]
#[bench::redb("redb")]
#[bench::sqlite("sqlite")]
fn friend_of_friend_of_filter(micro_pokec: Pokec)
{
  let mut rng = StdRng::seed_from_u64(991173);
  micro_pokec.friend_of_friend_of_filter(&mut rng);
}

#[library_benchmark(setup = load_with_ids)]
#[bench::redb("redb")]
#[bench::sqlite("sqlite")]
fn reciprocal_friends(micro_pokec: Pokec)
{
  let mut rng = StdRng::seed_from_u64(991173);
  micro_pokec.reciprocal_friends(&mut rng);
}

#[library_benchmark(setup = load_pokec)]
#[bench::redb("redb")]
#[bench::sqlite("sqlite")]
fn aggregate_count(micro_pokec: Pokec)
{
  micro_pokec.aggregate_count();
}

#[library_benchmark(setup = load_pokec)]
#[bench::redb("redb")]
#[bench::sqlite("sqlite")]
fn aggregate_count_filter(micro_pokec: Pokec)
{
  micro_pokec.aggregate_count_filter();
}

#[library_benchmark(setup = load_pokec)]
#[bench::redb("redb")]
#[bench::sqlite("sqlite")]
fn aggregate_min_max_avg(micro_pokec: Pokec)
{
  micro_pokec.aggregate_min_max_avg();
}

library_benchmark_group!(
    name = bench_micro_pokec_group;
    benchmarks = import_micro_pokec, single_vertex, single_vertex_where, friend_of, friend_of_filter, friend_of_friend_of, friend_of_friend_of_filter, reciprocal_friends, aggregate_count, aggregate_count_filter, aggregate_min_max_avg
);

main!(
  setup = common::get_bench_data(); 
  library_benchmark_groups = bench_micro_pokec_group);
