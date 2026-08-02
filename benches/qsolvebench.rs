use std::{hint::black_box, str::FromStr, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use qsolve::{
    board::Board,
    heuristic::all_heuristics,
    solveiter::solve_iter,
    solvestate::{SolveState, SolveStrategy},
};

fn benchmark_puzzle(c: &mut Criterion, name: &str, file: &str) {
    let mut g = c.benchmark_group(name);
    g.measurement_time(Duration::from_secs(10));
    g.sample_size(500);
    for (label, strategy) in [
        ("Fast", SolveStrategy::Fast),
        ("Simple", SolveStrategy::Simple),
        ("Short", SolveStrategy::Short),
    ] {
        g.bench_function(label, |b| {
            let content = std::fs::read_to_string(file).unwrap();
            let board = Board::from_str(&content).unwrap();
            b.iter(|| {
                let solve_state = SolveState::from(&board);
                let heuristics = all_heuristics(black_box(&board));
                let state_iter_items =
                    solve_iter(solve_state, strategy, &heuristics).collect::<Vec<_>>();
                let _final_state = black_box(&state_iter_items.iter().last().unwrap().solve_state);
            })
        });
    }
    g.finish();
}

fn benchmark_board_construction(c: &mut Criterion) {
    let mut g = c.benchmark_group("BoardConstruction");
    g.measurement_time(Duration::from_secs(10));
    g.sample_size(500);
    for (name, file) in [
        ("LinkedIn1", "games/linkedin-1-empty.txt"),
        ("GameOfCrowns1", "games/gameofcrowns-1.txt"),
    ] {
        let content = std::fs::read_to_string(file).unwrap();
        g.bench_function(name, |b| {
            b.iter(|| black_box(Board::from_str(black_box(content.as_str())).unwrap()))
        });
    }
    g.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_puzzle(c, "LinkedIn1", "games/linkedin-1-empty.txt");
    benchmark_puzzle(c, "GameOfCrowns1", "games/gameofcrowns-1.txt");
    benchmark_board_construction(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
