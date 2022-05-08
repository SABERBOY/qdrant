mod prof;

use std::sync::atomic::Ordering;
use std::time::Instant;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{thread_rng, SeedableRng};
use segment::fixtures::index_fixtures::{random_vector, FakeFilterContext, TestRawScorerProducer};
use segment::index::hnsw_index::graph_layers::GraphLayers;
use segment::index::hnsw_index::point_scorer::FilteredScorer;
use segment::spaces::simple::{CosineMetric, DotProductMetric};
use segment::types::PointOffsetType;

const NUM_VECTORS: usize = 200000;
const DIM: usize = 16;
const M: usize = 16;
const TOP: usize = 10;
const EF_CONSTRUCT: usize = 100;
const EF: usize = 100;
const USE_HEURISTIC: bool = true;

fn hnsw_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let vector_holder = TestRawScorerProducer::new(DIM, NUM_VECTORS, DotProductMetric {}, &mut rng);
    let mut group = c.benchmark_group("hnsw-index-search-group");
    let mut rng = thread_rng();
    let fake_filter_context = FakeFilterContext {};

    let now = Instant::now();

    let mut graph_layers = GraphLayers::new(NUM_VECTORS, M, M * 2, EF_CONSTRUCT, 10, USE_HEURISTIC);
    for idx in 0..(NUM_VECTORS as PointOffsetType) {
        let added_vector = vector_holder.vectors.get(idx).to_vec();
        let raw_scorer = vector_holder.get_raw_scorer(added_vector);
        let scorer = FilteredScorer::new(&raw_scorer, Some(&fake_filter_context));
        let level = graph_layers.get_random_layer(&mut rng);
        graph_layers.link_new_point(idx, level, scorer);
    }

    let build_duration = now.elapsed().as_secs_f64();
    eprintln!("build_duration = {:#?}", build_duration);

    let mut total_cmps = 0;
    let mut iterations = 0;
    group.bench_function("hnsw_search", |b| {
        b.iter(|| {
            let query = random_vector(&mut rng, DIM);

            let raw_scorer = vector_holder.get_raw_scorer(query);
            let scorer = FilteredScorer::new(&raw_scorer, Some(&fake_filter_context));
            graph_layers.search(TOP, EF, scorer);
            iterations += 1;
            total_cmps += raw_scorer.num_comparisons.load(Ordering::SeqCst);
        })
    });

    eprintln!("total_cmps / iterations = {:#?}", total_cmps / iterations);

    let mut plain_search_range: Vec<PointOffsetType> =
        (0..NUM_VECTORS as PointOffsetType).collect();

    group.bench_function("plain_search", |b| {
        b.iter(|| {
            let query = random_vector(&mut rng, DIM);

            let raw_scorer = vector_holder.get_raw_scorer(query);
            let mut scorer = FilteredScorer::new(&raw_scorer, Some(&fake_filter_context));

            let mut top_score = 0.;
            let scores = scorer.score_points(&mut plain_search_range, NUM_VECTORS);
            scores.iter().copied().for_each(|score| {
                if score.score > top_score {
                    top_score = score.score
                }
            });
        })
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(prof::FlamegraphProfiler::new(100));
    targets = hnsw_benchmark
}

criterion_main!(benches);
