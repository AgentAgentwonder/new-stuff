use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditResult {
    pub contract_address: String,
    pub security_score: u8,
    pub risk_level: RiskLevel,
    pub findings: Vec<Finding>,
    pub audit_sources: Vec<AuditSource>,
    pub metadata: AuditMetadata,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=39 => RiskLevel::Critical,
            40..=59 => RiskLevel::High,
            60..=79 => RiskLevel::Medium,
            80..=100 => RiskLevel::Low,
            _ => RiskLevel::Critical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub severity: Severity,
    pub category: String,
    pub title: String,
    pub description: String,
    pub recommendation: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSource {
    pub name: String,
    pub status: AuditStatus,
    pub score: Option<u8>,
    pub last_updated: Option<DateTime<Utc>>,
    pub report_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditStatus {
    Verified,
    Pending,
    Failed,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditMetadata {
    pub is_mintable: bool,
    pub has_freeze_authority: bool,
    pub is_mutable: bool,
    pub has_blacklist: bool,
    pub is_honeypot: bool,
    pub creator_address: Option<String>,
    pub total_supply: Option<String>,
    pub holder_count: Option<u64>,
}

pub struct AuditCache {
    cache: Mutex<HashMap<String, AuditResult>>,
    max_age_seconds: i64,
}

impl AuditCache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            max_age_seconds: 3600, // 1 hour
        }
    }

    pub fn get(&self, address: &str) -> Option<AuditResult> {
        let cache = self.cache.lock().ok()?;
        let result = cache.get(address)?;

        let age = Utc::now().signed_duration_since(result.timestamp);
        if age.num_seconds() > self.max_age_seconds {
            return None;
        }

        Some(result.clone())
    }

    pub fn set(&self, address: String, result: AuditResult) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(address, result);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
}

impl Default for AuditCache {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HeuristicScanner;

impl HeuristicScanner {
    pub fn scan_token_program(program_code: Option<&str>) -> Vec<Finding> {
        let mut findings = Vec::new();

        if let Some(code) = program_code {
            // Check for dangerous functions
            if code.contains("selfdestruct") || code.contains("suicide") {
                findings.push(Finding {
                    severity: Severity::Critical,
                    category: "Dangerous Function".to_string(),
                    title: "Self-destruct function detected".to_string(),
                    description:
                        "Contract contains selfdestruct function which can destroy the contract"
                            .to_string(),
                    recommendation: Some(
                        "Avoid interacting with contracts that can be destroyed".to_string(),
                    ),
                    source: "Heuristic".to_string(),
                });
            }

            if code.contains("delegatecall") {
                findings.push(Finding {
                    severity: Severity::High,
                    category: "Dangerous Function".to_string(),
                    title: "Delegatecall detected".to_string(),
                    description:
                        "Contract uses delegatecall which can be risky if not properly implemented"
                            .to_string(),
                    recommendation: Some(
                        "Review the delegatecall implementation carefully".to_string(),
                    ),
                    source: "Heuristic".to_string(),
                });
            }

            // Check for unchecked external calls
            if code.contains(".call(") && !code.contains("require(") {
                findings.push(Finding {
                    severity: Severity::Medium,
                    category: "Unsafe Pattern".to_string(),
                    title: "Unchecked external call".to_string(),
                    description: "External calls without proper error handling detected"
                        .to_string(),
                    recommendation: Some(
                        "Always check return values of external calls".to_string(),
                    ),
                    source: "Heuristic".to_string(),
                });
            }
        }

        findings
    }

    pub fn analyze_metadata(metadata: &AuditMetadata) -> Vec<Finding> {
        let mut findings = Vec::new();

        if metadata.is_mintable {
            findings.push(Finding {
                severity: Severity::Medium,
                category: "Token Economics".to_string(),
                title: "Mintable token".to_string(),
                description: "Token has mint authority which can create new tokens".to_string(),
                recommendation: Some(
                    "Be aware that supply can increase, potentially diluting value".to_string(),
                ),
                source: "Heuristic".to_string(),
            });
        }

        if metadata.has_freeze_authority {
            findings.push(Finding {
                severity: Severity::High,
                category: "Centralization".to_string(),
                title: "Freeze authority present".to_string(),
                description: "Token has freeze authority which can prevent transfers".to_string(),
                recommendation: Some("Consider the risks of frozen tokens".to_string()),
                source: "Heuristic".to_string(),
            });
        }

        if metadata.has_blacklist {
            findings.push(Finding {
                severity: Severity::Medium,
                category: "Centralization".to_string(),
                title: "Blacklist mechanism detected".to_string(),
                description: "Token can blacklist addresses, preventing them from transferring"
                    .to_string(),
                recommendation: Some("Your address could be blacklisted at any time".to_string()),
                source: "Heuristic".to_string(),
            });
        }

        if metadata.is_honeypot {
            findings.push(Finding {
                severity: Severity::Critical,
                category: "Scam".to_string(),
                title: "Potential honeypot".to_string(),
                description: "Token shows characteristics of a honeypot scam".to_string(),
                recommendation: Some("DO NOT INTERACT - High risk of losing funds".to_string()),
                source: "Heuristic".to_string(),
            });
        }

        if let Some(holders) = metadata.holder_count {
            if holders < 10 {
                findings.push(Finding {
                    severity: Severity::High,
                    category: "Liquidity".to_string(),
                    title: "Very low holder count".to_string(),
                    description: format!("Only {} holders detected", holders),
                    recommendation: Some("Low distribution increases rug pull risk".to_string()),
                    source: "Heuristic".to_string(),
                });
            }
        }

        findings
    }

    pub fn calculate_score(findings: &[Finding]) -> u8 {
        let mut score = 100u8;

        for finding in findings {
            let deduction = match finding.severity {
                Severity::Critical => 30,
                Severity::High => 15,
                Severity::Medium => 8,
                Severity::Low => 3,
                Severity::Info => 0,
            };
            score = score.saturating_sub(deduction);
        }

        score
    }
}

pub struct CertikClient;

impl CertikClient {
    pub async fn fetch_audit(contract_address: &str) -> Result<AuditSource, String> {
        // Mock implementation - in production, would call Certik API
        // For now, simulate with random data based on address

        let score = Self::mock_score(contract_address);

        Ok(AuditSource {
            name: "CertiK".to_string(),
            status: if score > 0 {
                AuditStatus::Verified
            } else {
                AuditStatus::Unavailable
            },
            score: if score > 0 { Some(score) } else { None },
            last_updated: if score > 0 { Some(Utc::now()) } else { None },
            report_url: if score > 0 {
                Some(format!(
                    "https://skynet.certik.com/projects/{}",
                    contract_address
                ))
            } else {
                None
            },
        })
    }

    fn mock_score(address: &str) -> u8 {
        // Generate deterministic score from address for testing
        let hash: u8 = address.bytes().fold(0u8, |acc, b| acc.wrapping_add(b));
        if hash % 3 == 0 {
            0 // No audit available
        } else {
            60 + (hash % 30) // Score between 60-89
        }
    }
}

pub struct TrailOfBitsClient;

impl TrailOfBitsClient {
    pub async fn fetch_audit(contract_address: &str) -> Result<AuditSource, String> {
        // Mock implementation - in production, would call Trail of Bits API

        let score = Self::mock_score(contract_address);

        Ok(AuditSource {
            name: "Trail of Bits".to_string(),
            status: if score > 0 {
                AuditStatus::Verified
            } else {
                AuditStatus::Unavailable
            },
            score: if score > 0 { Some(score) } else { None },
            last_updated: if score > 0 { Some(Utc::now()) } else { None },
            report_url: if score > 0 {
                Some(format!(
                    "https://www.trailofbits.com/audits/{}",
                    contract_address
                ))
            } else {
                None
            },
        })
    }

    fn mock_score(address: &str) -> u8 {
        let hash: u8 = address.bytes().fold(0u8, |acc, b| acc.wrapping_add(b));
        if hash % 4 == 0 {
            0
        } else {
            65 + (hash % 25)
        }
    }
}

pub async fn perform_audit(
    contract_address: &str,
    metadata: AuditMetadata,
) -> Result<AuditResult, String> {
    let mut findings = Vec::new();
    let mut audit_sources = Vec::new();

    // Fetch external audits
    if let Ok(certik) = CertikClient::fetch_audit(contract_address).await {
        audit_sources.push(certik);
    }

    if let Ok(tob) = TrailOfBitsClient::fetch_audit(contract_address).await {
        audit_sources.push(tob);
    }

    // Run heuristic scans
    let metadata_findings = HeuristicScanner::analyze_metadata(&metadata);
    findings.extend(metadata_findings);

    // If we have contract code, scan it (for now we skip this)
    // In a real implementation, you would fetch the contract bytecode/source
    // let code_findings = HeuristicScanner::scan_token_program(Some(contract_code));
    // findings.extend(code_findings);

    // Calculate aggregate score
    let heuristic_score = HeuristicScanner::calculate_score(&findings);
    let external_scores: Vec<u8> = audit_sources.iter().filter_map(|s| s.score).collect();

    let security_score = if external_scores.is_empty() {
        heuristic_score
    } else {
        let external_avg = external_scores.iter().sum::<u8>() / external_scores.len() as u8;
        ((heuristic_score as u16 + external_avg as u16) / 2) as u8
    };

    let risk_level = RiskLevel::from_score(security_score);

    Ok(AuditResult {
        contract_address: contract_address.to_string(),
        security_score,
        risk_level,
        findings,
        audit_sources,
        metadata,
        timestamp: Utc::now(),
    })
}

// Tauri Commands

#[tauri::command]
pub async fn scan_contract(
    contract_address: String,
    app: AppHandle,
) -> Result<AuditResult, String> {
    let cache: tauri::State<AuditCache> = app.state();

    // Check cache first
    if let Some(cached) = cache.get(&contract_address) {
        return Ok(cached);
    }

    // Fetch token metadata (mock for now)
    let metadata = fetch_token_metadata(&contract_address).await?;

    // Perform audit
    let result = perform_audit(&contract_address, metadata).await?;

    // Cache result
    cache.set(contract_address, result.clone());

    Ok(result)
}

#[tauri::command]
pub async fn get_cached_audit(
    contract_address: String,
    app: AppHandle,
) -> Result<Option<AuditResult>, String> {
    let cache: tauri::State<AuditCache> = app.state();
    Ok(cache.get(&contract_address))
}

#[tauri::command]
pub async fn clear_audit_cache(app: AppHandle) -> Result<(), String> {
    let cache: tauri::State<AuditCache> = app.state();
    cache.clear();
    Ok(())
}

#[tauri::command]
pub fn check_risk_threshold(
    security_score: u8,
    user_threshold: Option<u8>,
) -> Result<bool, String> {
    let threshold = user_threshold.unwrap_or(60);
    Ok(security_score < threshold)
}

async fn fetch_token_metadata(contract_address: &str) -> Result<AuditMetadata, String> {
    // Mock implementation - in production, fetch from Solana RPC
    // Simulating different characteristics based on address

    let hash = contract_address
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_add(b as u64));

    Ok(AuditMetadata {
        is_mintable: hash % 3 == 0,
        has_freeze_authority: hash % 5 == 0,
        is_mutable: hash % 4 == 0,
        has_blacklist: hash % 7 == 0,
        is_honeypot: hash % 13 == 0,
        creator_address: Some(format!("Creator{}", hash % 1000)),
        total_supply: Some(format!("{}", 1_000_000_000 + (hash % 1_000_000_000))),
        holder_count: Some(((hash % 10000) + 100)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(RiskLevel::from_score(90), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(70), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(50), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(30), RiskLevel::Critical);
    }

    #[test]
    fn test_score_calculation() {
        let findings = vec![
            Finding {
                severity: Severity::Critical,
                category: "Test".to_string(),
                title: "Critical issue".to_string(),
                description: "Test".to_string(),
                recommendation: None,
                source: "Test".to_string(),
            },
            Finding {
                severity: Severity::High,
                category: "Test".to_string(),
                title: "High issue".to_string(),
                description: "Test".to_string(),
                recommendation: None,
                source: "Test".to_string(),
            },
        ];

        let score = HeuristicScanner::calculate_score(&findings);
        assert_eq!(score, 55); // 100 - 30 - 15
    }

    #[test]
    fn test_honeypot_detection() {
        let metadata = AuditMetadata {
            is_mintable: false,
            has_freeze_authority: false,
            is_mutable: false,
            has_blacklist: false,
            is_honeypot: true,
            creator_address: None,
            total_supply: None,
            holder_count: None,
        };

        let findings = HeuristicScanner::analyze_metadata(&metadata);
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
    }

    #[test]
    fn test_mintable_detection() {
        let metadata = AuditMetadata {
            is_mintable: true,
            has_freeze_authority: false,
            is_mutable: false,
            has_blacklist: false,
            is_honeypot: false,
            creator_address: None,
            total_supply: None,
            holder_count: None,
        };

        let findings = HeuristicScanner::analyze_metadata(&metadata);
        assert!(findings.iter().any(|f| f.title.contains("Mintable")));
    }

    #[test]
    fn test_low_holder_count() {
        let metadata = AuditMetadata {
            is_mintable: false,
            has_freeze_authority: false,
            is_mutable: false,
            has_blacklist: false,
            is_honeypot: false,
            creator_address: None,
            total_supply: None,
            holder_count: Some(5),
        };

        let findings = HeuristicScanner::analyze_metadata(&metadata);
        assert!(findings.iter().any(|f| f.title.contains("holder count")));
    }

    #[tokio::test]
    async fn test_audit_flow() {
        let metadata = AuditMetadata {
            is_mintable: true,
            has_freeze_authority: false,
            is_mutable: false,
            has_blacklist: false,
            is_honeypot: false,
            creator_address: Some("test".to_string()),
            total_supply: Some("1000000".to_string()),
            holder_count: Some(100),
        };

        let result = perform_audit("test_address", metadata).await.unwrap();
        assert!(!result.findings.is_empty());
        assert!(result.security_score <= 100);
    }
}
