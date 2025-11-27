use crate::token_flow::clustering::{
    assess_cluster_risk, build_wallet_clusters, detect_cluster_performance,
    perform_louvain_clustering, LouvainConfig,
};
use crate::token_flow::detection::{
    detect_circular_flows, detect_wash_trading, generate_alerts_from_patterns,
};
use crate::token_flow::graph::{generate_sankey_data, TransactionGraph};
use crate::token_flow::types::*;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowAnalysisRequest {
    pub token_address: String,
    pub transactions: Vec<TokenTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowAnalysisResponse {
    pub graph: TokenFlowGraph,
    pub clusters: Vec<WalletCluster>,
    pub alerts: Vec<TokenFlowAlert>,
    pub sankey: SankeyData,
    pub timeline: Vec<TimelineFrame>,
    pub wash_trading: Vec<WashTradingPattern>,
    pub circular_flows: Vec<CircularFlow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFlowRequest {
    pub analysis: FlowAnalysisResponse,
    pub format: FlowExportFormat,
    pub filters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFlowResponse {
    pub export: FlowExportData,
}

#[derive(Default)]
pub struct FlowAnalysisState {
    pub snapshots: Vec<FlowSnapshot>,
    pub subscriptions: Vec<ClusterSubscription>,
}

pub type SharedFlowAnalysisState = std::sync::Arc<RwLock<FlowAnalysisState>>;

pub fn create_token_flow_state() -> SharedFlowAnalysisState {
    std::sync::Arc::new(RwLock::new(FlowAnalysisState::default()))
}

#[tauri::command]
pub async fn analyze_token_flows(
    state: tauri::State<'_, SharedFlowAnalysisState>,
    request: FlowAnalysisRequest,
) -> Result<FlowAnalysisResponse, String> {
    let graph = TransactionGraph::from_transactions(request.transactions.clone());
    let flow_graph = graph.to_flow_graph(&request.token_address);

    let cluster_map = perform_louvain_clustering(&flow_graph.edges, LouvainConfig::default());
    let mut clusters = build_wallet_clusters(&flow_graph, &cluster_map);

    let circular_flows = detect_circular_flows(&graph);
    let wash_trading = detect_wash_trading(&flow_graph.edges);

    let alerts = generate_alerts_from_patterns(&circular_flows, &wash_trading);

    detect_cluster_performance(&mut clusters);
    assess_cluster_risk(&mut clusters, &alerts);

    let sankey = generate_sankey_data(&flow_graph);
    let timeline = build_timeline_frames(&flow_graph.edges);

    let response = FlowAnalysisResponse {
        graph: flow_graph.clone(),
        clusters: clusters.clone(),
        alerts: alerts.clone(),
        sankey,
        timeline,
        wash_trading,
        circular_flows,
    };

    persist_snapshot(state, &response).await;

    Ok(response)
}

#[tauri::command]
pub async fn export_flow_analysis(
    request: ExportFlowRequest,
) -> Result<ExportFlowResponse, String> {
    let ExportFlowRequest {
        analysis,
        format,
        filters,
    } = request;

    let metadata = FlowExportMetadata {
        exported_at: chrono::Utc::now().timestamp(),
        time_range: analysis.graph.time_range.clone(),
        filters,
    };

    let snapshot = if matches!(format, FlowExportFormat::Png | FlowExportFormat::Svg) {
        Some(generate_snapshot_placeholder(format.clone()))
    } else {
        None
    };

    let data = FlowExportContent {
        graph: Some(analysis.graph.clone()),
        clusters: Some(analysis.clusters.clone()),
        alerts: Some(analysis.alerts.clone()),
        snapshot,
    };

    Ok(ExportFlowResponse {
        export: FlowExportData {
            format,
            data,
            metadata,
        },
    })
}

fn build_timeline_frames(edges: &[TokenFlowEdge]) -> Vec<TimelineFrame> {
    let mut frames_map: HashMap<i64, Vec<&TokenFlowEdge>> = HashMap::new();

    for edge in edges {
        frames_map.entry(edge.timestamp).or_default().push(edge);
    }

    let mut timestamps: Vec<i64> = frames_map.keys().cloned().collect();
    timestamps.sort_unstable();

    timestamps
        .into_iter()
        .map(|timestamp| {
            let edges = frames_map.get(&timestamp).cloned().unwrap_or_default();
            let active_nodes: Vec<String> = edges
                .iter()
                .flat_map(|edge| vec![edge.source.clone(), edge.target.clone()])
                .collect();

            TimelineFrame {
                timestamp,
                flows: edges.into_iter().cloned().collect(),
                active_nodes,
            }
        })
        .collect()
}

async fn persist_snapshot(
    state: tauri::State<'_, SharedFlowAnalysisState>,
    response: &FlowAnalysisResponse,
) {
    let mut state = state.write().await;

    if state.snapshots.len() >= 100 {
        state.snapshots.remove(0);
    }

    state.snapshots.push(FlowSnapshot {
        id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        graph: response.graph.clone(),
        clusters: response.clusters.clone(),
        alerts: response.alerts.clone(),
    });
}

fn generate_snapshot_placeholder(format: FlowExportFormat) -> String {
    match format {
        FlowExportFormat::Png => general_purpose::STANDARD.encode("PNG_SNAPSHOT_PLACEHOLDER"),
        FlowExportFormat::Svg => general_purpose::STANDARD.encode("SVG_SNAPSHOT_PLACEHOLDER"),
        _ => String::new(),
    }
}

#[tauri::command]
pub async fn list_cluster_subscriptions(
    state: tauri::State<'_, SharedFlowAnalysisState>,
) -> Result<Vec<ClusterSubscription>, String> {
    let state = state.read().await;
    Ok(state.subscriptions.clone())
}

#[tauri::command]
pub async fn upsert_cluster_subscription(
    state: tauri::State<'_, SharedFlowAnalysisState>,
    subscription: ClusterSubscription,
) -> Result<(), String> {
    let mut state = state.write().await;
    if let Some(existing) = state
        .subscriptions
        .iter_mut()
        .find(|sub| sub.id == subscription.id)
    {
        *existing = subscription;
    } else {
        state.subscriptions.push(subscription);
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_cluster_subscription(
    state: tauri::State<'_, SharedFlowAnalysisState>,
    subscription_id: String,
) -> Result<(), String> {
    let mut state = state.write().await;
    state.subscriptions.retain(|sub| sub.id != subscription_id);
    Ok(())
}
