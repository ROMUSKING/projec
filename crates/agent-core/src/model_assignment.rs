//! Model assignment logic based on performance reports.
//! 
//! This module implements the deterministic assignment of models to roles.

use crate::reporting::AggregateReport;
use agent_config::{AgentConfig, RoutingConfig, ProviderRoute};
use common::{Result, Error};

pub struct ModelAssigner;

impl ModelAssigner {
    /// Update configuration based on performance report
    pub fn update_assignments(config: &mut AgentConfig, report: &AggregateReport) -> Result<()> {
        let mut best_models: Vec<ProviderRoute> = Vec::new();
        
        // Simple algorithm: Score = Success Rate * (1000 / Latency) * (1 / Cost)
        // This favors high success, low latency, low cost.
        
        for model_stats in &report.models {
            let latency_factor = if model_stats.avg_latency_ms > 0.0 {
                1000.0 / model_stats.avg_latency_ms
            } else {
                0.0
            };
            
            let cost_factor = if model_stats.total_cost > 0.0 {
                1.0 / (model_stats.total_cost + 0.0001) // Avoid div by zero
            } else {
                100.0 // Assume low cost if 0
            };
            
            let score = model_stats.success_rate as f64 * latency_factor * cost_factor;
            
            // Find existing route config to preserve API keys/urls
            if let Some(existing) = config.llm.routing.providers.iter().find(|p| p.model == model_stats.model_name) {
                let mut updated = existing.clone();
                // Update priority based on score (higher score -> lower priority number = better)
                // Normalize score roughly to 1-100 priority
                let priority = (1000.0 / (score + 1.0)).clamp(1.0, 100.0) as u32;
                updated.priority = priority;
                best_models.push(updated);
            }
        }
        
        if !best_models.is_empty() {
            // Sort by priority
            best_models.sort_by_key(|m| m.priority);
            
            // Update config
            config.llm.routing.providers = best_models;
        }
        
        Ok(())
    }
}
