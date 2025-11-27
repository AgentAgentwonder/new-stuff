use super::types::*;
use std::collections::{HashMap, HashSet};

pub struct TransactionGraph {
    pub adjacency: HashMap<String, Vec<String>>,
    pub edges: HashMap<String, TokenFlowEdge>,
    pub nodes: HashMap<String, TokenFlowNode>,
}

impl TransactionGraph {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
            edges: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn from_transactions(transactions: Vec<TokenTransaction>) -> Self {
        let mut graph = Self::new();

        for tx in transactions {
            let edge = TokenFlowEdge {
                id: tx.transaction_hash.clone(),
                source: tx.source.clone(),
                target: tx.target.clone(),
                amount: tx.amount,
                timestamp: tx.timestamp,
                token_address: tx.token_address.clone(),
                transaction_hash: tx.transaction_hash.clone(),
            };

            graph.add_edge(edge);
        }

        graph.classify_nodes();
        graph
    }

    pub fn add_edge(&mut self, edge: TokenFlowEdge) {
        // Add to adjacency list
        self.adjacency
            .entry(edge.source.clone())
            .or_insert_with(Vec::new)
            .push(edge.target.clone());

        // Add nodes if they don't exist
        self.nodes
            .entry(edge.source.clone())
            .or_insert_with(|| TokenFlowNode {
                id: edge.source.clone(),
                address: edge.source.clone(),
                label: None,
                balance: 0.0,
                kind: NodeKind::Intermediate,
                cluster_id: None,
                risk: RiskLevel::Low,
            });

        self.nodes
            .entry(edge.target.clone())
            .or_insert_with(|| TokenFlowNode {
                id: edge.target.clone(),
                address: edge.target.clone(),
                label: None,
                balance: 0.0,
                kind: NodeKind::Intermediate,
                cluster_id: None,
                risk: RiskLevel::Low,
            });

        // Update balances
        if let Some(source_node) = self.nodes.get_mut(&edge.source) {
            source_node.balance -= edge.amount;
        }
        if let Some(target_node) = self.nodes.get_mut(&edge.target) {
            target_node.balance += edge.amount;
        }

        // Store edge
        self.edges.insert(edge.id.clone(), edge);
    }

    pub fn classify_nodes(&mut self) {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut out_degree: HashMap<String, usize> = HashMap::new();

        // Calculate degrees
        for (source, targets) in &self.adjacency {
            *out_degree.entry(source.clone()).or_insert(0) += targets.len();
            for target in targets {
                *in_degree.entry(target.clone()).or_insert(0) += 1;
            }
        }

        // Classify based on degrees
        for (id, node) in self.nodes.iter_mut() {
            let in_deg = in_degree.get(id).unwrap_or(&0);
            let out_deg = out_degree.get(id).unwrap_or(&0);

            node.kind = if *out_deg > 0 && *in_deg == 0 {
                NodeKind::Source
            } else if *in_deg > 0 && *out_deg == 0 {
                NodeKind::Destination
            } else {
                NodeKind::Intermediate
            };
        }
    }

    pub fn find_paths(&self, start: &str, end: &str, max_depth: usize) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        let mut current_path = vec![start.to_string()];
        let mut visited = HashSet::new();
        visited.insert(start.to_string());

        self.dfs_paths(
            start,
            end,
            max_depth,
            &mut current_path,
            &mut visited,
            &mut paths,
        );
        paths
    }

    fn dfs_paths(
        &self,
        current: &str,
        end: &str,
        depth_remaining: usize,
        current_path: &mut Vec<String>,
        visited: &mut HashSet<String>,
        paths: &mut Vec<Vec<String>>,
    ) {
        if current == end {
            paths.push(current_path.clone());
            return;
        }

        if depth_remaining == 0 {
            return;
        }

        if let Some(neighbors) = self.adjacency.get(current) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    visited.insert(neighbor.clone());
                    current_path.push(neighbor.clone());
                    self.dfs_paths(
                        neighbor,
                        end,
                        depth_remaining - 1,
                        current_path,
                        visited,
                        paths,
                    );
                    current_path.pop();
                    visited.remove(neighbor);
                }
            }
        }
    }

    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in self.nodes.keys() {
            if !visited.contains(node) {
                self.find_cycles_util(
                    node,
                    &mut visited,
                    &mut rec_stack,
                    &mut Vec::new(),
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn find_cycles_util(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = self.adjacency.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.find_cycles_util(neighbor, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle
                    if let Some(start_idx) = path.iter().position(|n| n == neighbor) {
                        let cycle = path[start_idx..].to_vec();
                        if cycle.len() > 1 {
                            cycles.push(cycle);
                        }
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    pub fn to_flow_graph(&self, token_address: &str) -> TokenFlowGraph {
        TokenFlowGraph {
            nodes: self.nodes.values().cloned().collect(),
            edges: self.edges.values().cloned().collect(),
            token_address: token_address.to_string(),
            time_range: self.calculate_time_range(),
        }
    }

    fn calculate_time_range(&self) -> TimeRange {
        let timestamps: Vec<i64> = self.edges.values().map(|e| e.timestamp).collect();
        let start = timestamps.iter().min().copied().unwrap_or(0);
        let end = timestamps.iter().max().copied().unwrap_or(0);
        TimeRange { start, end }
    }
}

pub fn generate_sankey_data(graph: &TokenFlowGraph) -> SankeyData {
    let mut nodes = Vec::new();
    let mut node_indices: HashMap<String, usize> = HashMap::new();

    // Create nodes
    for (idx, node) in graph.nodes.iter().enumerate() {
        nodes.push(SankeyNode {
            id: node.id.clone(),
            name: node
                .label
                .clone()
                .unwrap_or_else(|| truncate_address(&node.address)),
        });
        node_indices.insert(node.id.clone(), idx);
    }

    // Create links
    let mut links = Vec::new();
    for edge in &graph.edges {
        if let (Some(&source_idx), Some(&target_idx)) = (
            node_indices.get(&edge.source),
            node_indices.get(&edge.target),
        ) {
            links.push(SankeyLink {
                source: source_idx,
                target: target_idx,
                value: edge.amount,
            });
        }
    }

    SankeyData { nodes, links }
}

fn truncate_address(address: &str) -> String {
    if address.len() > 12 {
        format!("{}...{}", &address[..6], &address[address.len() - 4..])
    } else {
        address.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_from_transactions() {
        let transactions = vec![
            TokenTransaction {
                source: "A".to_string(),
                target: "B".to_string(),
                amount: 100.0,
                timestamp: 1000,
                token_address: "TOKEN1".to_string(),
                transaction_hash: "tx1".to_string(),
            },
            TokenTransaction {
                source: "B".to_string(),
                target: "C".to_string(),
                amount: 50.0,
                timestamp: 2000,
                token_address: "TOKEN1".to_string(),
                transaction_hash: "tx2".to_string(),
            },
        ];

        let graph = TransactionGraph::from_transactions(transactions);

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn test_node_classification() {
        let transactions = vec![TokenTransaction {
            source: "A".to_string(),
            target: "B".to_string(),
            amount: 100.0,
            timestamp: 1000,
            token_address: "TOKEN1".to_string(),
            transaction_hash: "tx1".to_string(),
        }];

        let graph = TransactionGraph::from_transactions(transactions);

        assert_eq!(graph.nodes.get("A").unwrap().kind, NodeKind::Source);
        assert_eq!(graph.nodes.get("B").unwrap().kind, NodeKind::Destination);
    }

    #[test]
    fn test_cycle_detection() {
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

        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
    }
}
