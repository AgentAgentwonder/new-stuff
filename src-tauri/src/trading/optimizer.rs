use super::backtesting::{backtest_run, BacktestConfig, BacktestMetrics, BacktestResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationParameter {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub current_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub strategy_id: String,
    pub backtest_config: BacktestConfig,
    pub parameters: Vec<OptimizationParameter>,
    pub method: String, // genetic, grid, random, monte_carlo
    pub population_size: Option<u32>,
    pub generations: Option<u32>,
    pub mutation_rate: Option<f64>,
    pub crossover_rate: Option<f64>,
    pub iterations: Option<u32>,
    pub optimization_target: String, // sharpe_ratio, total_return, etc.
    pub max_drawdown_constraint: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub parameter_set: HashMap<String, f64>,
    pub metrics: BacktestMetrics,
    pub score: f64,
    pub rank: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRun {
    pub id: String,
    pub config: OptimizationConfig,
    pub status: String, // running, completed, failed, cancelled
    pub progress: f64,  // 0-100
    pub results: Vec<OptimizationResult>,
    pub best_result: Option<OptimizationResult>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityAnalysis {
    pub parameter_name: String,
    pub values: Vec<f64>,
    pub metrics: Vec<BacktestMetrics>,
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub parameters: HashMap<String, f64>,
    pub metrics: Option<BacktestMetrics>,
    pub score: f64,
}

#[derive(Debug, Default)]
pub struct OptimizerState {
    pub runs: HashMap<String, OptimizationRun>,
}

pub type SharedOptimizerState = Arc<Mutex<OptimizerState>>;

pub fn register_optimizer_state(app: &tauri::App) {
    let state = Arc::new(Mutex::new(OptimizerState::default()));
    app.manage(state);
}

fn score_metrics(metrics: &BacktestMetrics, target: &str, max_drawdown: Option<f64>) -> f64 {
    if let Some(max_dd) = max_drawdown {
        if metrics.max_drawdown_percent > max_dd {
            return 0.0;
        }
    }

    match target {
        "sharpe_ratio" => metrics.sharpe_ratio,
        "total_return" => metrics.total_return,
        "win_rate" => metrics.win_rate,
        "profit_factor" => metrics.profit_factor,
        "sortino_ratio" => metrics.sortino_ratio,
        _ => metrics.sharpe_ratio,
    }
}

fn mutate_parameter(value: f64, min: f64, max: f64, step: f64, mutation_rate: f64) -> f64 {
    let mut new_value = value;
    if rand::random::<f64>() < mutation_rate {
        let direction = if rand::random::<bool>() { 1.0 } else { -1.0 };
        new_value += direction * step;
        new_value = new_value.clamp(min, max);
    }
    new_value
}

fn crossover(
    parent1: &HashMap<String, f64>,
    parent2: &HashMap<String, f64>,
    crossover_rate: f64,
) -> HashMap<String, f64> {
    let mut child = parent1.clone();
    if rand::random::<f64>() < crossover_rate {
        for (key, value) in parent2 {
            if rand::random::<bool>() {
                child.insert(key.clone(), *value);
            }
        }
    }
    child
}

fn generate_random_candidate(params: &[OptimizationParameter]) -> HashMap<String, f64> {
    let mut candidate = HashMap::new();
    for param in params {
        let steps = ((param.max - param.min) / param.step).max(1.0) as u32;
        let random_step = rand::random::<u32>() % (steps + 1);
        let value = param.min + param.step * random_step as f64;
        candidate.insert(param.name.clone(), value);
    }
    candidate
}

async fn evaluate_candidate(
    candidate: &HashMap<String, f64>,
    config: &OptimizationConfig,
) -> Result<(BacktestMetrics, f64), String> {
    let mut backtest_config = config.backtest_config.clone();

    // Apply parameters to backtest config (e.g., adjust commission/slippage)
    if let Some(commission) = candidate.get("commission_rate") {
        backtest_config.commission_rate = *commission;
    }
    if let Some(slippage) = candidate.get("slippage_rate") {
        backtest_config.slippage_rate = *slippage;
    }

    let result = backtest_run(backtest_config).await?;
    let score = score_metrics(
        &result.metrics,
        &config.optimization_target,
        config.max_drawdown_constraint,
    );

    Ok((result.metrics, score))
}

async fn run_genetic_algorithm(
    config: OptimizationConfig,
    state: Arc<Mutex<OptimizerState>>,
    run_id: String,
) {
    let population_size = config.population_size.unwrap_or(20);
    let generations = config.generations.unwrap_or(30);
    let mutation_rate = config.mutation_rate.unwrap_or(0.15);
    let crossover_rate = config.crossover_rate.unwrap_or(0.6);

    let mut population: Vec<Candidate> = (0..population_size)
        .map(|_| Candidate {
            parameters: generate_random_candidate(&config.parameters),
            metrics: None,
            score: 0.0,
        })
        .collect();

    for generation in 0..generations {
        for candidate in population.iter_mut() {
            if candidate.metrics.is_none() {
                match evaluate_candidate(&candidate.parameters, &config).await {
                    Ok((metrics, score)) => {
                        candidate.metrics = Some(metrics);
                        candidate.score = score;
                    }
                    Err(err) => {
                        let mut state = state.lock().unwrap();
                        if let Some(run) = state.runs.get_mut(&run_id) {
                            run.status = "failed".to_string();
                            run.error = Some(err);
                            run.completed_at = Some(chrono::Utc::now().timestamp_millis());
                        }
                        return;
                    }
                }
            }
        }

        population.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let best_candidate = population.first().cloned();
        if let Some(best) = best_candidate {
            let mut state = state.lock().unwrap();
            if let Some(run) = state.runs.get_mut(&run_id) {
                let mut results: Vec<OptimizationResult> = population
                    .iter()
                    .filter_map(|candidate| {
                        candidate
                            .metrics
                            .as_ref()
                            .map(|metrics| OptimizationResult {
                                parameter_set: candidate.parameters.clone(),
                                metrics: metrics.clone(),
                                score: candidate.score,
                                rank: 0,
                            })
                    })
                    .collect();

                // Sort results by score and update ranks
                results.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                for (rank, result) in results.iter_mut().enumerate() {
                    result.rank = rank + 1;
                }

                run.results = results;
                run.best_result = Some(OptimizationResult {
                    parameter_set: best.parameters.clone(),
                    metrics: best.metrics.clone().unwrap_or_else(|| BacktestMetrics {
                        total_return: 0.0,
                        total_return_percent: 0.0,
                        annualized_return: 0.0,
                        sharpe_ratio: 0.0,
                        sortino_ratio: 0.0,
                        max_drawdown: 0.0,
                        max_drawdown_percent: 0.0,
                        win_rate: 0.0,
                        profit_factor: 0.0,
                        total_trades: 0,
                        winning_trades: 0,
                        losing_trades: 0,
                        average_win: 0.0,
                        average_loss: 0.0,
                        largest_win: 0.0,
                        largest_loss: 0.0,
                        average_trade_duration: 0,
                        exposure_time: 0.0,
                    }),
                    score: best.score,
                    rank: 1,
                });
                run.progress = ((generation + 1) as f64 / generations as f64) * 100.0;
            }
        }

        // Create next generation
        let elites = (population_size as f64 * 0.2).ceil() as usize; // Keep top 20%
        let mut new_population = population[..elites].to_vec();

        while new_population.len() < population_size as usize {
            let parent1 = &population[rand::random::<usize>() % population.len()];
            let parent2 = &population[rand::random::<usize>() % population.len()];

            let mut child_params =
                crossover(&parent1.parameters, &parent2.parameters, crossover_rate);
            for param in &config.parameters {
                if let Some(value) = child_params.get(&param.name).copied() {
                    let mutated =
                        mutate_parameter(value, param.min, param.max, param.step, mutation_rate);
                    child_params.insert(param.name.clone(), mutated);
                }
            }

            new_population.push(Candidate {
                parameters: child_params,
                metrics: None,
                score: 0.0,
            });
        }

        population = new_population;
    }

    let mut state = state.lock().unwrap();
    if let Some(run) = state.runs.get_mut(&run_id) {
        run.status = "completed".to_string();
        run.completed_at = Some(chrono::Utc::now().timestamp_millis());
        run.progress = 100.0;
    }
}

async fn run_random_search(
    config: OptimizationConfig,
    state: Arc<Mutex<OptimizerState>>,
    run_id: String,
) {
    let iterations = config.iterations.unwrap_or(50);
    let mut best_result: Option<OptimizationResult> = None;
    let mut results: Vec<OptimizationResult> = Vec::new();

    for i in 0..iterations {
        let parameters = generate_random_candidate(&config.parameters);
        match evaluate_candidate(&parameters, &config).await {
            Ok((metrics, score)) => {
                let result = OptimizationResult {
                    parameter_set: parameters,
                    metrics,
                    score,
                    rank: 0,
                };

                results.push(result.clone());

                if best_result
                    .as_ref()
                    .map(|best| score > best.score)
                    .unwrap_or(true)
                {
                    best_result = Some(result.clone());
                }

                let mut state = state.lock().unwrap();
                if let Some(run) = state.runs.get_mut(&run_id) {
                    run.progress = ((i + 1) as f64 / iterations as f64) * 100.0;
                    run.results = results.clone();
                    run.best_result = best_result.clone();
                }
            }
            Err(err) => {
                let mut state = state.lock().unwrap();
                if let Some(run) = state.runs.get_mut(&run_id) {
                    run.status = "failed".to_string();
                    run.error = Some(err);
                    run.completed_at = Some(chrono::Utc::now().timestamp_millis());
                }
                return;
            }
        }
    }

    let mut state = state.lock().unwrap();
    if let Some(run) = state.runs.get_mut(&run_id) {
        run.status = "completed".to_string();
        run.completed_at = Some(chrono::Utc::now().timestamp_millis());
        run.progress = 100.0;
    }
}

#[tauri::command]
pub async fn optimizer_start(
    config: OptimizationConfig,
    state: tauri::State<'_, SharedOptimizerState>,
) -> Result<String, String> {
    let run_id = Uuid::new_v4().to_string();

    {
        let mut state = state.lock().map_err(|e| e.to_string())?;
        state.runs.insert(
            run_id.clone(),
            OptimizationRun {
                id: run_id.clone(),
                config: config.clone(),
                status: "running".to_string(),
                progress: 0.0,
                results: Vec::new(),
                best_result: None,
                started_at: chrono::Utc::now().timestamp_millis(),
                completed_at: None,
                error: None,
            },
        );
    }

    let state_clone = state.inner().clone();
    let config_clone = config.clone();
    let run_id_clone = run_id.clone();

    tauri::async_runtime::spawn(async move {
        match config_clone.method.as_str() {
            "genetic" => run_genetic_algorithm(config_clone, state_clone, run_id_clone).await,
            "random" | "monte_carlo" => {
                run_random_search(config_clone, state_clone, run_id_clone).await;
            }
            "grid" => {
                // For simplicity, grid search uses random search with more iterations
                run_random_search(config_clone, state_clone, run_id_clone).await;
            }
            _ => {
                run_random_search(config_clone, state_clone, run_id_clone).await;
            }
        }
    });

    Ok(run_id)
}

#[tauri::command]
pub async fn optimizer_cancel(
    id: String,
    state: tauri::State<'_, SharedOptimizerState>,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if let Some(run) = state.runs.get_mut(&id) {
        run.status = "cancelled".to_string();
        run.completed_at = Some(chrono::Utc::now().timestamp_millis());
    }
    Ok(())
}

#[tauri::command]
pub async fn optimizer_get_runs(
    state: tauri::State<'_, SharedOptimizerState>,
) -> Result<Vec<OptimizationRun>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.runs.values().cloned().collect())
}

#[tauri::command]
pub async fn optimizer_get_run(
    id: String,
    state: tauri::State<'_, SharedOptimizerState>,
) -> Result<Option<OptimizationRun>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.runs.get(&id).cloned())
}

// Simple random number generation utilities
mod rand {
    use std::cell::Cell;

    thread_local! {
        static RNG_STATE: Cell<u64> = Cell::new(987654321);
    }

    pub fn random<T>() -> T
    where
        T: RandGenerate,
    {
        RNG_STATE.with(|state| {
            let mut x = state.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            state.set(x);
            T::generate((x as f64) / (u64::MAX as f64))
        })
    }

    pub trait RandGenerate {
        fn generate(value: f64) -> Self;
    }

    impl RandGenerate for f64 {
        fn generate(value: f64) -> Self {
            value
        }
    }

    impl RandGenerate for bool {
        fn generate(value: f64) -> Self {
            value > 0.5
        }
    }

    impl RandGenerate for u32 {
        fn generate(value: f64) -> Self {
            (value * u32::MAX as f64) as u32
        }
    }

    impl RandGenerate for usize {
        fn generate(value: f64) -> Self {
            (value * usize::MAX as f64) as usize
        }
    }
}
