//! Reporting and aggregation for agent performance.
//! 
//! This module aggregates survey data and generates reports.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use common::{Result, Error};
use crate::telemetry::SurveyData;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ModelPerformance {
    pub model_name: String,
    pub total_tasks: u32,
    pub success_rate: f32,
    pub avg_latency_ms: f64,
    pub avg_score: f32,
    pub total_cost: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregateReport {
    pub generated_at: common::chrono::DateTime<common::chrono::Utc>,
    pub models: Vec<ModelPerformance>,
}

pub struct ReportGenerator;

impl ReportGenerator {
    pub fn generate(data: &[SurveyData]) -> AggregateReport {
        let mut map: HashMap<String, Vec<&SurveyData>> = HashMap::new();
        
        for item in data {
            map.entry(item.model.clone()).or_default().push(item);
        }
        
        let mut models = Vec::new();
        
        for (name, items) in map {
            let total = items.len() as u32;
            let successes = items.iter().filter(|i| i.success).count();
            let avg_lat = items.iter().map(|i| i.latency_ms).sum::<u64>() as f64 / total as f64;
            let avg_score = items.iter().filter_map(|i| i.score).sum::<f32>() / total as f32;
            let cost = items.iter().map(|i| i.cost_usd).sum();
            
            models.push(ModelPerformance {
                model_name: name,
                total_tasks: total,
                success_rate: if total > 0 { successes as f32 / total as f32 } else { 0.0 },
                avg_latency_ms: avg_lat,
                avg_score: avg_score,
                total_cost: cost,
            });
        }
        
        AggregateReport {
            generated_at: common::chrono::Utc::now(),
            models,
        }
    }
}
