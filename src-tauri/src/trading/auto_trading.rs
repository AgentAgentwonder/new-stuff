use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use uuid::Uuid;

const DEFAULT_STARTING_CAPITAL: f64 = 100_000.0;

fn is_config_empty(config: &HashMap<String, Value>) -> bool {
    config.is_empty()
}

fn is_param_empty(params: &HashMap<String, f64>) -> bool {
    params.is_empty()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSource {
    pub source_type: String,
    pub id: String,
    pub weight: f64,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "is_config_empty")]
    pub config: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskControls {
    pub max_position_size: f64,
    pub max_daily_loss: f64,
    pub max_drawdown: f64,
    pub max_open_positions: u32,
    pub stop_loss_percent: f64,
    pub take_profit_percent: f64,
    pub trailing_stop_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSizingConfig {
    pub method: String,
    pub fixed_percent: Option<f64>,
    pub kelly_fraction: Option<f64>,
    pub target_volatility: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStrategyInput {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub signal_sources: Vec<SignalSource>,
    pub combination_logic: String,
    pub weight_threshold: Option<f64>,
    pub position_sizing: PositionSizingConfig,
    pub risk_controls: RiskControls,
    pub allowed_symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStrategy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub signal_sources: Vec<SignalSource>,
    pub combination_logic: String,
    pub weight_threshold: Option<f64>,
    pub position_sizing: PositionSizingConfig,
    pub risk_controls: RiskControls,
    pub allowed_symbols: Vec<String>,
    #[serde(default, skip_serializing_if = "is_param_empty")]
    pub optimized_parameters: HashMap<String, f64>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TradingStrategyUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub signal_sources: Option<Vec<SignalSource>>,
    pub combination_logic: Option<String>,
    pub weight_threshold: Option<f64>,
    pub position_sizing: Option<PositionSizingConfig>,
    pub risk_controls: Option<RiskControls>,
    pub allowed_symbols: Option<Vec<String>>,
    pub optimized_parameters: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyExecution {
    pub id: String,
    pub strategy_id: String,
    pub strategy_name: String,
    pub status: ExecutionStatus,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub started_at: DateTime<Utc>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "chrono::serde::ts_milliseconds_option"
    )]
    pub stopped_at: Option<DateTime<Utc>>,
    pub trades_executed: u32,
    pub total_pnl: f64,
    pub total_pnl_percent: f64,
    pub win_rate: f64,
    pub current_drawdown: f64,
    pub daily_pnl: f64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Running,
    Paused,
    Stopped,
    Error,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub trailing_stop: Option<f64>,
    pub opened_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct AutoTradingEngine {
    strategies: HashMap<String, TradingStrategy>,
    executions: HashMap<String, StrategyExecution>,
    positions: HashMap<String, Vec<Position>>,
    kill_switch_active: bool,
    starting_capital: f64,
    current_capital: f64,
}

impl AutoTradingEngine {
    pub fn new(starting_capital: f64) -> Self {
        Self {
            strategies: HashMap::new(),
            executions: HashMap::new(),
            positions: HashMap::new(),
            kill_switch_active: false,
            starting_capital,
            current_capital: starting_capital,
        }
    }

    pub fn add_strategy(&mut self, input: TradingStrategyInput) -> TradingStrategy {
        let now = Utc::now();
        let strategy = TradingStrategy {
            id: Uuid::new_v4().to_string(),
            name: input.name,
            description: input.description,
            enabled: input.enabled,
            signal_sources: input.signal_sources,
            combination_logic: input.combination_logic,
            weight_threshold: input.weight_threshold,
            position_sizing: input.position_sizing,
            risk_controls: input.risk_controls,
            allowed_symbols: input.allowed_symbols,
            optimized_parameters: HashMap::new(),
            created_at: now,
            updated_at: now,
        };

        self.strategies
            .insert(strategy.id.clone(), strategy.clone());
        strategy
    }

    pub fn update_strategy(
        &mut self,
        id: &str,
        updates: TradingStrategyUpdate,
    ) -> Result<TradingStrategy, String> {
        let strategy = self
            .strategies
            .get_mut(id)
            .ok_or_else(|| format!("Strategy {} not found", id))?;

        if let Some(name) = updates.name {
            strategy.name = name;
        }
        if let Some(description) = updates.description {
            strategy.description = description;
        }
        if let Some(enabled) = updates.enabled {
            strategy.enabled = enabled;
        }
        if let Some(signal_sources) = updates.signal_sources {
            strategy.signal_sources = signal_sources;
        }
        if let Some(combination_logic) = updates.combination_logic {
            strategy.combination_logic = combination_logic;
        }
        if updates.weight_threshold.is_some() {
            strategy.weight_threshold = updates.weight_threshold;
        }
        if let Some(position_sizing) = updates.position_sizing {
            strategy.position_sizing = position_sizing;
        }
        if let Some(risk_controls) = updates.risk_controls {
            strategy.risk_controls = risk_controls;
        }
        if let Some(allowed_symbols) = updates.allowed_symbols {
            strategy.allowed_symbols = allowed_symbols;
        }
        if let Some(params) = updates.optimized_parameters {
            strategy.optimized_parameters = params;
        }

        strategy.updated_at = Utc::now();
        Ok(strategy.clone())
    }

    pub fn delete_strategy(&mut self, id: &str) -> Result<(), String> {
        if let Some(exec) = self.executions.get(id) {
            if exec.status == ExecutionStatus::Running {
                return Err("Cannot delete a running strategy".into());
            }
        }
        self.strategies.remove(id);
        self.executions.remove(id);
        self.positions.remove(id);
        Ok(())
    }

    pub fn start_strategy(&mut self, strategy_id: &str) -> Result<StrategyExecution, String> {
        if self.kill_switch_active {
            return Err("Kill switch is active".to_string());
        }

        let strategy = self
            .strategies
            .get(strategy_id)
            .ok_or_else(|| format!("Strategy {} not found", strategy_id))?;

        if !strategy.enabled {
            return Err("Strategy is disabled".into());
        }

        let execution = StrategyExecution {
            id: Uuid::new_v4().to_string(),
            strategy_id: strategy_id.to_string(),
            strategy_name: strategy.name.clone(),
            status: ExecutionStatus::Running,
            started_at: Utc::now(),
            stopped_at: None,
            trades_executed: 0,
            total_pnl: 0.0,
            total_pnl_percent: 0.0,
            win_rate: 0.0,
            current_drawdown: 0.0,
            daily_pnl: 0.0,
            last_error: None,
        };

        self.executions
            .insert(strategy_id.to_string(), execution.clone());
        self.positions.insert(strategy_id.to_string(), Vec::new());

        Ok(execution)
    }

    pub fn stop_strategy(&mut self, strategy_id: &str) -> Result<(), String> {
        if let Some(execution) = self.executions.get_mut(strategy_id) {
            execution.status = ExecutionStatus::Stopped;
            execution.stopped_at = Some(Utc::now());
            Ok(())
        } else {
            Err(format!("No execution found for strategy {}", strategy_id))
        }
    }

    pub fn pause_strategy(&mut self, strategy_id: &str) -> Result<(), String> {
        if let Some(execution) = self.executions.get_mut(strategy_id) {
            execution.status = ExecutionStatus::Paused;
            Ok(())
        } else {
            Err(format!("No execution found for strategy {}", strategy_id))
        }
    }

    pub fn activate_kill_switch(&mut self) {
        self.kill_switch_active = true;
        for execution in self.executions.values_mut() {
            if execution.status == ExecutionStatus::Running {
                execution.status = ExecutionStatus::Stopped;
                execution.stopped_at = Some(Utc::now());
            }
        }
    }

    pub fn deactivate_kill_switch(&mut self) {
        self.kill_switch_active = false;
    }

    pub fn is_kill_switch_active(&self) -> bool {
        self.kill_switch_active
    }

    pub fn evaluate_signals(&self, strategy_id: &str, signals: &HashMap<String, f64>) -> bool {
        let strategy = match self.strategies.get(strategy_id) {
            Some(strategy) => strategy,
            None => return false,
        };

        let enabled_sources: Vec<&SignalSource> = strategy
            .signal_sources
            .iter()
            .filter(|source| source.enabled)
            .collect();

        if enabled_sources.is_empty() {
            return false;
        }

        match strategy.combination_logic.as_str() {
            "all" => enabled_sources
                .iter()
                .all(|source| signals.get(&source.id).copied().unwrap_or(0.0) > 0.0),
            "any" => enabled_sources
                .iter()
                .any(|source| signals.get(&source.id).copied().unwrap_or(0.0) > 0.0),
            "majority" => {
                let positive_count = enabled_sources
                    .iter()
                    .filter(|source| signals.get(&source.id).copied().unwrap_or(0.0) > 0.0)
                    .count();
                positive_count > enabled_sources.len() / 2
            }
            "weighted" => {
                let weighted_sum: f64 = enabled_sources
                    .iter()
                    .map(|source| {
                        let signal_value = signals.get(&source.id).copied().unwrap_or(0.0);
                        signal_value * source.weight
                    })
                    .sum();
                let threshold = strategy.weight_threshold.unwrap_or(0.5);
                weighted_sum >= threshold
            }
            _ => false,
        }
    }

    pub fn calculate_position_size(
        &self,
        strategy_id: &str,
        current_price: f64,
    ) -> Result<f64, String> {
        let strategy = self
            .strategies
            .get(strategy_id)
            .ok_or_else(|| format!("Strategy {} not found", strategy_id))?;

        let available_capital = self.current_capital;

        let size = match strategy.position_sizing.method.as_str() {
            "fixed" => {
                let percent = strategy.position_sizing.fixed_percent.unwrap_or(10.0);
                (available_capital * percent / 100.0) / current_price
            }
            "kelly" => {
                let fraction = strategy.position_sizing.kelly_fraction.unwrap_or(0.25);
                let position_value = available_capital * fraction;
                position_value / current_price
            }
            "risk_parity" => {
                let position_value =
                    available_capital * strategy.risk_controls.max_position_size / 100.0;
                position_value / current_price
            }
            "volatility_based" => {
                let target_vol = strategy.position_sizing.target_volatility.unwrap_or(2.0);
                let assumed_vol = 3.0;
                let scaling_factor = target_vol / assumed_vol;
                let position_value = available_capital * strategy.risk_controls.max_position_size
                    / 100.0
                    * scaling_factor;
                position_value / current_price
            }
            _ => (available_capital * 10.0 / 100.0) / current_price,
        };

        Ok(size.max(0.0))
    }

    pub fn check_risk_controls(
        &self,
        strategy_id: &str,
        proposed_trade_value: f64,
    ) -> Result<(), String> {
        let strategy = self
            .strategies
            .get(strategy_id)
            .ok_or_else(|| format!("Strategy {} not found", strategy_id))?;

        let execution = self
            .executions
            .get(strategy_id)
            .ok_or_else(|| format!("No execution found for strategy {}", strategy_id))?;

        let position_percent = if self.current_capital > 0.0 {
            (proposed_trade_value / self.current_capital) * 100.0
        } else {
            100.0
        };

        if position_percent > strategy.risk_controls.max_position_size {
            return Err(format!(
                "Position size {:.2}% exceeds max {:.2}%",
                position_percent, strategy.risk_controls.max_position_size
            ));
        }

        if execution.daily_pnl < 0.0 {
            let daily_loss_percent = if self.starting_capital > 0.0 {
                (execution.daily_pnl.abs() / self.starting_capital) * 100.0
            } else {
                0.0
            };
            if daily_loss_percent >= strategy.risk_controls.max_daily_loss {
                return Err(format!(
                    "Daily loss {:.2}% exceeds max {:.2}%",
                    daily_loss_percent, strategy.risk_controls.max_daily_loss
                ));
            }
        }

        if execution.current_drawdown >= strategy.risk_controls.max_drawdown {
            return Err(format!(
                "Drawdown {:.2}% exceeds max {:.2}%",
                execution.current_drawdown, strategy.risk_controls.max_drawdown
            ));
        }

        let open_positions = self
            .positions
            .get(strategy_id)
            .map(|p| p.len())
            .unwrap_or(0);
        if open_positions as u32 >= strategy.risk_controls.max_open_positions {
            return Err(format!(
                "Open positions {} at max {}",
                open_positions, strategy.risk_controls.max_open_positions
            ));
        }

        Ok(())
    }

    pub fn apply_parameters(
        &mut self,
        strategy_id: &str,
        parameters: HashMap<String, f64>,
    ) -> Result<TradingStrategy, String> {
        let strategy = self
            .strategies
            .get_mut(strategy_id)
            .ok_or_else(|| format!("Strategy {} not found", strategy_id))?;

        strategy.optimized_parameters = parameters;
        strategy.updated_at = Utc::now();
        Ok(strategy.clone())
    }

    pub fn get_strategies(&self) -> Vec<TradingStrategy> {
        self.strategies.values().cloned().collect()
    }

    pub fn get_strategy(&self, id: &str) -> Option<TradingStrategy> {
        self.strategies.get(id).cloned()
    }

    pub fn get_execution(&self, strategy_id: &str) -> Option<StrategyExecution> {
        self.executions.get(strategy_id).cloned()
    }

    pub fn get_all_executions(&self) -> Vec<StrategyExecution> {
        self.executions.values().cloned().collect()
    }
}

pub type SharedAutoTradingEngine = Arc<Mutex<AutoTradingEngine>>;

pub fn register_auto_trading_state(app: &tauri::App) {
    let engine: SharedAutoTradingEngine =
        Arc::new(Mutex::new(AutoTradingEngine::new(DEFAULT_STARTING_CAPITAL)));
    app.manage(engine);
}

#[tauri::command]
pub async fn auto_trading_create_strategy(
    strategy: TradingStrategyInput,
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<TradingStrategy, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.add_strategy(strategy))
}

#[tauri::command]
pub async fn auto_trading_update_strategy(
    id: String,
    updates: TradingStrategyUpdate,
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<TradingStrategy, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.update_strategy(&id, updates)
}

#[tauri::command]
pub async fn auto_trading_delete_strategy(
    id: String,
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<(), String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.delete_strategy(&id)
}

#[tauri::command]
pub async fn auto_trading_start_strategy(
    strategy_id: String,
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<StrategyExecution, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.start_strategy(&strategy_id)
}

#[tauri::command]
pub async fn auto_trading_stop_strategy(
    strategy_id: String,
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<(), String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.stop_strategy(&strategy_id)
}

#[tauri::command]
pub async fn auto_trading_pause_strategy(
    strategy_id: String,
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<(), String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.pause_strategy(&strategy_id)
}

#[tauri::command]
pub async fn auto_trading_activate_kill_switch(
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<(), String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.activate_kill_switch();
    Ok(())
}

#[tauri::command]
pub async fn auto_trading_deactivate_kill_switch(
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<(), String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.deactivate_kill_switch();
    Ok(())
}

#[tauri::command]
pub async fn auto_trading_get_strategies(
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<Vec<TradingStrategy>, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_strategies())
}

#[tauri::command]
pub async fn auto_trading_get_strategy(
    id: String,
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<Option<TradingStrategy>, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_strategy(&id))
}

#[tauri::command]
pub async fn auto_trading_get_executions(
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<Vec<StrategyExecution>, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_all_executions())
}

#[tauri::command]
pub async fn auto_trading_apply_parameters(
    strategy_id: String,
    parameters: HashMap<String, f64>,
    engine: tauri::State<'_, SharedAutoTradingEngine>,
) -> Result<TradingStrategy, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.apply_parameters(&strategy_id, parameters)
}
