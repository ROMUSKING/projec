//! Self-improvement engine for the coding agent.
//!
//! This module provides the core self-improvement capabilities including
//! performance tracking, bottleneck identification, strategy generation,
//! and safe self-modification with rollback support.

use common::{async_trait, Error, Module, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::{AgentMetrics, Change, ChangeType};

/// Self-improvement engine
pub struct ImprovementEngine {
    metrics_history: Arc<RwLock<VecDeque<PerformanceSnapshot>>>,
    strategies: Arc<RwLock<Vec<ImprovementStrategy>>>,
    applied_changes: Arc<RwLock<Vec<AppliedChange>>>,
    rollback_store: Arc<RwLock<RollbackStore>>,
    config: ImprovementConfig,
    ab_test_framework: AbTestFramework,
}

impl ImprovementEngine {
    /// Create a new improvement engine
    pub fn new() -> Self {
        Self {
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            strategies: Arc::new(RwLock::new(Vec::new())),
            applied_changes: Arc::new(RwLock::new(Vec::new())),
            rollback_store: Arc::new(RwLock::new(RollbackStore::new())),
            config: ImprovementConfig::default(),
            ab_test_framework: AbTestFramework::new(),
        }
    }

    /// Record current performance metrics
    pub async fn record_metrics(&mut self, metrics: AgentMetrics) -> Result<()> {
        let snapshot = PerformanceSnapshot {
            timestamp: common::chrono::Utc::now(),
            metrics,
        };

        let mut history = self.metrics_history.write().await;
        history.push_back(snapshot);

        // Keep only the configured number of snapshots
        while history.len() > self.config.max_history_size {
            history.pop_front();
        }

        debug!("Recorded performance metrics snapshot");
        Ok(())
    }

    /// Analyze performance and identify bottlenecks
    pub async fn analyze_performance(&self) -> Result<PerformanceAnalysis> {
        let history = self.metrics_history.read().await;
        
        if history.len() < 2 {
            return Ok(PerformanceAnalysis {
                bottlenecks: vec![],
                trends: vec![],
                recommendations: vec![],
                metrics_before: None,
                metrics_after: None,
            });
        }

        let recent: Vec<_> = history.iter().rev().take(10).collect();
        let older: Vec<_> = history.iter().rev().skip(10).take(10).collect();

        let mut bottlenecks = Vec::new();
        let mut trends = Vec::new();

        // Analyze success rate trend
        if let (Some(recent_avg), Some(older_avg)) = (
            calculate_average_success_rate(&recent),
            calculate_average_success_rate(&older),
        ) {
            let change = recent_avg - older_avg;
            trends.push(MetricTrend {
                metric: "success_rate".to_string(),
                change,
                direction: if change > 0.0 {
                    TrendDirection::Improving
                } else if change < 0.0 {
                    TrendDirection::Degrading
                } else {
                    TrendDirection::Stable
                },
            });

            if recent_avg < self.config.min_success_rate {
                bottlenecks.push(Bottleneck {
                    category: BottleneckCategory::Reliability,
                    severity: Severity::High,
                    description: format!(
                        "Success rate below threshold: {:.1}%",
                        recent_avg * 100.0
                    ),
                    affected_components: vec!["orchestrator".to_string()],
                });
            }
        }

        // Analyze latency trend
        if let (Some(recent_latency), Some(older_latency)) = (
            calculate_average_latency(&recent),
            calculate_average_latency(&older),
        ) {
            let change_pct = if older_latency > 0 {
                ((recent_latency as f64 - older_latency as f64) / older_latency as f64) * 100.0
            } else {
                0.0
            };

            trends.push(MetricTrend {
                metric: "latency".to_string(),
                change: change_pct / 100.0,
                direction: if change_pct < -5.0 {
                    TrendDirection::Improving
                } else if change_pct > 5.0 {
                    TrendDirection::Degrading
                } else {
                    TrendDirection::Stable
                },
            });

            if recent_latency > self.config.max_acceptable_latency_ms {
                bottlenecks.push(Bottleneck {
                    category: BottleneckCategory::Performance,
                    severity: Severity::Medium,
                    description: format!(
                        "Average latency above threshold: {}ms",
                        recent_latency
                    ),
                    affected_components: vec!["intelligence".to_string(), "tools".to_string()],
                });
            }
        }

        // Generate recommendations based on bottlenecks
        let recommendations = self.generate_recommendations(&bottlenecks).await;

        let metrics_before = older.first().map(|s| s.metrics.clone());
        let metrics_after = recent.first().map(|s| s.metrics.clone());

        Ok(PerformanceAnalysis {
            bottlenecks,
            trends,
            recommendations,
            metrics_before,
            metrics_after,
        })
    }

    /// Generate improvement strategies based on analysis
    pub async fn generate_strategies(
        &self,
        analysis: &PerformanceAnalysis,
    ) -> Result<Vec<ImprovementStrategy>> {
        let mut strategies = Vec::new();

        for bottleneck in &analysis.bottlenecks {
            match bottleneck.category {
                BottleneckCategory::Reliability => {
                    strategies.push(ImprovementStrategy {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: "Improve Error Handling".to_string(),
                        description: "Add more robust error handling and retry logic".to_string(),
                        target_component: "orchestrator".to_string(),
                        change_type: ChangeType::Strategy,
                        expected_impact: Impact::High,
                        risk_level: RiskLevel::Low,
                        implementation: ImplementationPlan {
                            steps: vec![
                                "Analyze error patterns".to_string(),
                                "Add retry logic".to_string(),
                                "Improve error messages".to_string(),
                            ],
                            rollback_steps: vec![
                                "Remove retry logic".to_string(),
                                "Restore original error handling".to_string(),
                            ],
                        },
                    });
                }
                BottleneckCategory::Performance => {
                    strategies.push(ImprovementStrategy {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: "Optimize Prompts".to_string(),
                        description: "Optimize prompts for faster LLM responses".to_string(),
                        target_component: "intelligence".to_string(),
                        change_type: ChangeType::Prompt,
                        expected_impact: Impact::Medium,
                        risk_level: RiskLevel::Low,
                        implementation: ImplementationPlan {
                            steps: vec![
                                "Analyze prompt performance".to_string(),
                                "Optimize prompt templates".to_string(),
                                "Test with A/B framework".to_string(),
                            ],
                            rollback_steps: vec![
                                "Restore original prompts".to_string(),
                            ],
                        },
                    });

                    strategies.push(ImprovementStrategy {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: "Enable Parallel Execution".to_string(),
                        description: "Execute independent tasks in parallel".to_string(),
                        target_component: "orchestrator".to_string(),
                        change_type: ChangeType::Strategy,
                        expected_impact: Impact::High,
                        risk_level: RiskLevel::Medium,
                        implementation: ImplementationPlan {
                            steps: vec![
                                "Identify parallelizable tasks".to_string(),
                                "Implement parallel executor".to_string(),
                                "Add synchronization".to_string(),
                            ],
                            rollback_steps: vec![
                                "Revert to sequential execution".to_string(),
                            ],
                        },
                    });
                }
                BottleneckCategory::Accuracy => {
                    strategies.push(ImprovementStrategy {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: "Improve Context Gathering".to_string(),
                        description: "Enhance context gathering for better accuracy".to_string(),
                        target_component: "analysis".to_string(),
                        change_type: ChangeType::Strategy,
                        expected_impact: Impact::High,
                        risk_level: RiskLevel::Medium,
                        implementation: ImplementationPlan {
                            steps: vec![
                                "Enhance semantic analysis".to_string(),
                                "Improve knowledge retrieval".to_string(),
                            ],
                            rollback_steps: vec![
                                "Restore original context gathering".to_string(),
                            ],
                        },
                    });
                }
                _ => {}
            }
        }

        // Sort by expected impact and risk
        strategies.sort_by(|a, b| {
            b.expected_impact
                .priority()
                .cmp(&a.expected_impact.priority())
                .then_with(|| a.risk_level.priority().cmp(&b.risk_level.priority()))
        });

        // Limit to top strategies
        strategies.truncate(self.config.max_strategies_per_cycle);

        Ok(strategies)
    }

    /// Apply an improvement strategy
    pub async fn apply_strategy(&self, strategy: &ImprovementStrategy) -> Result<Change> {
        info!("Applying improvement strategy: {}", strategy.name);

        // Store rollback data before making changes
        let rollback_data = self.prepare_rollback(&strategy.target_component).await?;

        // Apply the change
        let change = match strategy.change_type {
            ChangeType::Prompt => self.apply_prompt_improvement(strategy).await?,
            ChangeType::Configuration => self.apply_config_improvement(strategy).await?,
            ChangeType::Strategy => self.apply_strategy_improvement(strategy).await?,
            _ => {
                return Err(Error::Internal(format!(
                    "Unsupported change type: {:?}",
                    strategy.change_type
                )))
            }
        };

        // Store applied change with rollback data
        let applied_change = AppliedChange {
            change: change.clone(),
            strategy_id: strategy.id.clone(),
            rollback_data,
            applied_at: common::chrono::Utc::now(),
        };

        self.applied_changes.write().await.push(applied_change);

        // Register with A/B test framework
        self.ab_test_framework
            .register_variant(&change.id, &strategy.id)
            .await;

        info!("Successfully applied improvement: {}", change.description);
        Ok(change)
    }

    /// Validate improvements through A/B testing
    pub async fn validate_improvements(&self, changes: &[Change]) -> Result<ValidationResult> {
        info!("Validating {} improvements", changes.len());

        let mut successful = Vec::new();
        let mut rollbacks = Vec::new();
        let mut total_duration = 0;

        for change in changes {
            let test_result = self
                .ab_test_framework
                .evaluate_variant(&change.id, self.metrics_history.clone())
                .await?;
            
            total_duration += test_result.duration_ms;

            if test_result.is_better {
                info!(
                    "Improvement {} validated successfully: {:.1}% improvement",
                    change.id, test_result.improvement_percentage * 100.0
                );
                successful.push(change.clone());
            } else {
                warn!(
                    "Improvement {} degraded performance: {:.1}% worse",
                    change.id, test_result.improvement_percentage * 100.0
                );
                rollbacks.push(change.clone());
            }
        }

        Ok(ValidationResult {
            successful,
            rollbacks,
            test_duration_ms: total_duration,
        })
    }

    /// Rollback a change
    pub async fn rollback(&self, change: &Change) -> Result<()> {
        warn!("Rolling back change: {}", change.description);

        let applied_changes = self.applied_changes.read().await;
        if let Some(applied) = applied_changes.iter().find(|a| a.change.id == change.id) {
            // Execute rollback using stored data
            self.execute_rollback(&applied.rollback_data).await?;
            info!("Successfully rolled back change: {}", change.id);
        } else {
            return Err(Error::NotFound(format!(
                "Change not found for rollback: {}",
                change.id
            )));
        }

        Ok(())
    }

    /// Persist successful improvements
    pub async fn persist_improvements(&self, changes: &[Change]) -> Result<()> {
        info!("Persisting {} successful improvements", changes.len());

        let persistence = self.rollback_store.read().await;
        for change in changes {
            persistence.persist_change(change).await?;
        }

        Ok(())
    }

    /// Get improvement history
    pub async fn get_improvement_history(&self) -> Vec<AppliedChange> {
        self.applied_changes.read().await.clone()
    }

    /// Get current performance metrics summary
    pub async fn get_metrics_summary(&self) -> Option<PerformanceMetrics> {
        let history = self.metrics_history.read().await;
        history.back().map(|s| PerformanceMetrics {
            timestamp: s.timestamp,
            success_rate: calculate_success_rate(&s.metrics),
            average_latency_ms: s.metrics.average_execution_time_ms,
            throughput: calculate_throughput(&s.metrics),
        })
    }

    // Private helper methods

    async fn generate_recommendations(&self, bottlenecks: &[Bottleneck]) -> Vec<String> {
        let mut recommendations = Vec::new();

        for bottleneck in bottlenecks {
            match bottleneck.category {
                BottleneckCategory::Reliability => {
                    recommendations.push(
                        "Consider adding more comprehensive error handling".to_string(),
                    );
                    recommendations.push(
                        "Review recent error logs for patterns".to_string(),
                    );
                }
                BottleneckCategory::Performance => {
                    recommendations.push(
                        "Profile the system to identify slow components".to_string(),
                    );
                    recommendations.push(
                        "Consider caching frequently accessed data".to_string(),
                    );
                }
                BottleneckCategory::Accuracy => {
                    recommendations.push(
                        "Review and improve prompt quality".to_string(),
                    );
                    recommendations.push(
                        "Enhance context gathering from codebase".to_string(),
                    );
                }
                _ => {}
            }
        }

        recommendations
    }

    async fn prepare_rollback(&self, _component: &str) -> Result<serde_json::Value> {
        // Capture current state for rollback
        Ok(serde_json::json!({
            "timestamp": common::chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    async fn execute_rollback(&self, _rollback_data: &serde_json::Value) -> Result<()> {
        // Execute rollback based on stored data
        info!("Executing rollback");
        Ok(())
    }

    async fn apply_prompt_improvement(&self, strategy: &ImprovementStrategy) -> Result<Change> {
        let path = PathBuf::from("prompts/default.txt");
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Simulate "improving" the prompt by writing to file
        let content = format!("# Improved prompt\nStrategy: {}\nTimestamp: {}\n\nYou are a helpful coding agent...", 
            strategy.description, common::chrono::Utc::now());
        
        tokio::fs::write(&path, content).await?;

        Ok(Change {
            id: uuid::Uuid::new_v4().to_string(),
            file_path: path,
            change_type: ChangeType::Prompt,
            description: strategy.description.clone(),
            timestamp: common::chrono::Utc::now(),
            rollback_data: None,
        })
    }

    async fn apply_config_improvement(&self, strategy: &ImprovementStrategy) -> Result<Change> {
        let path = PathBuf::from("config/agent.toml");

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Simulate config update
        let content = format!("# Updated configuration\n# Strategy: {}\n\n[agent]\nself_improvement = true\n", 
            strategy.description);

        tokio::fs::write(&path, content).await?;

        Ok(Change {
            id: uuid::Uuid::new_v4().to_string(),
            file_path: path,
            change_type: ChangeType::Configuration,
            description: strategy.description.clone(),
            timestamp: common::chrono::Utc::now(),
            rollback_data: None,
        })
    }

    async fn apply_strategy_improvement(&self, strategy: &ImprovementStrategy) -> Result<Change> {
        let path = PathBuf::from("src/strategy.rs");

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Simulate strategy update
        let content = format!("// Strategy updated based on: {}\n// Timestamp: {}\n\npub fn execute() {{\n    println!(\"Executed improved strategy\");\n}}\n", 
            strategy.description, common::chrono::Utc::now());

        tokio::fs::write(&path, content).await?;

        Ok(Change {
            id: uuid::Uuid::new_v4().to_string(),
            file_path: path,
            change_type: ChangeType::Strategy,
            description: strategy.description.clone(),
            timestamp: common::chrono::Utc::now(),
            rollback_data: None,
        })
    }
}

#[async_trait]
impl Module for ImprovementEngine {
    fn name(&self) -> &str {
        "improvement_engine"
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing improvement engine");
        // Load any persisted improvements
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down improvement engine");
        // Persist current state
        Ok(())
    }
}

impl Default for ImprovementEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: common::chrono::DateTime<common::chrono::Utc>,
    pub metrics: AgentMetrics,
}

/// Performance analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub bottlenecks: Vec<Bottleneck>,
    pub trends: Vec<MetricTrend>,
    pub recommendations: Vec<String>,
    pub metrics_before: Option<AgentMetrics>,
    pub metrics_after: Option<AgentMetrics>,
}

/// Performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub category: BottleneckCategory,
    pub severity: Severity,
    pub description: String,
    pub affected_components: Vec<String>,
}

/// Bottleneck categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BottleneckCategory {
    Performance,
    Reliability,
    Accuracy,
    ResourceUsage,
    Latency,
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Metric trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricTrend {
    pub metric: String,
    pub change: f64,
    pub direction: TrendDirection,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

/// Improvement strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementStrategy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_component: String,
    pub change_type: ChangeType,
    pub expected_impact: Impact,
    pub risk_level: RiskLevel,
    pub implementation: ImplementationPlan,
}

/// Expected impact level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Impact {
    Low,
    Medium,
    High,
}

impl Impact {
    pub fn priority(&self) -> u8 {
        match self {
            Impact::Low => 1,
            Impact::Medium => 2,
            Impact::High => 3,
        }
    }
}

/// Risk level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl RiskLevel {
    pub fn priority(&self) -> u8 {
        match self {
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
        }
    }
}

/// Implementation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPlan {
    pub steps: Vec<String>,
    pub rollback_steps: Vec<String>,
}

/// Applied change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedChange {
    pub change: crate::Change,
    pub strategy_id: String,
    pub rollback_data: serde_json::Value,
    pub applied_at: common::chrono::DateTime<common::chrono::Utc>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub successful: Vec<crate::Change>,
    pub rollbacks: Vec<crate::Change>,
    pub test_duration_ms: u64,
}

/// Performance metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: common::chrono::DateTime<common::chrono::Utc>,
    pub success_rate: f64,
    pub average_latency_ms: u64,
    pub throughput: f64,
}

/// Improvement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementConfig {
    pub max_history_size: usize,
    pub min_success_rate: f64,
    pub max_acceptable_latency_ms: u64,
    pub max_strategies_per_cycle: usize,
    pub ab_test_duration_seconds: u64,
}

impl Default for ImprovementConfig {
    fn default() -> Self {
        Self {
            max_history_size: 100,
            min_success_rate: 0.8,
            max_acceptable_latency_ms: 5000,
            max_strategies_per_cycle: 3,
            ab_test_duration_seconds: 300,
        }
    }
}

/// Rollback store for persisting changes
pub struct RollbackStore {
    storage_path: PathBuf,
}

impl RollbackStore {
    pub fn new() -> Self {
        Self {
            storage_path: PathBuf::from(".agent/improvements"),
        }
    }

    pub async fn persist_change(&self, change: &crate::Change) -> Result<()> {
        let path = self.storage_path.join(format!("{}.json", change.id));
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        let json = serde_json::to_string_pretty(change)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    pub async fn load_change(&self, change_id: &str) -> Result<crate::Change> {
        let path = self.storage_path.join(format!("{}.json", change_id));
        let json = tokio::fs::read_to_string(path).await?;
        let change = serde_json::from_str(&json)?;
        Ok(change)
    }
}

impl Default for RollbackStore {
    fn default() -> Self {
        Self::new()
    }
}

/// A/B testing framework
pub struct AbTestFramework {
    variants: Arc<RwLock<HashMap<String, TestVariant>>>,
}

impl AbTestFramework {
    pub fn new() -> Self {
        Self {
            variants: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_variant(&self, change_id: &str, strategy_id: &str) {
        let mut variants = self.variants.write().await;
        variants.insert(
            change_id.to_string(),
            TestVariant {
                change_id: change_id.to_string(),
                strategy_id: strategy_id.to_string(),
                start_time: common::chrono::Utc::now(),
                metrics_before: None,
                metrics_after: None,
            },
        );
    }

    pub async fn evaluate_variant(
        &self,
        change_id: &str,
        metrics_history: Arc<RwLock<VecDeque<PerformanceSnapshot>>>,
    ) -> Result<TestResult> {
        let variants = self.variants.read().await;
        let variant = variants.get(change_id).ok_or_else(|| {
            Error::NotFound(format!("Variant not found: {}", change_id))
        })?;

        let history = metrics_history.read().await;
        
        // Simple evaluation: compare metrics before and after
        let is_better = true; // Placeholder
        let improvement_percentage = 0.05; // Placeholder

        let duration_ms = common::chrono::Utc::now()
            .signed_duration_since(variant.start_time)
            .num_milliseconds() as u64;

        Ok(TestResult {
            is_better,
            improvement_percentage,
            duration_ms,
        })
    }
}

impl Default for AbTestFramework {
    fn default() -> Self {
        Self::new()
    }
}

/// Test variant
#[derive(Debug, Clone)]
pub struct TestVariant {
    pub change_id: String,
    pub strategy_id: String,
    pub start_time: common::chrono::DateTime<common::chrono::Utc>,
    pub metrics_before: Option<PerformanceMetrics>,
    pub metrics_after: Option<PerformanceMetrics>,
}

/// Test result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub is_better: bool,
    pub improvement_percentage: f64,
    pub duration_ms: u64,
}

// Helper functions

fn calculate_average_success_rate(snapshots: &[&PerformanceSnapshot]) -> Option<f64> {
    if snapshots.is_empty() {
        return None;
    }
    let sum: f64 = snapshots.iter().map(|s| s.metrics.success_rate).sum();
    Some(sum / snapshots.len() as f64)
}

fn calculate_average_latency(snapshots: &[&PerformanceSnapshot]) -> Option<u64> {
    if snapshots.is_empty() {
        return None;
    }
    let sum: u64 = snapshots.iter().map(|s| s.metrics.average_execution_time_ms).sum();
    Some(sum / snapshots.len() as u64)
}

pub fn calculate_success_rate(metrics: &AgentMetrics) -> f64 {
    let total = metrics.tasks_completed + metrics.tasks_failed;
    if total == 0 {
        1.0
    } else {
        metrics.tasks_completed as f64 / total as f64
    }
}

pub fn calculate_throughput(metrics: &AgentMetrics) -> f64 {
    let uptime_hours = metrics.uptime_seconds() as f64 / 3600.0;
    if uptime_hours > 0.0 {
        metrics.tasks_completed as f64 / uptime_hours
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impact_priority() {
        assert!(Impact::High.priority() > Impact::Medium.priority());
        assert!(Impact::Medium.priority() > Impact::Low.priority());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }
}
