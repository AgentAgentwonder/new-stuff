use super::graph::TransactionGraph;
use super::types::*;
use std::collections::HashMap;
use uuid::Uuid;

const CIRCULAR_FLOW_THRESHOLD: f64 = 0.8;
const WASH_TRADING_MIN_CYCLES: usize = 3;
const PING_PONG_TIME_WINDOW: i64 = 3600; // 1 hour in seconds

pub fn detect_circular_flows(graph: &TransactionGraph) -> Vec<CircularFlow> {
    let cycles = graph.detect_cycles();
    let mut circular_flows = Vec::new();

    for cycle in cycles {
        if cycle.len() < 2 {
            continue;
        }

        let mut total_amount = 0.0;
        let mut cycle_count = 0;
        let mut token_address = String::new();

        // Calculate amount and count cycles
        for i in 0..cycle.len() {
            let current = &cycle[i];
            let next = &cycle[(i + 1) % cycle.len()];

            for edge in graph.edges.values() {
                if edge.source == *current && edge.target == *next {
                    total_amount += edge.amount;
                    cycle_count += 1;
                    if token_address.is_empty() {
                        token_address = edge.token_address.clone();
                    }
                    break;
                }
            }
        }

        let confidence = calculate_circular_confidence(&cycle, graph);

        if confidence >= CIRCULAR_FLOW_THRESHOLD {
            circular_flows.push(CircularFlow {
                id: Uuid::new_v4().to_string(),
                wallets: cycle,
                amount: total_amount,
                token_address,
                cycles: cycle_count,
                confidence,
                detected_at: chrono::Utc::now().timestamp(),
            });
        }
    }

    circular_flows
}

fn calculate_circular_confidence(cycle: &[String], graph: &TransactionGraph) -> f64 {
    if cycle.is_empty() {
        return 0.0;
    }

    let mut matching_edges = 0;
    let mut total_checks = 0;

    for i in 0..cycle.len() {
        let current = &cycle[i];
        let next = &cycle[(i + 1) % cycle.len()];

        total_checks += 1;

        for edge in graph.edges.values() {
            if edge.source == *current && edge.target == *next {
                matching_edges += 1;
                break;
            }
        }
    }

    if total_checks == 0 {
        0.0
    } else {
        matching_edges as f64 / total_checks as f64
    }
}

pub fn detect_wash_trading(edges: &[TokenFlowEdge]) -> Vec<WashTradingPattern> {
    let mut patterns = Vec::new();

    // Detect ping-pong pattern
    patterns.extend(detect_ping_pong_pattern(edges));

    // Detect circular pattern (would be similar to circular flows but with specific characteristics)
    patterns.extend(detect_circular_wash_trading(edges));

    // Detect layered pattern
    patterns.extend(detect_layered_pattern(edges));

    patterns
}

fn detect_ping_pong_pattern(edges: &[TokenFlowEdge]) -> Vec<WashTradingPattern> {
    let mut patterns = Vec::new();
    let mut wallet_pairs: HashMap<(String, String), Vec<&TokenFlowEdge>> = HashMap::new();

    // Group edges by wallet pairs
    for edge in edges {
        let pair = if edge.source < edge.target {
            (edge.source.clone(), edge.target.clone())
        } else {
            (edge.target.clone(), edge.source.clone())
        };
        wallet_pairs.entry(pair).or_insert_with(Vec::new).push(edge);
    }

    // Check for ping-pong pattern
    for ((wallet1, wallet2), pair_edges) in wallet_pairs {
        if pair_edges.len() < WASH_TRADING_MIN_CYCLES * 2 {
            continue;
        }

        let mut forward = 0;
        let mut backward = 0;
        let mut total_volume = 0.0;

        for edge in &pair_edges {
            if edge.source == wallet1 {
                forward += 1;
            } else {
                backward += 1;
            }
            total_volume += edge.amount;
        }

        // Ping-pong should have roughly equal forward and backward transactions
        let balance_ratio = (forward as f64 / (forward + backward) as f64 - 0.5).abs();
        let confidence = 1.0 - (balance_ratio * 2.0);

        if confidence > 0.7
            && forward >= WASH_TRADING_MIN_CYCLES
            && backward >= WASH_TRADING_MIN_CYCLES
        {
            // Check if transactions happen within time windows
            let time_windows_count = check_time_windows(&pair_edges);

            if time_windows_count >= WASH_TRADING_MIN_CYCLES {
                patterns.push(WashTradingPattern {
                    id: Uuid::new_v4().to_string(),
                    wallets: vec![wallet1, wallet2],
                    token_address: pair_edges[0].token_address.clone(),
                    volume: total_volume,
                    transaction_count: pair_edges.len(),
                    confidence,
                    detected_at: chrono::Utc::now().timestamp(),
                    pattern: WashTradingPatternKind::PingPong,
                });
            }
        }
    }

    patterns
}

fn check_time_windows(edges: &[&TokenFlowEdge]) -> usize {
    let mut sorted_edges: Vec<_> = edges.iter().collect();
    sorted_edges.sort_by_key(|e| e.timestamp);

    let mut windows = 0;
    let mut i = 0;

    while i < sorted_edges.len() {
        let window_start = sorted_edges[i].timestamp;
        let mut count = 1;
        let mut j = i + 1;

        while j < sorted_edges.len()
            && sorted_edges[j].timestamp - window_start <= PING_PONG_TIME_WINDOW
        {
            count += 1;
            j += 1;
        }

        if count >= 2 {
            windows += 1;
        }

        i = j.max(i + 1);
    }

    windows
}

fn detect_circular_wash_trading(edges: &[TokenFlowEdge]) -> Vec<WashTradingPattern> {
    let mut patterns = Vec::new();
    let graph = TransactionGraph::from_transactions(
        edges
            .iter()
            .map(|e| TokenTransaction {
                source: e.source.clone(),
                target: e.target.clone(),
                amount: e.amount,
                timestamp: e.timestamp,
                token_address: e.token_address.clone(),
                transaction_hash: e.transaction_hash.clone(),
            })
            .collect(),
    );

    let cycles = graph.detect_cycles();

    for cycle in cycles {
        if cycle.len() < 3 {
            continue;
        }

        let mut total_volume = 0.0;
        let mut transaction_count = 0;
        let mut token_address = String::new();

        for i in 0..cycle.len() {
            let current = &cycle[i];
            let next = &cycle[(i + 1) % cycle.len()];

            for edge in edges {
                if edge.source == *current && edge.target == *next {
                    total_volume += edge.amount;
                    transaction_count += 1;
                    if token_address.is_empty() {
                        token_address = edge.token_address.clone();
                    }
                }
            }
        }

        if transaction_count >= WASH_TRADING_MIN_CYCLES {
            let confidence = calculate_circular_confidence(&cycle, &graph);

            patterns.push(WashTradingPattern {
                id: Uuid::new_v4().to_string(),
                wallets: cycle,
                token_address,
                volume: total_volume,
                transaction_count,
                confidence,
                detected_at: chrono::Utc::now().timestamp(),
                pattern: WashTradingPatternKind::Circular,
            });
        }
    }

    patterns
}

fn detect_layered_pattern(edges: &[TokenFlowEdge]) -> Vec<WashTradingPattern> {
    let mut patterns = Vec::new();
    let graph = TransactionGraph::from_transactions(
        edges
            .iter()
            .map(|e| TokenTransaction {
                source: e.source.clone(),
                target: e.target.clone(),
                amount: e.amount,
                timestamp: e.timestamp,
                token_address: e.token_address.clone(),
                transaction_hash: e.transaction_hash.clone(),
            })
            .collect(),
    );

    // Find paths with multiple intermediaries
    for node1 in graph.nodes.keys() {
        for node2 in graph.nodes.keys() {
            if node1 == node2 {
                continue;
            }

            let paths = graph.find_paths(node1, node2, 5);

            for path in paths {
                if path.len() >= 4 {
                    let mut total_volume = 0.0;
                    let mut transaction_count = 0;
                    let mut token_address = String::new();

                    for i in 0..path.len() - 1 {
                        for edge in edges {
                            if edge.source == path[i] && edge.target == path[i + 1] {
                                total_volume += edge.amount;
                                transaction_count += 1;
                                if token_address.is_empty() {
                                    token_address = edge.token_address.clone();
                                }
                            }
                        }
                    }

                    let confidence = if transaction_count > 0 {
                        (1.0 / path.len() as f64).min(0.9)
                    } else {
                        0.0
                    };

                    if confidence > 0.5 && transaction_count >= WASH_TRADING_MIN_CYCLES {
                        patterns.push(WashTradingPattern {
                            id: Uuid::new_v4().to_string(),
                            wallets: path,
                            token_address,
                            volume: total_volume,
                            transaction_count,
                            confidence,
                            detected_at: chrono::Utc::now().timestamp(),
                            pattern: WashTradingPatternKind::Layered,
                        });
                    }
                }
            }
        }
    }

    patterns
}

pub fn generate_alerts_from_patterns(
    circular_flows: &[CircularFlow],
    wash_trading: &[WashTradingPattern],
) -> Vec<TokenFlowAlert> {
    let mut alerts = Vec::new();

    // Alerts for circular flows
    for flow in circular_flows {
        let severity = if flow.confidence > 0.95 {
            AlertSeverity::Critical
        } else if flow.confidence > 0.85 {
            AlertSeverity::High
        } else {
            AlertSeverity::Medium
        };

        let mut metadata = HashMap::new();
        metadata.insert("confidence".to_string(), serde_json::json!(flow.confidence));
        metadata.insert("cycles".to_string(), serde_json::json!(flow.cycles));
        metadata.insert("amount".to_string(), serde_json::json!(flow.amount));

        alerts.push(TokenFlowAlert {
            id: Uuid::new_v4().to_string(),
            alert_type: AlertType::CircularFlow,
            severity,
            title: "Circular Flow Detected".to_string(),
            description: format!(
                "Detected circular flow involving {} wallets with {} cycles and {:.2} confidence",
                flow.wallets.len(),
                flow.cycles,
                flow.confidence
            ),
            cluster_id: None,
            wallets: flow.wallets.clone(),
            token_address: Some(flow.token_address.clone()),
            metadata,
            timestamp: flow.detected_at,
            acknowledged: false,
        });
    }

    // Alerts for wash trading
    for pattern in wash_trading {
        let severity = if pattern.confidence > 0.9 {
            AlertSeverity::Critical
        } else if pattern.confidence > 0.8 {
            AlertSeverity::High
        } else {
            AlertSeverity::Medium
        };

        let pattern_name = match pattern.pattern {
            WashTradingPatternKind::PingPong => "Ping-Pong",
            WashTradingPatternKind::Circular => "Circular",
            WashTradingPatternKind::Layered => "Layered",
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            "confidence".to_string(),
            serde_json::json!(pattern.confidence),
        );
        metadata.insert("pattern".to_string(), serde_json::json!(pattern_name));
        metadata.insert("volume".to_string(), serde_json::json!(pattern.volume));

        alerts.push(TokenFlowAlert {
            id: Uuid::new_v4().to_string(),
            alert_type: AlertType::WashTrading,
            severity,
            title: format!("{} Wash Trading Pattern Detected", pattern_name),
            description: format!(
                "Detected {} wash trading pattern involving {} wallets with volume {} and {:.2} confidence",
                pattern_name,
                pattern.wallets.len(),
                pattern.volume,
                pattern.confidence
            ),
            cluster_id: None,
            wallets: pattern.wallets.clone(),
            token_address: Some(pattern.token_address.clone()),
            metadata,
            timestamp: pattern.detected_at,
            acknowledged: false,
        });
    }

    alerts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_pong_detection() {
        let mut edges = Vec::new();
        let timestamp_base = 1000;

        for i in 0..6 {
            edges.push(TokenFlowEdge {
                id: format!("tx{}", i),
                source: if i % 2 == 0 {
                    "A".to_string()
                } else {
                    "B".to_string()
                },
                target: if i % 2 == 0 {
                    "B".to_string()
                } else {
                    "A".to_string()
                },
                amount: 100.0,
                timestamp: timestamp_base + (i as i64 * 300),
                token_address: "TOKEN1".to_string(),
                transaction_hash: format!("tx{}", i),
            });
        }

        let patterns = detect_ping_pong_pattern(&edges);
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_circular_flow_detection() {
        let mut graph = TransactionGraph::new();

        graph.add_edge(TokenFlowEdge {
            id: "tx1".to_string(),
            source: "A".to_string(),
            target: "B".to_string(),
            amount: 100.0,
            timestamp: 1000,
            token_address: "TOKEN1".to_string(),
            transaction_hash: "tx1".to_string(),
        });

        graph.add_edge(TokenFlowEdge {
            id: "tx2".to_string(),
            source: "B".to_string(),
            target: "C".to_string(),
            amount: 100.0,
            timestamp: 2000,
            token_address: "TOKEN1".to_string(),
            transaction_hash: "tx2".to_string(),
        });

        graph.add_edge(TokenFlowEdge {
            id: "tx3".to_string(),
            source: "C".to_string(),
            target: "A".to_string(),
            amount: 100.0,
            timestamp: 3000,
            token_address: "TOKEN1".to_string(),
            transaction_hash: "tx3".to_string(),
        });

        let flows = detect_circular_flows(&graph);
        assert!(!flows.is_empty());
    }
}
