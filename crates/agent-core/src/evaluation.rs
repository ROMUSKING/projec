//! Cross-evaluation module for agent peer review.
//! 
//! This module implements scoring rubrics, persona-based evaluation,
//! and data aggregation for performance analysis.

use serde::{Deserialize, Serialize};
use common::{Result, Error};
use crate::intelligence::IntentCategory;

/// A distinct role the agent assumes during evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Persona {
    /// The agent performing the work
    Worker,
    /// A persona focused on architectural correctness
    Architect,
    /// A persona focused on code quality, security, and bugs
    Reviewer,
    /// A persona focused on user experience and relevance
    ProductOwner,
}

impl std::fmt::Display for Persona {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Persona::Worker => write!(f, "Worker"),
            Persona::Architect => write!(f, "Architect"),
            Persona::Reviewer => write!(f, "Reviewer"),
            Persona::ProductOwner => write!(f, "Product Owner"),
        }
    }
}

/// A specific metric to be scored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub description: String,
    pub weight: f32,
    /// Score from 1 to 10
    pub score: u8,
    pub reasoning: String,
}

/// The rubric defining how to score a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rubric {
    pub criteria: Vec<Metric>,
}

impl Rubric {
    pub fn for_intent(intent: &IntentCategory) -> Self {
        match intent {
            IntentCategory::CodeGeneration | IntentCategory::CodeModification => Rubric {
                criteria: vec![
                    Metric {
                        name: "Accuracy".into(),
                        description: "Does the code do exactly what was requested?".into(),
                        weight: 1.5,
                        score: 0,
                        reasoning: String::new(),
                    },
                    Metric {
                        name: "Efficiency".into(),
                        description: "Is the code performant and resource-light?".into(),
                        weight: 1.0,
                        score: 0,
                        reasoning: String::new(),
                    },
                    Metric {
                        name: "Safety".into(),
                        description: "Are there security vulnerabilities or unsafe blocks?".into(),
                        weight: 2.0,
                        score: 0,
                        reasoning: String::new(),
                    },
                    Metric {
                        name: "Style".into(),
                        description: "Does it follow idiomatic conventions?".into(),
                        weight: 0.5,
                        score: 0,
                        reasoning: String::new(),
                    },
                ]
            },
            _ => Rubric {
                criteria: vec![
                    Metric {
                        name: "Relevance".into(),
                        description: "Is the output relevant to the user request?".into(),
                        weight: 1.0,
                        score: 0,
                        reasoning: String::new(),
                    },
                    Metric {
                        name: "Clarity".into(),
                        description: "Is the output easy to understand?".into(),
                        weight: 1.0,
                        score: 0,
                        reasoning: String::new(),
                    },
                ]
            }
        }
    }
}

/// The result of an evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationReport {
    pub task_id: String,
    pub evaluator: Persona,
    pub evaluatee: Persona,
    pub metrics: Vec<Metric>,
    pub weighted_score: f32,
    pub max_possible_score: f32,
    pub timestamp: common::chrono::DateTime<common::chrono::Utc>,
}

impl EvaluationReport {
    pub fn calculate_score(&mut self) {
        let mut total = 0.0;
        let mut max = 0.0;
        for metric in &self.metrics {
            total += metric.score as f32 * metric.weight;
            max += 10.0 * metric.weight;
        }
        self.weighted_score = total;
        self.max_possible_score = max;
    }
    
    pub fn summary(&self) -> String {
        let percentage = if self.max_possible_score > 0.0 {
            (self.weighted_score / self.max_possible_score) * 100.0
        } else {
            0.0
        };
        
        format!(
            "Evaluation by {}: {:.1}% ({:.1}/{:.1})", 
            self.evaluator, percentage, self.weighted_score, self.max_possible_score
        )
    }
}

/// Engine to drive the evaluation process
pub struct EvaluationEngine {
    // In a real system, this would connect to a DB
    history: std::sync::Arc<tokio::sync::RwLock<Vec<EvaluationReport>>>,
}

impl EvaluationEngine {
    pub fn new() -> Self {
        Self {
            history: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Perform a cross-evaluation using the LLM
    pub async fn evaluate(
        &self,
        intelligence: &crate::intelligence::IntelligenceEngine,
        task: &crate::Task,
        output: &str,
        evaluator_persona: Persona,
    ) -> Result<EvaluationReport> {
        let rubric = Rubric::for_intent(&task.intent.category);
        
        let mut criteria_strings = Vec::new();
        for m in &rubric.criteria {
            criteria_strings.push(format!("- {}: {}", m.name, m.description));
        }
        let criteria_text = criteria_strings.join("\n");

        let prompt = format!(
            r###"You are acting as a {}. 
Your goal is to evaluate the following work output based on strict criteria.

Task Description: "{}"

Output to Evaluate:
```
{}
```

Rubric Criteria:
{}

Provide your evaluation in JSON format matching this structure:
[
  {{ "name": "MetricName", "score": <1-10 integer>, "reasoning": "..." }}, ...
]
Only return the JSON array."###,
            evaluator_persona,
            task.description,
            common::utils::truncate(output, 2000), 
            criteria_text
        );

        // Call LLM
        let context = crate::intelligence::Context::default();
        let response = intelligence.generate(&context, &prompt).await?;
        
        // Parse JSON response
        let clean_json = response.content
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```");
            
        let scored_metrics: Vec<MetricRaw> = serde_json::from_str(clean_json)
            .map_err(|e| Error::Serialization(e))?;

        // Merge scores back into rubric
        let mut final_metrics = rubric.criteria;
        for metric in &mut final_metrics {
            if let Some(scored) = scored_metrics.iter().find(|m| m.name == metric.name) {
                metric.score = scored.score;
                metric.reasoning = scored.reasoning.clone();
            }
        }

        let mut report = EvaluationReport {
            task_id: task.id.to_string(),
            evaluator: evaluator_persona,
            evaluatee: Persona::Worker,
            metrics: final_metrics,
            weighted_score: 0.0,
            max_possible_score: 0.0,
            timestamp: common::chrono::Utc::now(),
        };
        
        report.calculate_score();
        
        // Store report
        self.history.write().await.push(report.clone());
        
        Ok(report)
    }
    
    pub async fn get_dashboard(&self) -> String {
        let history = self.history.read().await;
        if history.is_empty() {
            return "No evaluations recorded.".to_string();
        }
        
        let mut report = String::from("# Performance Dashboard\n\n");
        
        for eval in history.iter() {
            report.push_str(&format!("## Task {}\n", eval.task_id));
            report.push_str(&format!("**{}** -> **{}**: {}\n\n", eval.evaluator, eval.evaluatee, eval.summary()));
            
            report.push_str("| Metric | Score | Weight | Reasoning |\n");
            report.push_str("|---|---|---|---|
");
            for m in &eval.metrics {
                report.push_str(&format!("| {} | {}/10 | {} | {} |\n", m.name, m.score, m.weight, m.reasoning));
            }
            report.push_str("\n");
        }
        
        report
    }
}

#[derive(Deserialize)]
struct MetricRaw {
    name: String,
    score: u8,
    reasoning: String,
}