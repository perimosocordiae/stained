use clap::Parser;
use stained::{agent, game};

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value_t = 2)]
    players: usize,
    #[clap(short, long)]
    quiet: bool,
    #[clap(long, default_value_t = 1)]
    repeats: usize,
    #[clap(long, value_parser, value_delimiter = ',', default_value = "0,1")]
    ai_levels: Vec<usize>,
}

struct RunInfo {
    winner_score: i32,
    winner_unfilled: i32,
    winner_idx: usize,
}

fn run_game(args: &Args) -> Option<RunInfo> {
    let mut g = match game::GameState::init(args.players) {
        Ok(game) => game,
        Err(e) => {
            eprintln!("Error creating game state: {e}");
            return None;
        }
    };
    let ais = (0..args.players)
        .map(|i| agent::create_agent(args.ai_levels[i % args.ai_levels.len()]))
        .collect::<Vec<_>>();
    loop {
        if !args.quiet {
            println!("P{}: {:?}", g.curr_player_idx, g.phase);
            g.current_player().pretty_print();
        }
        let act = ais[g.curr_player_idx].choose_action(&g);
        if !args.quiet {
            println!(" {act:?}");
        }
        match g.take_turn(&act) {
            Ok(true) => {
                let scores = g.player_scores();
                let winner_idx = g.winner_idx()?;
                if !args.quiet {
                    println!("Game over: winner={winner_idx}",);
                }
                return Some(RunInfo {
                    winner_score: scores[winner_idx].total(),
                    winner_unfilled: -scores[winner_idx].empty_slots,
                    winner_idx,
                });
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!(
                    "Error processing {act:?} for player {}:\n{e}",
                    g.curr_player_idx
                );
                return None;
            }
        }
    }
}

struct Stats {
    count: usize,
    min: i32,
    max: i32,
    sum: i32,
    sum_sq: i32,
}
impl Stats {
    fn new() -> Self {
        Stats {
            count: 0,
            min: i32::MAX,
            max: i32::MIN,
            sum: 0,
            sum_sq: 0,
        }
    }

    fn add(&mut self, value: i32) {
        self.count += 1;
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
        self.sum += value;
        self.sum_sq += value * value;
    }

    fn report(&self, name: &str) {
        let mean = self.sum as f64 / self.count as f64;
        let variance = (self.sum_sq as f64 / self.count as f64) - (mean * mean);
        println!(
            "{name}: min={}, max={}, mean={mean:.2}, stddev={:.2}",
            self.min,
            self.max,
            variance.sqrt()
        );
    }
}

fn main() {
    let args = Args::parse();
    let mut time_stats = Stats::new();
    let mut score_stats = Stats::new();
    let mut unfilled_stats = Stats::new();
    let mut win_counts = vec![0; args.players];
    for _ in 0..args.repeats {
        let start_time = std::time::Instant::now();
        if let Some(info) = run_game(&args) {
            score_stats.add(info.winner_score);
            unfilled_stats.add(info.winner_unfilled);
            win_counts[info.winner_idx] += 1;
        }
        let elapsed = start_time.elapsed();
        time_stats.add(elapsed.as_micros() as i32);
    }
    println!(
        "{} out of {} games were successful",
        score_stats.count, args.repeats
    );
    time_stats.report("Time (us)");
    if score_stats.count > 0 {
        score_stats.report("Top Score");
        unfilled_stats.report("Unfilled ");
    }
    for (i, count) in win_counts.iter().enumerate() {
        println!("Player {i}: {count} wins");
    }
}
