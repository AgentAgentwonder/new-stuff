#!/usr/bin/env python3
"""
ML Risk Scoring Model Training Script
Phase 6 Task 6.1

This script trains a logistic regression model to predict rug pull risk
based on token features.

Requirements:
    pip install pandas scikit-learn numpy

Usage:
    python train_risk_model.py --data path/to/training_data.csv --output ./model_output/
"""

import argparse
import json
import sys
from pathlib import Path

try:
    import pandas as pd
    import numpy as np
    from sklearn.linear_model import LogisticRegression
    from sklearn.model_selection import train_test_split, cross_val_score
    from sklearn.metrics import (
        roc_auc_score,
        precision_recall_curve,
        classification_report,
        confusion_matrix,
    )
except ImportError:
    print("Error: Required packages not installed.")
    print("Install with: pip install pandas scikit-learn numpy")
    sys.exit(1)


def load_data(filepath: str) -> tuple[pd.DataFrame, pd.Series]:
    """Load and prepare training data."""
    print(f"Loading data from {filepath}...")
    df = pd.read_csv(filepath)
    
    # Verify required columns
    required_cols = [
        'gini_coefficient', 'top_10_percentage', 'total_holders',
        'liquidity_usd', 'has_mint_authority', 'has_freeze_authority',
        'verified', 'audited', 'community_trust_score', 'sentiment_score',
        'token_age_days', 'volume_24h', 'price_volatility', 'is_rug_pull'
    ]
    
    missing_cols = set(required_cols) - set(df.columns)
    if missing_cols:
        raise ValueError(f"Missing required columns: {missing_cols}")
    
    # Separate features and labels
    X = df[required_cols[:-1]]  # All except is_rug_pull
    y = df['is_rug_pull']
    
    print(f"Loaded {len(df)} samples ({y.sum()} rug pulls, {len(y) - y.sum()} legitimate)")
    return X, y


def train_model(X: pd.DataFrame, y: pd.Series, test_size: float = 0.2) -> dict:
    """Train logistic regression model and return results."""
    print("\nTraining model...")
    
    # Train/test split
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=test_size, random_state=42, stratify=y
    )
    
    # Train model
    model = LogisticRegression(
        penalty='l2',
        C=1.0,
        solver='lbfgs',
        max_iter=1000,
        random_state=42
    )
    model.fit(X_train, y_train)
    
    # Predictions
    y_pred = model.predict(X_test)
    y_pred_proba = model.predict_proba(X_test)[:, 1]
    
    # Evaluate
    auc_score = roc_auc_score(y_test, y_pred_proba)
    
    # Cross-validation
    cv_scores = cross_val_score(model, X_train, y_train, cv=5, scoring='roc_auc')
    
    # Precision-recall at various thresholds
    precision, recall, thresholds = precision_recall_curve(y_test, y_pred_proba)
    
    # Find precision at 90% recall
    idx_90_recall = np.argmax(recall >= 0.9)
    precision_at_90_recall = precision[idx_90_recall] if idx_90_recall < len(precision) else precision[-1]
    
    print(f"\nModel Performance:")
    print(f"  AUC-ROC: {auc_score:.4f}")
    print(f"  Cross-validation AUC: {cv_scores.mean():.4f} (+/- {cv_scores.std() * 2:.4f})")
    print(f"  Precision at 90% recall: {precision_at_90_recall:.4f}")
    print(f"\nClassification Report:")
    print(classification_report(y_test, y_pred, target_names=['Legitimate', 'Rug Pull']))
    print(f"\nConfusion Matrix:")
    print(confusion_matrix(y_test, y_pred))
    
    # Feature importance
    feature_importance = dict(zip(X.columns, np.abs(model.coef_[0])))
    sorted_features = sorted(feature_importance.items(), key=lambda x: x[1], reverse=True)
    print(f"\nTop 5 Most Important Features:")
    for feature, importance in sorted_features[:5]:
        print(f"  {feature}: {importance:.4f}")
    
    return {
        'model': model,
        'auc_roc': auc_score,
        'cv_scores': cv_scores,
        'precision_at_90_recall': precision_at_90_recall,
        'X_test': X_test,
        'y_test': y_test,
        'y_pred_proba': y_pred_proba,
        'feature_importance': feature_importance,
    }


def export_model(results: dict, X: pd.DataFrame, output_dir: Path):
    """Export model weights and metrics for Rust integration."""
    output_dir.mkdir(parents=True, exist_ok=True)
    
    model = results['model']
    
    # Convert weights to Rust format
    weights = {
        feature: float(coef)
        for feature, coef in zip(X.columns, model.coef_[0])
    }
    intercept = float(model.intercept_[0])
    
    # Scale to 0-100 risk score (multiply weights by 100 for scaling)
    scaled_weights = {k: v * 100.0 for k, v in weights.items()}
    scaled_intercept = intercept * 100.0
    
    model_data = {
        'weights': scaled_weights,
        'intercept': scaled_intercept,
        'threshold': 0.5
    }
    
    # Save model
    model_path = output_dir / 'model_weights.json'
    with open(model_path, 'w') as f:
        json.dump(model_data, f, indent=2)
    print(f"\nModel weights saved to {model_path}")
    
    # Save metrics
    metrics = {
        'auc_roc': results['auc_roc'],
        'cv_mean': float(results['cv_scores'].mean()),
        'cv_std': float(results['cv_scores'].std()),
        'precision_at_90_recall': results['precision_at_90_recall'],
        'training_date': pd.Timestamp.now().isoformat(),
        'feature_importance': {k: float(v) for k, v in results['feature_importance'].items()},
    }
    
    metrics_path = output_dir / 'model_metrics.json'
    with open(metrics_path, 'w') as f:
        json.dump(metrics, f, indent=2)
    print(f"Model metrics saved to {metrics_path}")


def main():
    parser = argparse.ArgumentParser(description='Train ML risk scoring model')
    parser.add_argument('--data', type=str, required=True, help='Path to training CSV file')
    parser.add_argument('--output', type=str, default='./model_output/', help='Output directory')
    parser.add_argument('--test-size', type=float, default=0.2, help='Test set size (0-1)')
    
    args = parser.parse_args()
    
    try:
        # Load data
        X, y = load_data(args.data)
        
        # Train model
        results = train_model(X, y, test_size=args.test_size)
        
        # Check if model meets baseline requirements
        if results['auc_roc'] < 0.75:
            print("\n⚠️  WARNING: Model AUC-ROC is below recommended threshold of 0.75")
            print("Consider collecting more training data or adjusting features.")
        elif results['auc_roc'] >= 0.85:
            print("\n✅ Model meets performance requirements (AUC ≥ 0.85)")
        
        # Export
        export_model(results, X, Path(args.output))
        
        print("\n✅ Training complete!")
        print(f"\nTo integrate this model into the application:")
        print(f"1. Copy {args.output}/model_weights.json to your application")
        print(f"2. Update the RiskModel::new() weights in src-tauri/src/ai.rs")
        print(f"3. Or use the hot-reload functionality to update at runtime")
        
    except Exception as e:
        print(f"\n❌ Error: {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()
