# ML Risk Scoring System - Phase 6 Task 6.1

## Overview

This system implements Machine Learning-based risk assessment for Solana tokens. It analyzes multiple features including holder concentration, liquidity, developer activity, sentiment, and token age to produce a comprehensive risk score from 0-100.

## Architecture

### Backend Components (Rust)

1. **Feature Engineering** (`src-tauri/src/ai.rs`)
   - Extracts 14 key features from token data
   - Normalizes features for model input
   - Combines data from holders, metadata, and verification sources

2. **Risk Model**
   - Logistic regression-style weighted scoring
   - Pre-trained weights based on common rug pull patterns
   - Serializable for persistence and updates

3. **Risk Analyzer**
   - Database-backed scoring history
   - Real-time inference
   - Model versioning and hot-swapping

4. **Storage**
   - SQLite database for risk scores and history
   - Model versioning table for retraining
   - Indexed queries for performance

### Frontend Components (TypeScript/React)

1. **RiskScoreBadge** - Visual risk level indicator
2. **RiskFactorsList** - Top contributing factors
3. **RiskHistoryChart** - Historical risk trend visualization
4. **RiskAnalysisPanel** - Comprehensive risk dashboard

## Feature Extraction

The model uses 14 features grouped by category:

### Holder Concentration
- **Gini Coefficient**: 0-1 scale, measures wealth distribution
- **Top 10 Percentage**: % of supply held by top 10 holders
- **Total Holders**: Inverse log-normalized holder count

### Liquidity
- **Liquidity USD**: Normalized liquidity amount
- **Liquidity to Market Cap Ratio**: Measure of exit liquidity

### Developer Signals
- **Has Mint Authority**: Boolean flag (high risk if true)
- **Has Freeze Authority**: Boolean flag (high risk if true)
- **Verified**: Token verification status
- **Audited**: Security audit status

### Community & Sentiment
- **Community Trust Score**: 0-1 scale from upvotes/downvotes
- **Sentiment Score**: -1 to 1 from sentiment analysis

### Age & Activity
- **Token Age Days**: Normalized age (< 30 days = higher risk)
- **Volume 24h**: Trading volume indicator
- **Price Volatility**: Price movement variance

## Model Training Workflow

### Current Model

The current model uses pre-defined weights based on observed rug pull patterns:

```rust
weights = {
  gini_coefficient: 30.0,
  top_10_percentage: 0.5,
  holder_count_inverse: 15.0,
  liquidity_score: -20.0,
  mint_authority: 25.0,
  freeze_authority: 20.0,
  verified: -15.0,
  audited: -20.0,
  community_trust: -10.0,
  sentiment: -5.0,
  age_score: -8.0,
  volatility: 12.0,
}
intercept = 50.0
```

### Retraining Procedure

When you have collected labeled data (legitimate tokens vs. rug pulls):

#### 1. Prepare Training Data

Create a CSV file with the following columns:
```
token_address,gini_coefficient,top_10_percentage,total_holders,liquidity_usd,has_mint_authority,has_freeze_authority,verified,audited,community_trust_score,sentiment_score,token_age_days,volume_24h,price_volatility,is_rug_pull
```

Example row:
```
ABC123...,0.85,75.0,120,5000.0,1,1,0,0,0.3,-0.2,3.0,1000.0,45.0,1
XYZ456...,0.35,28.0,8500,500000.0,0,0,1,1,0.9,0.6,180.0,250000.0,8.0,0
```

#### 2. Train Model (Python Script)

Create a training script using scikit-learn or LightGBM:

```python
import pandas as pd
from sklearn.linear_model import LogisticRegression
from sklearn.model_selection import train_test_split
from sklearn.metrics import roc_auc_score, precision_recall_curve
import json

# Load data
df = pd.read_csv('training_data.csv')

# Separate features and labels
X = df.drop(['token_address', 'is_rug_pull'], axis=1)
y = df['is_rug_pull']

# Train/test split
X_train, X_test, y_train, y_test = train_test_split(
    X, y, test_size=0.2, random_state=42, stratify=y
)

# Train model
model = LogisticRegression(penalty='l2', C=1.0)
model.fit(X_train, y_train)

# Evaluate
y_pred_proba = model.predict_proba(X_test)[:, 1]
auc_score = roc_auc_score(y_test, y_pred_proba)
print(f'AUC-ROC: {auc_score:.4f}')

# Calculate precision at various thresholds
precision, recall, thresholds = precision_recall_curve(y_test, y_pred_proba)
print(f'Precision at 90% recall: {precision[recall >= 0.9][0]:.4f}')

# Extract weights
weights = {
    feature: float(coef) 
    for feature, coef in zip(X.columns, model.coef_[0])
}
intercept = float(model.intercept_[0])

# Save model weights
model_data = {
    'weights': weights,
    'intercept': intercept,
    'threshold': 0.5
}

with open('model_weights.json', 'w') as f:
    json.dump(model_data, f, indent=2)

# Save metrics
metrics = {
    'auc_roc': auc_score,
    'training_samples': len(X_train),
    'test_samples': len(X_test),
    'feature_importance': {
        k: abs(v) for k, v in weights.items()
    }
}

with open('model_metrics.json', 'w') as f:
    json.dump(metrics, f, indent=2)
```

#### 3. Update Model in Application

After training, update the model in the Rust backend:

```rust
// Load new weights
let weights_json = std::fs::read_to_string("model_weights.json")?;
let model_data: serde_json::Value = serde_json::from_str(&weights_json)?;

// Parse weights
let mut new_weights = HashMap::new();
for (key, value) in model_data["weights"].as_object().unwrap() {
    new_weights.insert(key.clone(), value.as_f64().unwrap());
}

let intercept = model_data["intercept"].as_f64().unwrap();

// Create and save new model
let new_model = RiskModel::from_weights(new_weights, intercept);
let risk_analyzer = risk_analyzer_state.read().await;
*risk_analyzer.model.write().await = new_model;

// Persist to database
let metrics_json = std::fs::read_to_string("model_metrics.json").ok();
risk_analyzer.save_model(metrics_json).await?;
```

#### 4. Validate Model Performance

Add a validation command to test the new model:

```rust
#[tauri::command]
pub async fn validate_model(
    test_data: Vec<RiskFeatures>,
    labels: Vec<bool>,
    risk_analyzer: State<'_, SharedRiskAnalyzer>,
) -> Result<ModelMetrics, String> {
    let analyzer = risk_analyzer.read().await;
    let model = analyzer.model.read().await;
    
    let mut predictions = Vec::new();
    for features in &test_data {
        let (score, _) = model.score_token(features);
        predictions.push(score / 100.0); // Normalize to 0-1
    }
    
    // Calculate metrics
    let auc = calculate_auc(&labels, &predictions);
    let threshold = 0.6; // 60 out of 100
    let (precision, recall) = calculate_precision_recall(&labels, &predictions, threshold);
    
    Ok(ModelMetrics {
        auc,
        precision,
        recall,
        threshold,
    })
}
```

## Performance Metrics

The model should maintain:
- **AUC-ROC ≥ 0.85**: Overall discriminative ability
- **Precision ≥ 0.80 at 90% recall**: Minimize false positives
- **Inference time < 100ms**: Real-time scoring

## Monitoring & Alerts

### Automatic Retraining Triggers

1. **Model Drift**: When prediction accuracy drops below threshold
2. **New Patterns**: When new rug pull techniques are identified
3. **Scheduled**: Quarterly retraining with fresh labeled data

### Logging

All risk scores are logged with:
- Timestamp
- Token address
- Score and contributing factors
- Model version used

Query logs for analysis:
```sql
SELECT 
    DATE(timestamp) as date,
    AVG(score) as avg_score,
    COUNT(*) as tokens_scored
FROM risk_scores
WHERE timestamp >= datetime('now', '-30 days')
GROUP BY DATE(timestamp);
```

## Testing

### Unit Tests

Tests are included in `src-tauri/src/ai.rs`:

```bash
cargo test --package app_lib --lib ai::tests
```

Tests cover:
- Feature extraction correctness
- Model scoring logic
- High risk vs. low risk discrimination
- Model serialization/deserialization

### Integration Tests

Create integration tests to validate end-to-end flow:

```rust
#[tokio::test]
async fn test_risk_scoring_workflow() {
    // Initialize test database
    let analyzer = RiskAnalyzer::new(&test_app_handle()).await.unwrap();
    
    // Score a test token
    let features = RiskFeatures { /* ... */ };
    let score = analyzer.score_token("test_token", features).await.unwrap();
    
    // Verify score is in valid range
    assert!(score.score >= 0.0 && score.score <= 100.0);
    
    // Verify factors are returned
    assert!(!score.contributing_factors.is_empty());
    
    // Verify history tracking
    let history = analyzer.get_risk_history("test_token", 30).await.unwrap();
    assert_eq!(history.history.len(), 1);
}
```

## API Reference

### Tauri Commands

#### `get_token_risk_score`
Calculate risk score for a token.

**Input:**
```typescript
{ tokenAddress: string }
```

**Output:**
```typescript
{
  tokenAddress: string;
  score: number;
  riskLevel: "Low" | "Medium" | "High" | "Critical";
  contributingFactors: Array<{
    factorName: string;
    impact: number;
    severity: "Low" | "Medium" | "High";
    description: string;
  }>;
  timestamp: string;
}
```

#### `get_risk_history`
Get historical risk scores for a token.

**Input:**
```typescript
{ tokenAddress: string, days: number }
```

**Output:**
```typescript
{
  tokenAddress: string;
  history: Array<{
    timestamp: string;
    score: number;
    riskLevel: string;
  }>;
}
```

#### `get_latest_risk_score`
Get the most recent cached risk score (no recalculation).

**Input:**
```typescript
{ tokenAddress: string }
```

**Output:**
```typescript
RiskScore | null
```

## Future Enhancements

1. **Deep Learning Models**: Implement LSTM for temporal pattern detection
2. **Ensemble Methods**: Combine multiple models for improved accuracy
3. **Anomaly Detection**: Identify unusual trading patterns
4. **Real-time Updates**: Stream risk scores via WebSocket
5. **Explainable AI**: Enhanced SHAP-style feature importance
6. **Multi-chain Support**: Extend to Ethereum, BSC, etc.

## Troubleshooting

### Model not scoring correctly
- Check feature extraction in `get_token_risk_score`
- Verify data sources (holders, metadata, verification)
- Inspect feature values in logs

### Database errors
- Ensure app data directory is writable
- Check SQLite version compatibility
- Verify table schema with `sqlite3 risk_scores.db .schema`

### UI not updating
- Check browser console for errors
- Verify Tauri command is registered in lib.rs
- Test command directly with `invoke('get_token_risk_score', ...)`

## References

- [LightGBM Documentation](https://lightgbm.readthedocs.io/)
- [Scikit-learn Logistic Regression](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LogisticRegression.html)
- [ML Risk Assessment Best Practices](https://arxiv.org/abs/2103.00898)

## Support

For questions or issues:
1. Check logs in `~/.local/share/eclipse-market-pro/logs/`
2. Review test results: `cargo test --package app_lib --lib ai`
3. Validate feature extraction with debug logging
