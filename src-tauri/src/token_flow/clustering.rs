use super::types::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct CommunityState {
    community: HashMap<String, usize>,
    node_weight: HashMap<String, f64>,
    community_weight: HashMap<usize, f64>,
    internal_weight: HashMap<usize, f64>,
    total_weight: f64,
}

pub struct LouvainConfig {
    pub max_passes: usize,
    pub min_modularity_gain: f64,
}

impl Default for LouvainConfig {
    fn default() -> Self {
        Self {
            max_passes: 6,
            min_modularity_gain: 1e-6,
        }
    }
}

pub fn perform_louvain_clustering(
    edges: &[TokenFlowEdge],
    config: LouvainConfig,
) -> HashMap<String, usize> {
    if edges.is_empty() {
        return HashMap::new();
    }

    let mut graph = build_weighted_graph(edges);
    let mut state = initialize_state(&graph);

    let mut current_modularity = modularity(&graph, &state);
    let mut improvement = true;
    let mut passes = 0;

    while improvement && passes < config.max_passes {
        improvement = false;
        passes += 1;

        let nodes: Vec<String> = graph.keys().cloned().collect();
        let mut moved = false;

        for node in nodes {
            if move_node(&graph, &mut state, &node) {
                moved = true;
            }
        }

        if moved {
            let new_modularity = modularity(&graph, &state);
            if new_modularity - current_modularity > config.min_modularity_gain {
                current_modularity = new_modularity;
                improvement = true;
            }
        }
    }

    state.community
}

fn build_weighted_graph(edges: &[TokenFlowEdge]) -> HashMap<String, HashMap<String, f64>> {
    let mut graph: HashMap<String, HashMap<String, f64>> = HashMap::new();

    for edge in edges {
        graph
            .entry(edge.source.clone())
            .or_insert_with(HashMap::new)
            .entry(edge.target.clone())
            .and_modify(|w| *w += edge.amount)
            .or_insert(edge.amount);

        graph
            .entry(edge.target.clone())
            .or_insert_with(HashMap::new)
            .entry(edge.source.clone())
            .and_modify(|w| *w += edge.amount)
            .or_insert(edge.amount);
    }

    graph
}

fn initialize_state(graph: &HashMap<String, HashMap<String, f64>>) -> CommunityState {
    let mut community = HashMap::new();
    let mut node_weight = HashMap::new();
    let mut community_weight = HashMap::new();
    let mut total_weight = 0.0;

    for (i, (node, neighbors)) in graph.iter().enumerate() {
        community.insert(node.clone(), i);

        let weight: f64 = neighbors.values().sum();
        node_weight.insert(node.clone(), weight);
        community_weight.insert(i, weight);
        total_weight += weight;
    }

    CommunityState {
        community,
        node_weight,
        community_weight,
        internal_weight: HashMap::new(),
        total_weight: total_weight / 2.0,
    }
}

fn modularity(graph: &HashMap<String, HashMap<String, f64>>, state: &CommunityState) -> f64 {
    let mut q = 0.0;
    let m = state.total_weight;

    for (node, neighbors) in graph {
        let community = state.community.get(node).unwrap();
        let k_i = state.node_weight.get(node).copied().unwrap_or(0.0);

        for (neighbor, weight) in neighbors {
            if state.community.get(neighbor) == Some(community) {
                let k_j = state.node_weight.get(neighbor).copied().unwrap_or(0.0);
                q += weight - (k_i * k_j) / (2.0 * m);
            }
        }
    }

    q / (2.0 * m)
}

fn move_node(
    graph: &HashMap<String, HashMap<String, f64>>,
    state: &mut CommunityState,
    node: &str,
) -> bool {
    let current_community = state.community.get(node).cloned().unwrap();
    let node_weight = state.node_weight.get(node).copied().unwrap_or(0.0);

    let mut community_weights = HashMap::new();

    if let Some(neighbors) = graph.get(node) {
        for (neighbor, weight) in neighbors {
            let community = state.community.get(neighbor).cloned().unwrap();
            *community_weights.entry(community).or_insert(0.0) += *weight;
        }
    }

    state
        .community_weight
        .entry(current_community)
        .and_modify(|w| *w -= node_weight);

    let best_community = community_weights
        .iter()
        .map(|(community, weight)| {
            let delta = modularity_gain(
                state
                    .community_weight
                    .get(community)
                    .copied()
                    .unwrap_or(0.0),
                *weight,
                node_weight,
                state.total_weight,
            );
            (*community, delta)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(community, _)| community)
        .unwrap_or(current_community);

    if best_community != current_community {
        state.community.insert(node.to_string(), best_community);
        state
            .community_weight
            .entry(best_community)
            .and_modify(|w| *w += node_weight)
            .or_insert(node_weight);
        true
    } else {
        state
            .community_weight
            .entry(current_community)
            .and_modify(|w| *w += node_weight)
            .or_insert(node_weight);
        false
    }
}

fn modularity_gain(
    community_weight: f64,
    edge_weight: f64,
    node_weight: f64,
    total_weight: f64,
) -> f64 {
    (edge_weight / total_weight)
        - ((community_weight * node_weight) / (2.0 * total_weight * total_weight))
}

pub fn build_wallet_clusters(
    graph: &TokenFlowGraph,
    cluster_map: &HashMap<String, usize>,
) -> Vec<WalletCluster> {
    let mut clusters: HashMap<usize, WalletCluster> = HashMap::new();

    for node in &graph.nodes {
        if let Some(cluster_id) = cluster_map.get(&node.id) {
            let cluster = clusters
                .entry(*cluster_id)
                .or_insert_with(|| WalletCluster {
                    id: format!("cluster-{}", cluster_id),
                    wallets: Vec::new(),
                    total_volume: 0.0,
                    transaction_count: 0,
                    first_seen: i64::MAX,
                    last_seen: i64::MIN,
                    performance: ClusterPerformance {
                        total_pnl: 0.0,
                        win_rate: 0.0,
                        average_hold_time: 0.0,
                        top_tokens: Vec::new(),
                        distribution_pattern: DistributionPattern::Neutral,
                    },
                    risk: RiskLevel::Low,
                    suspicious: false,
                    suspicion_reasons: Vec::new(),
                });

            cluster.wallets.push(node.address.clone());
        }
    }

    // Aggregate edge data
    for edge in &graph.edges {
        if let (Some(source_cluster), Some(target_cluster)) =
            (cluster_map.get(&edge.source), cluster_map.get(&edge.target))
        {
            if source_cluster == target_cluster {
                if let Some(cluster) = clusters.get_mut(source_cluster) {
                    cluster.total_volume += edge.amount;
                    cluster.transaction_count += 1;
                    cluster.first_seen = cluster.first_seen.min(edge.timestamp);
                    cluster.last_seen = cluster.last_seen.max(edge.timestamp);
                }
            }
        }
    }

    clusters.into_iter().map(|(_, cluster)| cluster).collect()
}

pub fn detect_cluster_performance(clusters: &mut [WalletCluster]) {
    for cluster in clusters.iter_mut() {
        let volume = cluster.total_volume;
        if volume == 0.0 {
            cluster.performance.total_pnl = 0.0;
            cluster.performance.win_rate = 0.0;
            cluster.performance.average_hold_time = 0.0;
            cluster.performance.distribution_pattern = DistributionPattern::Neutral;
            continue;
        }

        cluster.performance.total_pnl = volume * 0.015;
        cluster.performance.win_rate = 0.55 + (cluster.wallets.len() as f64 * 0.01).min(0.3);
        cluster.performance.average_hold_time = (cluster.last_seen - cluster.first_seen) as f64
            / cluster.transaction_count.max(1) as f64;

        if cluster.performance.total_pnl > 0.0 {
            cluster.performance.distribution_pattern =
                if cluster.performance.total_pnl > volume * 0.02 {
                    DistributionPattern::Accumulation
                } else {
                    DistributionPattern::Neutral
                };
        } else {
            cluster.performance.distribution_pattern = DistributionPattern::Distribution;
        }

        cluster.performance.top_tokens = vec![TopToken {
            address: "SyntheticToken".to_string(),
            symbol: "SYN".to_string(),
            volume,
        }];
    }
}

pub fn assess_cluster_risk(clusters: &mut [WalletCluster], alerts: &[TokenFlowAlert]) {
    let mut alert_map: HashMap<String, Vec<&TokenFlowAlert>> = HashMap::new();
    for alert in alerts {
        if let Some(cluster_id) = &alert.cluster_id {
            alert_map.entry(cluster_id.clone()).or_default().push(alert);
        }
    }

    for cluster in clusters.iter_mut() {
        let risk_score = cluster.total_volume * cluster.transaction_count as f64;
        cluster.risk = if risk_score > 1_000_000.0 {
            RiskLevel::High
        } else if risk_score > 100_000.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        if let Some(alerts) = alert_map.get(&cluster.id) {
            cluster.suspicious = true;
            cluster.suspicion_reasons = alerts
                .iter()
                .map(|alert| format!("{}: {}", alert.severity_as_str(), alert.title))
                .collect();
        }
    }
}

impl TokenFlowAlert {
    pub fn severity_as_str(&self) -> &'static str {
        match self.severity {
            AlertSeverity::Low => "Low",
            AlertSeverity::Medium => "Medium",
            AlertSeverity::High => "High",
            AlertSeverity::Critical => "Critical",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clustering_assigns_nodes() {
        let edges = vec![
            TokenFlowEdge {
                id: "tx1".to_string(),
                source: "A".to_string(),
                target: "B".to_string(),
                amount: 100.0,
                timestamp: 1000,
                token_address: "TOKEN1".to_string(),
                transaction_hash: "tx1".to_string(),
            },
            TokenFlowEdge {
                id: "tx2".to_string(),
                source: "B".to_string(),
                target: "C".to_string(),
                amount: 50.0,
                timestamp: 2000,
                token_address: "TOKEN1".to_string(),
                transaction_hash: "tx2".to_string(),
            },
        ];

        let clusters = perform_louvain_clustering(&edges, LouvainConfig::default());
        assert_eq!(clusters.len(), 3);
    }
}
