# Launch Predictor AI Guide

## Overview

The Launch Predictor AI module provides forward-looking assessments for newly issued tokens. It assembles developer history, contract architecture, liquidity depth, and marketing telemetry into a structured feature vector, trains an interpretable logistic model, persists the derived signals, and exposes Tauri commands for real-time inference.

The pipeline combines three pillars:

1. **Feature Ingestion** – canonicalises raw launch metrics (developer reputation, proxy patterns, liquidity thermals, social engagement) and stores the serialised feature payload so future retraining runs can reuse the curated data.
2. **Model Lifecycle** – trains and evaluates an interpretable logistic model, tracks metrics for each version, and persists the active set of weights to SQLite. Retraining is fully async-safe and swaps the active checkpoint atomically.
3. **Inference & UX** – exposes commands for prediction, history, bias inspection, and feature extraction. The React panel surfaces the success probability, feature contributions, early-warning signals, and integrates with the watchlist to maintain operational monitoring.

## Feature Schema

The pipeline consumes the `TokenFeatures` schema located at `src-tauri/src/ai/launch_predictor/features.rs`. Important contributors include:

- **Developer Track Record**: reputation score, historical launch count, success rate, and category (experienced / studio / DAO / unproven). These drive the baseline success prior.
- **Contract Architecture**: complexity score, proxy / upgradeability flags, audit coverage. High complexity with proxy control increases risk weighting.
- **Liquidity Backing**: absolute USD depth, market-cap ratio, 24h liquidity delta, DEX order book depth. These inform exit risk and slippage potential.
- **Marketing Telemetry**: hype index, paid spend, social follower velocity, community engagement, influencer sentiment. The model detects hyped-but-hollow launches via cross-features.
- **Retention & Demand Signals**: watchlist conversions, holder retention, launch age. These stabilise the probability as the launch matures.

All boolean/flagged fields are normalised to `[0,1]` and monetary fields are treated with log transforms. The helper `TokenFeatures::feature_vector()` applies the canonical preprocessing.

## Training Data Requirements

To retrain the model credibly:

- **Sample Balance**: target a minimum of 500 labelled launches with at least 30% positive (successful) outcomes. This keeps the logistic surface stable and avoids collapse to the majority class.
- **Outcome Labelling**: supply a float between `0` and `1` representing realised launch success (e.g. sustained liquidity, price appreciation, retained holders). Binary `0/1` works, but fractional scores allow nuanced calibration.
- **Temporal Coverage**: include at least 90 days of historical launches; mixing bull and bear market samples prevents overfitting to short-term sentiment regimes.
- **Segment Diversity**: ensure adequate coverage across `developer_category` segments. The bias audit expects at least five samples per segment to perform disparity checks.
- **Data Hygiene**: deduplicate tokens by mint/address. When ingesting features, prefer the earliest snapshot (<12h from launch) to stabilise training iterations.

Use the `add_launch_training_data` command or direct SQLite inserts into the `training_data` table to append labelled samples.

## Retraining Workflow

1. **Prime Training Cache**: run `extract_token_features` for recent launches and store the outcome label once known via `add_launch_training_data`.
2. **Trigger Retraining**: call `retrain_launch_model`. The service fetches all `training_data`, performs gradient descent, records metrics (accuracy, precision, recall, F1), and persists the fresh weights plus metadata into `launch_models` with a bumped version.
3. **Activate Model**: the retraining method automatically deactivates the previous checkpoint and marks the new version as active. All future `predict_launch_success` invocations read from the in-memory copy.
4. **Bias Audit**: after retraining, call `get_launch_bias_report` to review success rate deltas per developer segment. Flagged segments (< -15% delta with >=5 samples) are returned for remediation.
5. **Client Refresh**: optionally invoke `load_latest_launch_model` on app start to hydrate the latest persisted weights into memory. The app `setup` already performs this automatically.

## Inference Commands

| Command | Description |
| --- | --- |
| `extract_token_features` | Returns a baseline feature set (mocked example for now) that can be enriched by upstream services. |
| `predict_launch_success` | Accepts `{ tokenAddress, features }` and returns success probability, confidence, feature contributions, and early warnings. Stores both the prediction and features in SQLite. |
| `get_launch_prediction_history` | Fetches historical predictions for the token, enabling charting and drift analysis. |
| `add_launch_training_data` | Persists a labelled launch outcome for future retrains. |
| `retrain_launch_model` | Runs gradient descent retraining and stores a new active model version. |
| `load_latest_launch_model` | Reloads the active model from SQLite into memory (useful after manual migrations). |
| `get_launch_bias_report` | Generates a discrepancy report across developer segments to detect adverse impact. |

## UI Integration

The `LaunchPredictorPanel` component (under `src/components/launchPredictor`) drives the UX:

- **ScoreCard** displays probability, confidence, peak timeframe, and key catalysts.
- **FeatureImportance** animates weight contributions to make the model explainable.
- **EarlyWarnings** lists triggered heuristics (low liquidity, proxy controls, hype mismatches).
- **WatchlistIntegration** attaches the token to an existing watchlist for ongoing monitoring.

Testing coverage (`src/__tests__/launchPredictor.test.tsx`) ensures the panel renders, handles errors, surfaces warnings, and enforces the loading state.

## Bias Monitoring

Bias reports rely on the `developer_category` field. Maintain balanced training data across:

- `experienced`
- `studio`
- `dao`
- `unproven`

The bias audit flags segments with ≥5 samples and >15% lower success rate than the global baseline. Use this to detect structural disadvantages, then introduce corrective weighting or targeted data augmentation.

## Maintenance Checklist

- Schedule weekly retraining via a task runner once fresh outcomes accumulate.
- Archive previous model versions if metrics regress.
- Review early warnings trending in the UI to adjust detection thresholds.
- Extend `extract_token_features` once upstream services deliver real on-chain analytics.
- Augment tests when adding new features or commands to maintain deterministic behaviour.
