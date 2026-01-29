//! Knowledge graph module.
//!
//! This module provides graph-based knowledge representation
//! for relationships between concepts, files, and code elements.

use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Knowledge graph for storing entities and relations
pub struct KnowledgeGraph {
    entities: HashMap<String, Entity>,
    relations: Vec<Relation>,
    // TODO: Add Neo4j or other graph database client
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            relations: Vec::new(),
        }
    }

    /// Initialize the graph connection
    pub async fn initialize(&mut self) -> Result<()> {
        // TODO: Connect to graph database
        Ok(())
    }

    /// Shutdown the graph connection
    pub async fn shutdown(&mut self) -> Result<()> {
        // TODO: Close graph database connection
        Ok(())
    }

    /// Add an entity to the graph
    pub async fn add_entity(&mut self, entity: Entity) -> Result<()> {
        self.entities.insert(entity.id.clone(), entity);
        Ok(())
    }

    /// Add a relation between entities
    pub async fn add_relation(&mut self, relation: Relation) -> Result<()> {
        // Validate entities exist
        if !self.entities.contains_key(&relation.source) {
            return Err(Error::NotFound(format!("Source entity not found: {}", relation.source)));
        }
        if !self.entities.contains_key(&relation.target) {
            return Err(Error::NotFound(format!("Target entity not found: {}", relation.target)));
        }

        self.relations.push(relation);
        Ok(())
    }

    /// Get an entity by ID
    pub fn get_entity(&self, id: &str) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// Find relations for an entity
    pub fn find_relations(&self, entity_id: &str) -> Vec<&Relation> {
        self.relations
            .iter()
            .filter(|r| r.source == entity_id || r.target == entity_id)
            .collect()
    }

    /// Search the graph
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<super::SearchResult>> {
        let mut results = Vec::new();

        // Search entities by name or properties
        for entity in self.entities.values() {
            if entity.name.contains(query) {
                results.push(super::SearchResult {
                    source: entity.id.clone(),
                    content: format!("{}: {:?}", entity.name, entity.entity_type),
                    relevance: 0.9,
                    result_type: super::ResultType::Historical,
                });
            }
        }

        results.truncate(limit);
        Ok(results)
    }

    /// Execute a graph query
    pub async fn query(&self, query: &super::GraphQuery) -> Result<Vec<super::GraphResult>> {
        match query.query_type {
            super::QueryType::FindEntity => self.query_find_entity(query).await,
            super::QueryType::FindRelations => self.query_find_relations(query).await,
            super::QueryType::FindPath => self.query_find_path(query).await,
            super::QueryType::SimilarEntities => self.query_similar_entities(query).await,
        }
    }

    async fn query_find_entity(&self, query: &super::GraphQuery) -> Result<Vec<super::GraphResult>> {
        // TODO: Implement entity finding
        let entity_name = query
            .parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Entity name required".to_string()))?;

        let mut results = Vec::new();
        for entity in self.entities.values() {
            if entity.name == entity_name {
                results.push(super::GraphResult {
                    entities: vec![entity.clone()],
                    relations: vec![],
                    score: 1.0,
                });
            }
        }

        Ok(results)
    }

    async fn query_find_relations(&self, query: &super::GraphQuery) -> Result<Vec<super::GraphResult>> {
        // TODO: Implement relation finding
        let entity_id = query
            .parameters
            .get("entity_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Entity ID required".to_string()))?;

        let relations = self.find_relations(entity_id);
        let related_entities: Vec<Entity> = relations
            .iter()
            .filter_map(|r| {
                if r.source == entity_id {
                    self.entities.get(&r.target).cloned()
                } else {
                    self.entities.get(&r.source).cloned()
                }
            })
            .collect();

        Ok(vec![super::GraphResult {
            entities: related_entities,
            relations: relations.into_iter().cloned().collect(),
            score: 1.0,
        }])
    }

    async fn query_find_path(&self, query: &super::GraphQuery) -> Result<Vec<super::GraphResult>> {
        let from_id = query.parameters.get("from").and_then(|v| v.as_str()).ok_or_else(|| Error::Validation("From ID required".to_string()))?;
        let to_id = query.parameters.get("to").and_then(|v| v.as_str()).ok_or_else(|| Error::Validation("To ID required".to_string()))?;

        if !self.entities.contains_key(from_id) {
            return Err(Error::NotFound(format!("Start entity not found: {}", from_id)));
        }
        if !self.entities.contains_key(to_id) {
            return Err(Error::NotFound(format!("End entity not found: {}", to_id)));
        }

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(vec![from_id.to_string()]);

        let mut visited = std::collections::HashSet::new();
        visited.insert(from_id.to_string());

        while let Some(path) = queue.pop_front() {
            let current = path.last().unwrap();

            if current == to_id {
                // Found path
                let mut entities = Vec::new();
                let mut relations = Vec::new();

                for i in 0..path.len() {
                    entities.push(self.entities.get(&path[i]).unwrap().clone());
                    if i < path.len() - 1 {
                        // Find relation between current and next
                        if let Some(rel) = self.relations.iter().find(|r|
                            (r.source == path[i] && r.target == path[i+1]) ||
                            (r.source == path[i+1] && r.target == path[i])
                        ) {
                            relations.push(rel.clone());
                        }
                    }
                }

                return Ok(vec![super::GraphResult {
                    entities,
                    relations,
                    score: 1.0,
                }]);
            }

            // Find neighbors
            let neighbors: Vec<String> = self.relations.iter()
                .filter(|r| r.source == *current || r.target == *current)
                .map(|r| if r.source == *current { r.target.clone() } else { r.source.clone() })
                .collect();

            for neighbor in neighbors {
                if visited.insert(neighbor.clone()) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    queue.push_back(new_path);
                }
            }
        }

        Ok(vec![])
    }

    async fn query_similar_entities(&self, query: &super::GraphQuery) -> Result<Vec<super::GraphResult>> {
        let entity_type_str = query.parameters.get("entity_type").and_then(|v| v.as_str());
        let properties = query.parameters.get("properties").and_then(|v| v.as_object());

        let mut results = Vec::new();

        for entity in self.entities.values() {
            // Filter by entity type if specified
            if let Some(type_str) = entity_type_str {
                // Simple string matching for now since we can't easily parse the enum back from string without FromStr
                if format!("{:?}", entity.entity_type).to_lowercase() != type_str.to_lowercase() {
                    continue;
                }
            }

            // Filter by properties if specified
            if let Some(props) = properties {
                let mut match_count = 0;
                let mut total_props = 0;

                for (key, value) in props {
                    total_props += 1;
                    if let Some(entity_val) = entity.properties.get(key) {
                        if entity_val == value {
                            match_count += 1;
                        }
                    }
                }

                if match_count > 0 {
                    let score = match_count as f32 / total_props as f32;
                    results.push(super::GraphResult {
                        entities: vec![entity.clone()],
                        relations: vec![],
                        score,
                    });
                }
            } else if entity_type_str.is_some() {
                // If only type is specified, return matches with score 1.0
                results.push(super::GraphResult {
                    entities: vec![entity.clone()],
                    relations: vec![],
                    score: 1.0,
                });
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    /// Find impact of changing an entity
    pub fn find_impact(&self, entity_id: &str) -> Vec<&Entity> {
        // Find all entities that depend on the given entity
        let mut impacted = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut to_visit = vec![entity_id];

        while let Some(current) = to_visit.pop() {
            if visited.insert(current) {
                for relation in &self.relations {
                    if relation.source == current && relation.relation_type == RelationType::DependsOn {
                        if let Some(entity) = self.entities.get(&relation.target) {
                            to_visit.push(&relation.target);
                            impacted.push(entity);
                        }
                    }
                }
            }
        }

        impacted
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Entity in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Types of entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Concept,
    File,
    Function,
    Type,
    Task,
    Module,
    Variable,
}

/// Relation between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub source: String,
    pub target: String,
    pub relation_type: RelationType,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Types of relations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    DependsOn,
    Implements,
    Uses,
    Contains,
    RelatesTo,
    Calls,
    Imports,
    Extends,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_entity() {
        let mut graph = KnowledgeGraph::new();
        let entity = Entity {
            id: "test-1".to_string(),
            name: "Test Entity".to_string(),
            entity_type: EntityType::Concept,
            properties: HashMap::new(),
        };

        graph.add_entity(entity.clone()).await.unwrap();
        assert!(graph.get_entity("test-1").is_some());
    }

    #[tokio::test]
    async fn test_add_relation() {
        let mut graph = KnowledgeGraph::new();

        let entity1 = Entity {
            id: "test-1".to_string(),
            name: "Entity 1".to_string(),
            entity_type: EntityType::Concept,
            properties: HashMap::new(),
        };

        let entity2 = Entity {
            id: "test-2".to_string(),
            name: "Entity 2".to_string(),
            entity_type: EntityType::Concept,
            properties: HashMap::new(),
        };

        graph.add_entity(entity1).await.unwrap();
        graph.add_entity(entity2).await.unwrap();

        let relation = Relation {
            source: "test-1".to_string(),
            target: "test-2".to_string(),
            relation_type: RelationType::DependsOn,
            properties: HashMap::new(),
        };

        graph.add_relation(relation).await.unwrap();
        assert_eq!(graph.find_relations("test-1").len(), 1);
    }

    #[tokio::test]
    async fn test_find_path() {
        let mut graph = KnowledgeGraph::new();
        let e1 = Entity { id: "1".to_string(), name: "1".to_string(), entity_type: EntityType::Concept, properties: HashMap::new() };
        let e2 = Entity { id: "2".to_string(), name: "2".to_string(), entity_type: EntityType::Concept, properties: HashMap::new() };
        let e3 = Entity { id: "3".to_string(), name: "3".to_string(), entity_type: EntityType::Concept, properties: HashMap::new() };

        graph.add_entity(e1).await.unwrap();
        graph.add_entity(e2).await.unwrap();
        graph.add_entity(e3).await.unwrap();

        graph.add_relation(Relation { source: "1".to_string(), target: "2".to_string(), relation_type: RelationType::DependsOn, properties: HashMap::new() }).await.unwrap();
        graph.add_relation(Relation { source: "2".to_string(), target: "3".to_string(), relation_type: RelationType::DependsOn, properties: HashMap::new() }).await.unwrap();

        let query = super::super::GraphQuery {
            query_type: super::super::QueryType::FindPath,
            parameters: {
                let mut p = HashMap::new();
                p.insert("from".to_string(), serde_json::json!("1"));
                p.insert("to".to_string(), serde_json::json!("3"));
                p
            },
        };

        let result = graph.query(&query).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].entities.len(), 3);
    }

    #[tokio::test]
    async fn test_similar_entities() {
        let mut graph = KnowledgeGraph::new();
        let mut props = HashMap::new();
        props.insert("tag".to_string(), serde_json::json!("rust"));

        let e1 = Entity { id: "1".to_string(), name: "1".to_string(), entity_type: EntityType::Concept, properties: props.clone() };
        let e2 = Entity { id: "2".to_string(), name: "2".to_string(), entity_type: EntityType::Concept, properties: props };
        let e3 = Entity { id: "3".to_string(), name: "3".to_string(), entity_type: EntityType::Concept, properties: HashMap::new() };

        graph.add_entity(e1).await.unwrap();
        graph.add_entity(e2).await.unwrap();
        graph.add_entity(e3).await.unwrap();

        let query = super::super::GraphQuery {
            query_type: super::super::QueryType::SimilarEntities,
            parameters: {
                let mut p = HashMap::new();
                let mut props_search = serde_json::Map::new();
                props_search.insert("tag".to_string(), serde_json::json!("rust"));
                p.insert("properties".to_string(), serde_json::Value::Object(props_search));
                p
            },
        };

        let result = graph.query(&query).await.unwrap();
        assert_eq!(result.len(), 2);
    }
}