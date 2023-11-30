use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, Bencher, BenchmarkGroup, Criterion};
use itertools::Itertools;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use std::fmt;
use std::fmt::Write;

/// Source: https://stackoverflow.com/a/26647446
fn levans_iterators(out: &mut dyn Write, input: &[&str]) -> fmt::Result {
    for e in input.iter().take(1) {
        write!(out, "{}", e)?;
    }

    for e in input.iter().skip(1) {
        write!(out, ", {}", e)?;
    }

    Ok(())
}

/// Source: https://stackoverflow.com/a/26647446
fn levans_fold(out: &mut dyn Write, input: &[&str]) -> fmt::Result {
    input.iter().try_fold(true, |first, elem| {
        if !first {
            write!(out, ", ")?;
        }
        write!(out, "{}", elem)?;
        Ok(false)
    })?;

    Ok(())
}

/// Source: https://stackoverflow.com/a/45134036
fn shepmaster_iter(out: &mut dyn Write, input: &[&str]) -> fmt::Result {
    let mut iter = input.iter();
    if let Some(item) = iter.next() {
        write!(out, "{}", item)?;

        for item in iter {
            write!(out, ", {}", item)?;
        }
    }

    Ok(())
}

fn itertools_format(out: &mut dyn Write, input: &[&str]) -> fmt::Result {
    write!(out, "{}", input.iter().format(", "))
}

/// Source: https://stackoverflow.com/a/26644600
fn chris_morgan(out: &mut dyn Write, input: &[&str]) -> fmt::Result {
    let mut first = true;
    for item in input {
        if !first {
            write!(out, ", ")?;
        }
        write!(out, "{}", item)?;
        first = false;
    }

    Ok(())
}

/// Source: https://stackoverflow.com/a/63878278
fn zombo(out: &mut dyn Write, input: &[&str]) -> fmt::Result {
    for (n, s) in input.iter().enumerate() {
        if n > 0 {
            write!(out, ", ")?;
        }
        write!(out, "{}", s)?;
    }

    Ok(())
}

fn fmttools_join(out: &mut dyn Write, input: &[&str]) -> fmt::Result {
    write!(out, "{}", fmttools::join(input, ", "))
}

fn naive_join(out: &mut dyn Write, input: &[&str]) -> fmt::Result {
    write!(out, "{}", input.join(", "))
}

/// Skip the overhead of the write! macro and just write strings directly
fn direct(out: &mut dyn Write, input: &[&str]) -> fmt::Result {
    let mut iter = input.iter();
    if let Some(item) = iter.next() {
        out.write_str(item)?;

        for item in iter {
            out.write_str(", ")?;
            out.write_str(item)?;
        }
    }

    Ok(())
}

fn bench_fn<F>(func: F) -> impl Fn(&mut Bencher, &[&str])
where
    F: Fn(&mut dyn Write, &[&str]) -> fmt::Result,
{
    move |b, input| {
        let buffer_size = input.iter().map(|x| x.len()).sum::<usize>() + 2 * input.len();
        let mut buffer = String::with_capacity(buffer_size);

        b.iter(|| {
            buffer.clear();
            let dyn_buffer: &mut dyn Write = black_box(&mut buffer);
            func(dyn_buffer, input).unwrap();
        })
    }
}

fn run_group_for_input(mut c: BenchmarkGroup<WallTime>, input: &[&str]) {
    c.bench_with_input("levans_iterators", input, bench_fn(levans_iterators));
    c.bench_with_input("levans_fold", input, bench_fn(levans_fold));
    c.bench_with_input("chris_morgan", input, bench_fn(chris_morgan));
    c.bench_with_input("zombo", input, bench_fn(zombo));
    c.bench_with_input("shepmaster_iter", input, bench_fn(shepmaster_iter));
    c.bench_with_input("itertools_format", input, bench_fn(itertools_format));
    c.bench_with_input("fmttools_join", input, bench_fn(fmttools_join));
    c.bench_with_input("naive_join", input, bench_fn(naive_join));
    c.bench_with_input("direct", input, bench_fn(direct));
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = ChaChaRng::seed_from_u64(123456789);
    let mut input: Vec<&str> = Vec::new();
    for _ in 0..10000 {
        // Format random integers to get our string values
        let string = format!("{}", rng.gen::<i32>());
        let leaked: &'static mut str = Box::leak(Box::from(string));
        input.push(leaked);
    }

    run_group_for_input(c.benchmark_group("empty"), &[]);
    run_group_for_input(c.benchmark_group("one"), &input[..1]);
    run_group_for_input(c.benchmark_group("small"), &input[..10]);
    run_group_for_input(c.benchmark_group("medium"), &input[..100]);
    run_group_for_input(c.benchmark_group("large"), &input[..10000]);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
