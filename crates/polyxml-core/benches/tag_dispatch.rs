//! Tag-dispatch strategy comparison at schema scale tiers.
//!
//! Baselines:
//!   1. Standard Rust `match` on string literals (LLVM bucketed memcmp chains)
//!   2. Runtime `HashMap<&str, u32>`
//!   3. Compile-time `phf::Map<&str, u32>`
//!
//! Each iteration sweeps the full tier (hit path for every tag); Criterion's
//! `Throughput::Elements` reports tags/second directly. Tokenization is
//! intentionally isolated OUT of the bench so lookup cost can be measured
//! independently of parsing (see
//! docs/benchmarks/rust-phf-dispatch.md).
//!
//! Regenerate the tag tables with scripts/gen_tag_dispatch_fixtures.py.

use std::collections::HashMap;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

#[path = "support/tag_dispatch_fixtures.rs"]
mod fixtures;

fn bench_tier(
    c: &mut Criterion,
    tier: &str,
    tags: &[&'static str],
    match_fn: fn(&str) -> Option<u32>,
    phf_map: &'static phf::Map<&'static str, u32>,
) {
    let hashmap: HashMap<&'static str, u32> = tags
        .iter()
        .enumerate()
        .map(|(i, tag)| (*tag, i as u32))
        .collect();
    // A tag present in no strategy: the miss path matters for unknown children.
    let miss = format!("Unknown{tier}Sentinel");

    let mut group = c.benchmark_group(format!("tag_dispatch/{tier}"));
    group.throughput(Throughput::Elements(tags.len() as u64));

    group.bench_function("match_hit", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            for tag in tags {
                acc += u64::from(match_fn(black_box(tag)).unwrap_or(u32::MAX));
            }
            black_box(acc);
        });
    });
    group.bench_function("match_miss", |b| {
        let miss = black_box(miss.as_str());
        b.iter(|| black_box(match_fn(miss)));
    });

    group.bench_function("hashmap_hit", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            for tag in tags {
                acc += u64::from(*hashmap.get(black_box(tag)).unwrap_or(&u32::MAX));
            }
            black_box(acc);
        });
    });
    group.bench_function("hashmap_miss", |b| {
        let miss = black_box(miss.as_str());
        b.iter(|| black_box(hashmap.get(miss)));
    });

    group.bench_function("phf_hit", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            for tag in tags {
                acc += u64::from(*phf_map.get(black_box(tag)).unwrap_or(&u32::MAX));
            }
            black_box(acc);
        });
    });
    group.bench_function("phf_miss", |b| {
        let miss = black_box(miss.as_str());
        b.iter(|| black_box(phf_map.get(miss)));
    });

    group.finish();
}

fn tag_dispatch(c: &mut Criterion) {
    bench_tier(
        c,
        "small_16",
        &fixtures::TAGS_16,
        fixtures::match_dispatch_16,
        &fixtures::PHF_16,
    );
    bench_tier(
        c,
        "medium_120",
        &fixtures::TAGS_120,
        fixtures::match_dispatch_120,
        &fixtures::PHF_120,
    );
    bench_tier(
        c,
        "large_600",
        &fixtures::TAGS_600,
        fixtures::match_dispatch_600,
        &fixtures::PHF_600,
    );
    bench_tier(
        c,
        "large_1500",
        &fixtures::TAGS_1500,
        fixtures::match_dispatch_1500,
        &fixtures::PHF_1500,
    );
}

criterion_group!(benches, tag_dispatch);
criterion_main!(benches);
