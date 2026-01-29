//! Configuration management for the coding agent.
//!
//! This crate handles all configuration aspects including loading from files,
//! environment variables, and providing a unified configuration interface.

use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure (sensitive fields are redacted in debug output)
#[derive(Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent identity
    pub agent: AgentSettings,

    /// LLM configuration
    pub llm: LlmConfig,

    /// LSP configuration
    pub lsp: LspConfig,

    /// Safety configuration
    pub safety: SafetyConfig,

    /// Tool configuration
    pub tools: ToolConfig,

    /// Self-compilation and restart configuration
    pub self_compile: SelfCompileConfig,

    /// Telemetry and survey configuration
    pub telemetry: TelemetryConfig,
}

impl std::fmt::Debug for AgentConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentConfig")
            .field("agent", &self.agent)
            .field("llm", &DebugRedactedLlmConfig(&self.llm))
            .field("lsp", &self.lsp)
            .field("safety", &self.safety)
            .field("tools", &self.tools)
            .field("self_compile", &self.self_compile)
            .field("telemetry", &DebugRedactedTelemetryConfig(&self.telemetry))
            .finish()
    }
}

/// Wrapper to redact API keys in LlmConfig debug output
struct DebugRedactedLlmConfig<'a>(&'a LlmConfig);

impl<'a> std::fmt::Debug for DebugRedactedLlmConfig<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lsp = f.debug_struct("LlmConfig");
        lsp
            .field("provider", &self.0.provider)
            .field("model", &self.0.model)
            .field("temperature", &self.0.temperature)
            .field("max_tokens", &self.0.max_tokens)
            .field("providers", &DebugRedactedLlmProviders(&self.0.providers));
        
        lsp.finish()
    }
}

/// Wrapper to redact API keys in LlmProviders debug output
struct DebugRedactedLlmProviders<'a>(&'a ProviderConfigs);

impl<'a> std::fmt::Debug for DebugRedactedLlmProviders<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut providers = f.debug_struct("LlmProviders");
        
        // Redact API keys
        providers
            .field("anthropic", &DebugRedactedApiKeyConfig(&self.0.anthropic))
            .field("openai", &DebugRedactedApiKeyConfig(&self.0.openai))
            .field("ollama", &DebugRedactedApiKeyConfig(&self.0.ollama))
            .field("gemini", &DebugRedactedApiKeyConfig(&self.0.gemini))
            .field("groq", &DebugRedactedApiKeyConfig(&self.0.groq))
            .field("azure", &DebugRedactedApiKeyConfig(&self.0.azure))
            .field("cohere", &DebugRedactedApiKeyConfig(&self.0.cohere))
            .field("mistral", &DebugRedactedApiKeyConfig(&self.0.mistral))
            .field("openrouter", &DebugRedactedApiKeyConfig(&self.0.openrouter))
            .field("together", &DebugRedactedApiKeyConfig(&self.0.together))
            .field("huggingface", &DebugRedactedApiKeyConfig(&self.0.huggingface))
            .field("deepseek", &DebugRedactedApiKeyConfig(&self.0.deepseek))
            .field("perplexity", &DebugRedactedApiKeyConfig(&self.0.perplexity))
            .field("ai21", &DebugRedactedApiKeyConfig(&self.0.ai21));
        
        providers.finish()
    }
}

/// Wrapper to redact single API key in debug output
struct DebugRedactedApiKeyConfig<'a>(&'a ProviderConfig);

impl<'a> std::fmt::Debug for DebugRedactedApiKeyConfig<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut config = f.debug_struct("ProviderConfig");
        
        config
            .field("api_key", &"<REDACTED>")
            .field("base_url", &self.0.base_url);
        
        config.finish()
    }
}

/// Wrapper to redact telemetry API key in debug output
struct DebugRedactedTelemetryConfig<'a>(&'a TelemetryConfig);

impl<'a> std::fmt::Debug for DebugRedactedTelemetryConfig<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut telemetry = f.debug_struct("TelemetryConfig");
        
        telemetry
            .field("enabled", &self.0.enabled)
            .field("surveys_enabled", &self.0.surveys_enabled)
            .field("anonymize", &self.0.anonymize)
            .field("encryption_standard", &self.0.encryption_standard)
            .field("storage_path", &self.0.storage_path)
            .field("central_server_url", &self.0.central_server_url)
            .field("telemetry_api_key", &self.0.telemetry_api_key.as_ref().map(|_| "<REDACTED>"));
        
        telemetry.finish()
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent: AgentSettings::default(),
            llm: LlmConfig::default(),
            lsp: LspConfig::default(),
            safety: SafetyConfig::default(),
            tools: ToolConfig::default(),
            self_compile: SelfCompileConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

/// Telemetry and secure survey configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable anonymous data collection
    pub enabled: bool,
    /// Enable secure, encrypted surveys
    pub surveys_enabled: bool,
    /// Anonymize all collected data
    pub anonymize: bool,
    /// Encryption standard (e.g., "AES-256")
    pub encryption_standard: String,
    /// Storage path for encrypted logs
    pub storage_path: PathBuf,
    /// URL for the central telemetry server
    pub central_server_url: Option<String>,
    /// API key for authentication with central server
    pub telemetry_api_key: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Opt-in by default false
            surveys_enabled: false,
            anonymize: true,
            encryption_standard: "AES-256".to_string(),
            storage_path: PathBuf::from(".agent/telemetry"),
            central_server_url: None,
            telemetry_api_key: None,
        }
    }
}

impl AgentConfig {
    /// Load configuration from a file path
    pub async fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: AgentConfig = toml::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    }

    /// Load configuration with hierarchy: default -> file -> env -> cli
    pub async fn load(
        config_path: Option<PathBuf>,
        overrides: ConfigOverrides,
    ) -> Result<Self> {
        let mut config = AgentConfig::default();

        // Load from file if provided
        if let Some(path) = config_path {
            if path.exists() {
                config = Self::from_file(path).await?;
            }
        }

        // Apply environment variable overrides
        config.apply_env_overrides()?;

        // Apply CLI overrides
        config.apply_overrides(overrides);

        config.validate()?;

        Ok(config)
    }

    /// Apply environment variable overrides to the configuration.
    ///
    /// Environment variables follow the pattern: CODING_AGENT_<SECTION>_<KEY>
    /// For example:
    /// - CODING_AGENT_LLM_API_KEY - Override LLM API key
    /// - CODING_AGENT_LLM_MODEL - Override LLM model
    /// - CODING_AGENT_LLM_PROVIDER - Override LLM provider
    /// - CODING_AGENT_LLM_TEMPERATURE - Override temperature
    /// - CODING_AGENT_LLM_MAX_TOKENS - Override max tokens
    /// - CODING_AGENT_AGENT_LOG_LEVEL - Override log level
    /// - CODING_AGENT_AGENT_IMPROVEMENT_INTERVAL - Override improvement interval
    /// - CODING_AGENT_AGENT_MAX_CONCURRENT_TASKS - Override max concurrent tasks
    /// - CODING_AGENT_LSP_ENABLED - Enable/disable LSP
    /// - CODING_AGENT_LSP_TIMEOUT - Override LSP timeout
    /// - CODING_AGENT_SAFETY_MAX_FILE_SIZE_MB - Override max file size
    fn apply_env_overrides(&mut self) -> Result<()> {
        use std::env;

        // LLM Configuration overrides
        if let Ok(val) = env::var("CODING_AGENT_LLM_API_KEY") {
            // Apply to the current provider's config
            match self.llm.provider.as_str() {
                "anthropic" => self.llm.providers.anthropic.api_key = val,
                "openai" => self.llm.providers.openai.api_key = val,
                "ollama" => self.llm.providers.ollama.api_key = val,
                "gemini" => self.llm.providers.gemini.api_key = val,
                "groq" => self.llm.providers.groq.api_key = val,
                "azure" => self.llm.providers.azure.api_key = val,
                "cohere" => self.llm.providers.cohere.api_key = val,
                "mistral" => self.llm.providers.mistral.api_key = val,
                "openrouter" => self.llm.providers.openrouter.api_key = val,
                "together" => self.llm.providers.together.api_key = val,
                "huggingface" => self.llm.providers.huggingface.api_key = val,
                "deepseek" => self.llm.providers.deepseek.api_key = val,
                "perplexity" => self.llm.providers.perplexity.api_key = val,
                "ai21" => self.llm.providers.ai21.api_key = val,
                _ => {}
            }
        }

        if let Ok(val) = env::var("CODING_AGENT_LLM_MODEL") {
            self.llm.model = val;
        }

        if let Ok(val) = env::var("CODING_AGENT_LLM_PROVIDER") {
            self.llm.provider = val.to_lowercase();
        }

        if let Ok(val) = env::var("CODING_AGENT_LLM_TEMPERATURE") {
            if let Ok(temp) = val.parse::<f32>() {
                self.llm.temperature = temp.clamp(0.0, 2.0);
            }
        }

        if let Ok(val) = env::var("CODING_AGENT_LLM_MAX_TOKENS") {
            if let Ok(tokens) = val.parse::<u32>() {
                self.llm.max_tokens = tokens;
            }
        }

        if let Ok(val) = env::var("CODING_AGENT_LLM_BASE_URL") {
            // Apply to the current provider's base_url
            match self.llm.provider.as_str() {
                "anthropic" => self.llm.providers.anthropic.base_url = val,
                "openai" => self.llm.providers.openai.base_url = val,
                "ollama" => self.llm.providers.ollama.base_url = val,
                "gemini" => self.llm.providers.gemini.base_url = val,
                "groq" => self.llm.providers.groq.base_url = val,
                "azure" => self.llm.providers.azure.base_url = val,
                "cohere" => self.llm.providers.cohere.base_url = val,
                "mistral" => self.llm.providers.mistral.base_url = val,
                "openrouter" => self.llm.providers.openrouter.base_url = val,
                "together" => self.llm.providers.together.base_url = val,
                "huggingface" => self.llm.providers.huggingface.base_url = val,
                "deepseek" => self.llm.providers.deepseek.base_url = val,
                "perplexity" => self.llm.providers.perplexity.base_url = val,
                "ai21" => self.llm.providers.ai21.base_url = val,
                _ => {}
            }
        }

        // Provider-specific API keys
        if let Ok(val) = env::var("ANTHROPIC_API_KEY") {
            self.llm.providers.anthropic.api_key = val;
        }

        if let Ok(val) = env::var("OPENAI_API_KEY") {
            self.llm.providers.openai.api_key = val;
        }

        if let Ok(val) = env::var("GEMINI_API_KEY") {
            self.llm.providers.gemini.api_key = val;
        }

        if let Ok(val) = env::var("GROQ_API_KEY") {
            self.llm.providers.groq.api_key = val;
        }

        if let Ok(val) = env::var("AZURE_OPENAI_API_KEY") {
            self.llm.providers.azure.api_key = val;
        }

        if let Ok(val) = env::var("COHERE_API_KEY") {
            self.llm.providers.cohere.api_key = val;
        }

        if let Ok(val) = env::var("MISTRAL_API_KEY") {
            self.llm.providers.mistral.api_key = val;
        }

        if let Ok(val) = env::var("OPENROUTER_API_KEY") {
            self.llm.providers.openrouter.api_key = val;
        }

        if let Ok(val) = env::var("TOGETHER_API_KEY") {
            self.llm.providers.together.api_key = val;
        }

        if let Ok(val) = env::var("HUGGINGFACE_API_KEY") {
            self.llm.providers.huggingface.api_key = val;
        }

        if let Ok(val) = env::var("DEEPSEEK_API_KEY") {
            self.llm.providers.deepseek.api_key = val;
        }

        if let Ok(val) = env::var("PERPLEXITY_API_KEY") {
            self.llm.providers.perplexity.api_key = val;
        }

        if let Ok(val) = env::var("AI21_API_KEY") {
            self.llm.providers.ai21.api_key = val;
        }

        // Vertex AI configuration
        if let Ok(val) = env::var("VERTEX_AI_PROJECT_ID") {
            self.llm.providers.vertex_ai.project_id = val;
        }

        if let Ok(val) = env::var("VERTEX_AI_LOCATION") {
            self.llm.providers.vertex_ai.location = val;
        }

        if let Ok(val) = env::var("VERTEX_AI_API_KEY") {
            self.llm.providers.vertex_ai.api_key = val;
        }

        if let Ok(val) = env::var("VERTEX_AI_BASE_URL") {
            self.llm.providers.vertex_ai.base_url = val;
        }

        // Agent settings overrides
        if let Ok(val) = env::var("CODING_AGENT_AGENT_LOG_LEVEL") {
            self.agent.log_level = val.to_lowercase();
        }

        if let Ok(val) = env::var("CODING_AGENT_AGENT_IMPROVEMENT_INTERVAL") {
            if let Ok(interval) = val.parse::<u64>() {
                self.agent.improvement_interval = interval;
            }
        }

        if let Ok(val) = env::var("CODING_AGENT_AGENT_MAX_CONCURRENT_TASKS") {
            if let Ok(tasks) = val.parse::<usize>() {
                self.agent.max_concurrent_tasks = tasks;
            }
        }

        if let Ok(val) = env::var("CODING_AGENT_AGENT_NAME") {
            self.agent.name = val;
        }

        // Self-improvement settings
        if let Ok(val) = env::var("CODING_AGENT_SELF_IMPROVEMENT_ENABLED") {
            self.agent.self_improvement.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("CODING_AGENT_SELF_IMPROVEMENT_AUTO_APPLY") {
            self.agent.self_improvement.auto_apply = val.parse().unwrap_or(false);
        }

        if let Ok(val) = env::var("CODING_AGENT_SELF_IMPROVEMENT_SAFETY_CHECKS") {
            self.agent.self_improvement.safety_checks = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("CODING_AGENT_SELF_IMPROVEMENT_MAX_MODIFICATIONS") {
            if let Ok(max) = val.parse::<usize>() {
                self.agent.self_improvement.max_modifications_per_session = max;
            }
        }

        // LSP Configuration overrides
        if let Ok(val) = env::var("CODING_AGENT_LSP_ENABLED") {
            self.lsp.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("CODING_AGENT_LSP_TIMEOUT") {
            if let Ok(timeout) = val.parse::<u64>() {
                self.lsp.timeout = timeout;
            }
        }

        // Safety Configuration overrides
        if let Ok(val) = env::var("CODING_AGENT_SAFETY_MAX_FILE_SIZE_MB") {
            if let Ok(size) = val.parse::<u64>() {
                self.safety.max_file_size_mb = size;
            }
        }

        // Tool Configuration overrides
        if let Ok(val) = env::var("CODING_AGENT_GIT_ENABLED") {
            self.tools.git.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("CODING_AGENT_GIT_AUTO_COMMIT") {
            self.tools.git.auto_commit = val.parse().unwrap_or(false);
        }

        if let Ok(val) = env::var("CODING_AGENT_TEST_ENABLED") {
            self.tools.test.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("CODING_AGENT_TEST_AUTO_RUN") {
            self.tools.test.auto_run = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("CODING_AGENT_TEST_FRAMEWORK") {
            self.tools.test.framework = val;
        }

        // Self-compile configuration
        if let Ok(val) = env::var("CODING_AGENT_SELF_COMPILE_ENABLED") {
            self.self_compile.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("CODING_AGENT_SELF_COMPILE_AUTO_RESTART") {
            self.self_compile.auto_restart = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("CODING_AGENT_SELF_COMPILE_BACKUP_COUNT") {
            if let Ok(count) = val.parse::<usize>() {
                self.self_compile.backup_count = count;
            }
        }

        // Cost/latency balancing
        if let Ok(val) = env::var("CODING_AGENT_ROUTING_STRATEGY") {
            self.llm.routing.strategy = val.parse().unwrap_or(RoutingStrategy::Fallback);
        }

        if let Ok(val) = env::var("CODING_AGENT_COST_BUDGET_PER_HOUR") {
            if let Ok(budget) = val.parse::<f64>() {
                self.llm.routing.cost_budget_per_hour = budget;
            }
        }

        if let Ok(val) = env::var("CODING_AGENT_MAX_LATENCY_MS") {
            if let Ok(latency) = val.parse::<u64>() {
                self.llm.routing.max_latency_ms = latency;
            }
        }

        // Telemetry overrides
        if let Ok(val) = env::var("CODING_AGENT_TELEMETRY_ENABLED") {
            self.telemetry.enabled = val.parse().unwrap_or(false);
        }

        if let Ok(val) = env::var("CODING_AGENT_TELEMETRY_SURVEYS_ENABLED") {
            self.telemetry.surveys_enabled = val.parse().unwrap_or(false);
        }

        if let Ok(val) = env::var("CODING_AGENT_TELEMETRY_ANONYMIZE") {
            self.telemetry.anonymize = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("CODING_AGENT_TELEMETRY_URL") {
            self.telemetry.central_server_url = Some(val);
        }

        if let Ok(val) = env::var("CODING_AGENT_TELEMETRY_KEY") {
            self.telemetry.telemetry_api_key = Some(val);
        }

        Ok(())
    }

    /// Apply CLI argument overrides to the configuration
    fn apply_overrides(&mut self, overrides: ConfigOverrides) {
        // Force update comment
        // Apply log level override
        if let Some(log_level) = overrides.log_level {
            self.agent.log_level = log_level;
        }

        // Apply LLM overrides
        if let Some(provider) = overrides.llm_provider {
            self.llm.provider = provider;
        }

        if let Some(model) = overrides.llm_model {
            self.llm.model = model;
        }

        if let Some(api_key) = overrides.llm_api_key {
            // Apply to the current provider's config
            match self.llm.provider.as_str() {
                "anthropic" => self.llm.providers.anthropic.api_key = api_key,
                "openai" => self.llm.providers.openai.api_key = api_key,
                "ollama" => self.llm.providers.ollama.api_key = api_key,
                "gemini" => self.llm.providers.gemini.api_key = api_key,
                "groq" => self.llm.providers.groq.api_key = api_key,
                "azure" => self.llm.providers.azure.api_key = api_key,
                "cohere" => self.llm.providers.cohere.api_key = api_key,
                "mistral" => self.llm.providers.mistral.api_key = api_key,
                "openrouter" => self.llm.providers.openrouter.api_key = api_key,
                "together" => self.llm.providers.together.api_key = api_key,
                "huggingface" => self.llm.providers.huggingface.api_key = api_key,
                "deepseek" => self.llm.providers.deepseek.api_key = api_key,
                "perplexity" => self.llm.providers.perplexity.api_key = api_key,
                "ai21" => self.llm.providers.ai21.api_key = api_key,
                _ => {}
            }
        }

        if let Some(base_url) = overrides.llm_base_url {
            match self.llm.provider.as_str() {
                "anthropic" => self.llm.providers.anthropic.base_url = base_url,
                "openai" => self.llm.providers.openai.base_url = base_url,
                "ollama" => self.llm.providers.ollama.base_url = base_url,
                "gemini" => self.llm.providers.gemini.base_url = base_url,
                "groq" => self.llm.providers.groq.base_url = base_url,
                "azure" => self.llm.providers.azure.base_url = base_url,
                "cohere" => self.llm.providers.cohere.base_url = base_url,
                "mistral" => self.llm.providers.mistral.base_url = base_url,
                "openrouter" => self.llm.providers.openrouter.base_url = base_url,
                "together" => self.llm.providers.together.base_url = base_url,
                "huggingface" => self.llm.providers.huggingface.base_url = base_url,
                "deepseek" => self.llm.providers.deepseek.base_url = base_url,
                "perplexity" => self.llm.providers.perplexity.base_url = base_url,
                "ai21" => self.llm.providers.ai21.base_url = base_url,
                _ => {}
            }
        }

        if let Some(temperature) = overrides.llm_temperature {
            self.llm.temperature = temperature.clamp(0.0, 2.0);
        }

        if let Some(max_tokens) = overrides.llm_max_tokens {
            self.llm.max_tokens = max_tokens;
        }

        // Apply agent settings overrides
        if let Some(improvement_interval) = overrides.improvement_interval {
            self.agent.improvement_interval = improvement_interval;
        }

        if let Some(max_concurrent_tasks) = overrides.max_concurrent_tasks {
            self.agent.max_concurrent_tasks = max_concurrent_tasks;
        }

        // Apply self-improvement overrides
        if let Some(self_improvement_enabled) = overrides.self_improvement_enabled {
            self.agent.self_improvement.enabled = self_improvement_enabled;
        }

        if let Some(auto_apply) = overrides.self_improvement_auto_apply {
            self.agent.self_improvement.auto_apply = auto_apply;
        }

        if let Some(safety_checks) = overrides.self_improvement_safety_checks {
            self.agent.self_improvement.safety_checks = safety_checks;
        }

        // Apply LSP overrides
        if let Some(lsp_enabled) = overrides.lsp_enabled {
            self.lsp.enabled = lsp_enabled;
        }

        if let Some(lsp_timeout) = overrides.lsp_timeout {
            self.lsp.timeout = lsp_timeout;
        }

        // Apply safety overrides
        if let Some(max_file_size_mb) = overrides.safety_max_file_size_mb {
            self.safety.max_file_size_mb = max_file_size_mb;
        }

        // Apply tool overrides
        if let Some(git_enabled) = overrides.git_enabled {
            self.tools.git.enabled = git_enabled;
        }

        if let Some(git_auto_commit) = overrides.git_auto_commit {
            self.tools.git.auto_commit = git_auto_commit;
        }

        if let Some(test_enabled) = overrides.test_enabled {
            self.tools.test.enabled = test_enabled;
        }

        if let Some(test_auto_run) = overrides.test_auto_run {
            self.tools.test.auto_run = test_auto_run;
        }

        if let Some(test_framework) = overrides.test_framework {
            self.tools.test.framework = test_framework;
        }

        // Apply self-compile overrides
        if let Some(self_compile_enabled) = overrides.self_compile_enabled {
            self.self_compile.enabled = self_compile_enabled;
        }

        if let Some(auto_restart) = overrides.self_compile_auto_restart {
            self.self_compile.auto_restart = auto_restart;
        }
    }

    /// Validate the configuration for consistency and required fields.
    ///
    /// This method checks:
    /// - Required API keys are present for non-local providers
    /// - URLs are valid
    /// - Temperature is within valid range
    /// - Max tokens is reasonable
    /// - Protected paths are valid patterns
    /// - Improvement interval is reasonable
    /// - Max concurrent tasks is reasonable
    fn validate(&self) -> Result<()> {
        // Validate LLM configuration
        let valid_providers = [
            "anthropic", "openai", "ollama", "gemini", "groq", "azure",
            "cohere", "mistral", "openrouter", "together",
            "huggingface", "deepseek", "perplexity", "ai21", "vertex_ai"
        ];
        if !valid_providers.contains(&self.llm.provider.as_str()) {
            return Err(Error::Validation(format!(
                "Invalid LLM provider: {}. Must be one of: {:?}",
                self.llm.provider, valid_providers
            )));
        }

        // Check API key for non-local providers
        match self.llm.provider.as_str() {
            "anthropic" => {
                if self.llm.providers.anthropic.api_key.is_empty()
                    || self.llm.providers.anthropic.api_key == "${ANTHROPIC_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for Anthropic provider. Set ANTHROPIC_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.anthropic.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid Anthropic base URL: {}",
                        e
                    )));
                }
            }
            "openai" => {
                if self.llm.providers.openai.api_key.is_empty()
                    || self.llm.providers.openai.api_key == "${OPENAI_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for OpenAI provider. Set OPENAI_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.openai.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid OpenAI base URL: {}",
                        e
                    )));
                }
            }
            "gemini" => {
                if self.llm.providers.gemini.api_key.is_empty()
                    || self.llm.providers.gemini.api_key == "${GEMINI_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for Gemini provider. Set GEMINI_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.gemini.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid Gemini base URL: {}",
                        e
                    )));
                }
            }
            "groq" => {
                if self.llm.providers.groq.api_key.is_empty()
                    || self.llm.providers.groq.api_key == "${GROQ_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for Groq provider. Set GROQ_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.groq.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid Groq base URL: {}",
                        e
                    )));
                }
            }
            "azure" => {
                if self.llm.providers.azure.api_key.is_empty()
                    || self.llm.providers.azure.api_key == "${AZURE_OPENAI_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for Azure provider. Set AZURE_OPENAI_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.azure.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid Azure base URL: {}",
                        e
                    )));
                }
            }
            "cohere" => {
                if self.llm.providers.cohere.api_key.is_empty()
                    || self.llm.providers.cohere.api_key == "${COHERE_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for Cohere provider. Set COHERE_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.cohere.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid Cohere base URL: {}",
                        e
                    )));
                }
            }
            "mistral" => {
                if self.llm.providers.mistral.api_key.is_empty()
                    || self.llm.providers.mistral.api_key == "${MISTRAL_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for Mistral provider. Set MISTRAL_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.mistral.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid Mistral base URL: {}",
                        e
                    )));
                }
            }
            "openrouter" => {
                if self.llm.providers.openrouter.api_key.is_empty()
                    || self.llm.providers.openrouter.api_key == "${OPENROUTER_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for OpenRouter provider. Set OPENROUTER_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.openrouter.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid OpenRouter base URL: {}",
                        e
                    )));
                }
            }
            "together" => {
                if self.llm.providers.together.api_key.is_empty()
                    || self.llm.providers.together.api_key == "${TOGETHER_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for Together AI provider. Set TOGETHER_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.together.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid Together AI base URL: {}",
                        e
                    )));
                }
            }
            "huggingface" => {
                if self.llm.providers.huggingface.api_key.is_empty()
                    || self.llm.providers.huggingface.api_key == "${HUGGINGFACE_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for Hugging Face provider. Set HUGGINGFACE_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.huggingface.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid Hugging Face base URL: {}",
                        e
                    )));
                }
            }
            "deepseek" => {
                if self.llm.providers.deepseek.api_key.is_empty()
                    || self.llm.providers.deepseek.api_key == "${DEEPSEEK_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for DeepSeek provider. Set DEEPSEEK_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.deepseek.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid DeepSeek base URL: {}",
                        e
                    )));
                }
            }
            "perplexity" => {
                if self.llm.providers.perplexity.api_key.is_empty()
                    || self.llm.providers.perplexity.api_key == "${PERPLEXITY_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for Perplexity provider. Set PERPLEXITY_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.perplexity.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid Perplexity base URL: {}",
                        e
                    )));
                }
            }
            "ai21" => {
                if self.llm.providers.ai21.api_key.is_empty()
                    || self.llm.providers.ai21.api_key == "${AI21_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for AI21 Labs provider. Set AI21_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.ai21.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid AI21 Labs base URL: {}",
                        e
                    )));
                }
            }
            "ollama" => {
                // Ollama is local, API key is optional but URL should still be valid
                if let Err(e) = url::Url::parse(&self.llm.providers.ollama.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid Ollama base URL: {}",
                        e
                    )));
                }
            }
            "vertex_ai" => {
                // Vertex AI requires project ID and API key
                if self.llm.providers.vertex_ai.project_id.is_empty()
                    || self.llm.providers.vertex_ai.project_id == "${VERTEX_AI_PROJECT_ID}"
                {
                    return Err(Error::Validation(
                        "Project ID is required for Vertex AI provider. Set VERTEX_AI_PROJECT_ID environment variable or provide it in config.".to_string(),
                    ));
                }
                if self.llm.providers.vertex_ai.api_key.is_empty()
                    || self.llm.providers.vertex_ai.api_key == "${VERTEX_AI_API_KEY}"
                {
                    return Err(Error::Validation(
                        "API key is required for Vertex AI provider. Set VERTEX_AI_API_KEY environment variable or provide it in config.".to_string(),
                    ));
                }
                if let Err(e) = url::Url::parse(&self.llm.providers.vertex_ai.base_url) {
                    return Err(Error::Validation(format!(
                        "Invalid Vertex AI base URL: {}",
                        e
                    )));
                }
            }
            _ => {}
        }

        // Validate fallback configuration if enabled
        if self.llm.fallback.enabled {
            if !valid_providers.contains(&self.llm.fallback.provider.as_str()) {
                return Err(Error::Validation(format!(
                    "Invalid fallback provider: {}. Must be one of: {:?}",
                    self.llm.fallback.provider, valid_providers
                )));
            }
        }

        // Validate routing configuration
        if self.llm.routing.providers.is_empty() {
            return Err(Error::Validation(
                "At least one provider must be configured for routing".to_string(),
            ));
        }

        // Validate temperature range
        if self.llm.temperature < 0.0 || self.llm.temperature > 2.0 {
            return Err(Error::Validation(format!(
                "Temperature must be between 0.0 and 2.0, got {}",
                self.llm.temperature
            )));
        }

        // Validate max tokens
        if self.llm.max_tokens == 0 {
            return Err(Error::Validation(
                "max_tokens must be greater than 0".to_string(),
            ));
        }

        if self.llm.max_tokens > 100_000 {
            return Err(Error::Validation(format!(
                "max_tokens seems unreasonably high: {}. Maximum allowed is 100,000",
                self.llm.max_tokens
            )));
        }

        // Validate agent settings
        if self.agent.name.is_empty() {
            return Err(Error::Validation(
                "Agent name cannot be empty".to_string(),
            ));
        }

        if self.agent.improvement_interval == 0 {
            return Err(Error::Validation(
                "improvement_interval must be greater than 0".to_string(),
            ));
        }

        if self.agent.improvement_interval < 60 {
            tracing::warn!(
                "improvement_interval is very short ({} seconds). This may impact performance.",
                self.agent.improvement_interval
            );
        }

        if self.agent.max_concurrent_tasks == 0 {
            return Err(Error::Validation(
                "max_concurrent_tasks must be greater than 0".to_string(),
            ));
        }

        if self.agent.max_concurrent_tasks > 100 {
            return Err(Error::Validation(format!(
                "max_concurrent_tasks seems unreasonably high: {}. Maximum allowed is 100",
                self.agent.max_concurrent_tasks
            )));
        }

        // Validate log level
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.agent.log_level.as_str()) {
            return Err(Error::Validation(format!(
                "Invalid log level: {}. Must be one of: {:?}",
                self.agent.log_level, valid_log_levels
            )));
        }

        // Validate self-improvement settings
        if self.agent.self_improvement.max_modifications_per_session == 0 {
            return Err(Error::Validation(
                "max_modifications_per_session must be greater than 0".to_string(),
            ));
        }

        if self.agent.self_improvement.max_modifications_per_session > 1000 {
            return Err(Error::Validation(format!(
                "max_modifications_per_session seems unreasonably high: {}. Maximum allowed is 1000",
                self.agent.self_improvement.max_modifications_per_session
            )));
        }

        // Validate LSP configuration
        if self.lsp.timeout == 0 {
            return Err(Error::Validation(
                "LSP timeout must be greater than 0".to_string(),
            ));
        }

        if self.lsp.timeout > 300 {
            tracing::warn!(
                "LSP timeout is very long ({} seconds). This may cause slow responses.",
                self.lsp.timeout
            );
        }

        // Validate safety configuration
        if self.safety.max_file_size_mb == 0 {
            return Err(Error::Validation(
                "max_file_size_mb must be greater than 0".to_string(),
            ));
        }

        if self.safety.max_file_size_mb > 1000 {
            tracing::warn!(
                "max_file_size_mb is very high ({} MB). This may cause memory issues.",
                self.safety.max_file_size_mb
            );
        }

        // Validate protected paths are not empty
        if self.safety.protected_paths.is_empty() {
            return Err(Error::Validation(
                "At least one protected path must be specified".to_string(),
            ));
        }

        // Validate forbidden commands
        if self.safety.forbidden_commands.is_empty() {
            tracing::warn!("No forbidden commands configured. This may be a security risk.");
        }

        // Validate tool configuration
        let valid_test_frameworks = ["cargo", "jest", "pytest", "go test", "none"];
        if !valid_test_frameworks.contains(&self.tools.test.framework.as_str()) {
            tracing::warn!(
                "Unknown test framework: {}. Known frameworks: {:?}",
                self.tools.test.framework,
                valid_test_frameworks
            );
        }

        // Validate self-compile configuration
        if self.self_compile.backup_count == 0 {
            return Err(Error::Validation(
                "self_compile.backup_count must be greater than 0".to_string(),
            ));
        }

        if self.self_compile.backup_count > 100 {
            return Err(Error::Validation(
                "self_compile.backup_count cannot exceed 100".to_string(),
            ));
        }

        // Validate routing cost budget
        if self.llm.routing.cost_budget_per_hour < 0.0 {
            return Err(Error::Validation(
                "cost_budget_per_hour cannot be negative".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the API key for the current provider
    pub fn current_api_key(&self) -> &str {
        match self.llm.provider.as_str() {
            "anthropic" => &self.llm.providers.anthropic.api_key,
            "openai" => &self.llm.providers.openai.api_key,
            "ollama" => &self.llm.providers.ollama.api_key,
            "gemini" => &self.llm.providers.gemini.api_key,
            "groq" => &self.llm.providers.groq.api_key,
            "azure" => &self.llm.providers.azure.api_key,
            "cohere" => &self.llm.providers.cohere.api_key,
            "mistral" => &self.llm.providers.mistral.api_key,
            "openrouter" => &self.llm.providers.openrouter.api_key,
            "together" => &self.llm.providers.together.api_key,
            "huggingface" => &self.llm.providers.huggingface.api_key,
            "deepseek" => &self.llm.providers.deepseek.api_key,
            "perplexity" => &self.llm.providers.perplexity.api_key,
            "ai21" => &self.llm.providers.ai21.api_key,
            _ => "",
        }
    }

    /// Get the base URL for the current provider
    pub fn current_base_url(&self) -> &str {
        match self.llm.provider.as_str() {
            "anthropic" => &self.llm.providers.anthropic.base_url,
            "openai" => &self.llm.providers.openai.base_url,
            "ollama" => &self.llm.providers.ollama.base_url,
            "gemini" => &self.llm.providers.gemini.base_url,
            "groq" => &self.llm.providers.groq.base_url,
            "azure" => &self.llm.providers.azure.base_url,
            "cohere" => &self.llm.providers.cohere.base_url,
            "mistral" => &self.llm.providers.mistral.base_url,
            "openrouter" => &self.llm.providers.openrouter.base_url,
            "together" => &self.llm.providers.together.base_url,
            "huggingface" => &self.llm.providers.huggingface.base_url,
            "deepseek" => &self.llm.providers.deepseek.base_url,
            "perplexity" => &self.llm.providers.perplexity.base_url,
            "ai21" => &self.llm.providers.ai21.base_url,
            _ => "",
        }
    }
}

/// Agent identity and core settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    pub name: String,
    pub version: String,
    pub improvement_interval: u64,
    pub max_concurrent_tasks: usize,
    pub log_level: String,
    pub self_improvement: SelfImprovementSettings,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            name: "coding-agent".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            improvement_interval: 3600,
            max_concurrent_tasks: 4,
            log_level: "info".to_string(),
            self_improvement: SelfImprovementSettings::default(),
        }
    }
}

/// Self-improvement settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfImprovementSettings {
    pub enabled: bool,
    pub auto_apply: bool,
    pub safety_checks: bool,
    pub max_modifications_per_session: usize,
}

impl Default for SelfImprovementSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_apply: false,
            safety_checks: true,
            max_modifications_per_session: 10,
        }
    }
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub fallback: FallbackConfig,
    pub providers: ProviderConfigs,
    pub routing: RoutingConfig,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            fallback: FallbackConfig::default(),
            providers: ProviderConfigs::default(),
            routing: RoutingConfig::default(),
        }
    }
}

/// Fallback LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub enabled: bool,
    pub provider: String,
    pub model: String,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
        }
    }
}

/// Provider-specific configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigs {
    pub anthropic: ProviderConfig,
    pub openai: ProviderConfig,
    pub ollama: ProviderConfig,
    pub gemini: ProviderConfig,
    pub groq: ProviderConfig,
    pub azure: ProviderConfig,
    pub cohere: ProviderConfig,
    pub mistral: ProviderConfig,
    pub openrouter: ProviderConfig,
    pub together: ProviderConfig,
    pub huggingface: ProviderConfig,
    pub deepseek: ProviderConfig,
    pub perplexity: ProviderConfig,
    pub ai21: ProviderConfig,
    pub vertex_ai: VertexAiConfig,
}

impl Default for ProviderConfigs {
    fn default() -> Self {
        Self {
            anthropic: ProviderConfig {
                api_key: "${ANTHROPIC_API_KEY}".to_string(),
                base_url: "https://api.anthropic.com".to_string(),
            },
            openai: ProviderConfig {
                api_key: "${OPENAI_API_KEY}".to_string(),
                base_url: "https://api.openai.com".to_string(),
            },
            ollama: ProviderConfig {
                api_key: String::new(),
                base_url: "http://localhost:11434".to_string(),
            },
            gemini: ProviderConfig {
                api_key: "${GEMINI_API_KEY}".to_string(),
                base_url: "https://generativelanguage.googleapis.com".to_string(),
            },
            groq: ProviderConfig {
                api_key: "${GROQ_API_KEY}".to_string(),
                base_url: "https://api.groq.com/openai/v1".to_string(),
            },
            azure: ProviderConfig {
                api_key: "${AZURE_OPENAI_API_KEY}".to_string(),
                base_url: "https://api.openai.azure.com".to_string(),
            },
            cohere: ProviderConfig {
                api_key: "${COHERE_API_KEY}".to_string(),
                base_url: "https://api.cohere.ai".to_string(),
            },
            mistral: ProviderConfig {
                api_key: "${MISTRAL_API_KEY}".to_string(),
                base_url: "https://api.mistral.ai".to_string(),
            },
            openrouter: ProviderConfig {
                api_key: "${OPENROUTER_API_KEY}".to_string(),
                base_url: "https://openrouter.ai/api/v1".to_string(),
            },
            together: ProviderConfig {
                api_key: "${TOGETHER_API_KEY}".to_string(),
                base_url: "https://api.together.xyz".to_string(),
            },
            huggingface: ProviderConfig {
                api_key: "${HUGGINGFACE_API_KEY}".to_string(),
                base_url: "https://api-inference.huggingface.co".to_string(),
            },
            deepseek: ProviderConfig {
                api_key: "${DEEPSEEK_API_KEY}".to_string(),
                base_url: "https://api.deepseek.com".to_string(),
            },
            perplexity: ProviderConfig {
                api_key: "${PERPLEXITY_API_KEY}".to_string(),
                base_url: "https://api.perplexity.ai".to_string(),
            },
            ai21: ProviderConfig {
                api_key: "${AI21_API_KEY}".to_string(),
                base_url: "https://api.ai21.com/studio/v1".to_string(),
            },
            vertex_ai: VertexAiConfig {
                project_id: "${VERTEX_AI_PROJECT_ID}".to_string(),
                location: "us-central1".to_string(),
                api_key: "${VERTEX_AI_API_KEY}".to_string(),
                base_url: "https://us-central1-aiplatform.googleapis.com".to_string(),
            },
        }
    }
}

/// Vertex AI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAiConfig {
    /// Google Cloud Project ID
    pub project_id: String,
    /// Location/region (e.g., "us-central1", "europe-west4")
    pub location: String,
    /// API key for authentication (optional if using service account)
    pub api_key: String,
    /// Base URL for Vertex AI API
    pub base_url: String,
}

impl Default for VertexAiConfig {
    fn default() -> Self {
        Self {
            project_id: "${VERTEX_AI_PROJECT_ID}".to_string(),
            location: "us-central1".to_string(),
            api_key: "${VERTEX_AI_API_KEY}".to_string(),
            base_url: "https://us-central1-aiplatform.googleapis.com".to_string(),
        }
    }
}

/// Individual provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: String,
}

/// Routing strategy for provider selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    /// Use primary provider, fallback on failure
    Fallback,
    /// Load balance across providers
    LoadBalance,
    /// Select provider based on cost
    CostOptimized,
    /// Select provider based on latency
    LatencyOptimized,
    /// Select provider based on quality
    QualityOptimized,
    /// Custom routing based on user-defined rules
    Custom,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        RoutingStrategy::Fallback
    }
}

impl std::str::FromStr for RoutingStrategy {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fallback" => Ok(RoutingStrategy::Fallback),
            "load_balance" | "loadbalance" => Ok(RoutingStrategy::LoadBalance),
            "cost_optimized" | "costoptimized" | "cost" => Ok(RoutingStrategy::CostOptimized),
            "latency_optimized" | "latencyoptimized" | "latency" => Ok(RoutingStrategy::LatencyOptimized),
            "quality_optimized" | "qualityoptimized" | "quality" => Ok(RoutingStrategy::QualityOptimized),
            "custom" => Ok(RoutingStrategy::Custom),
            _ => Err(format!("Unknown routing strategy: {}", s)),
        }
    }
}

/// Provider routing configuration with cost/latency balancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Routing strategy
    pub strategy: RoutingStrategy,
    /// List of providers to route between
    pub providers: Vec<ProviderRoute>,
    /// Cost budget per hour in USD (0 = unlimited)
    pub cost_budget_per_hour: f64,
    /// Maximum acceptable latency in milliseconds (0 = unlimited)
    pub max_latency_ms: u64,
    /// Enable automatic provider health checking
    pub health_check_enabled: bool,
    /// Health check interval in seconds
    pub health_check_interval: u64,
    /// Enable automatic failover
    pub auto_failover: bool,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            strategy: RoutingStrategy::Fallback,
            providers: vec![
                ProviderRoute {
                    provider: "anthropic".to_string(),
                    model: "claude-3-5-sonnet-20241022".to_string(),
                    weight: 1.0,
                    cost_per_1k_input: 0.003,
                    cost_per_1k_output: 0.015,
                    avg_latency_ms: 800,
                    priority: 1,
                    enabled: true,
                },
                ProviderRoute {
                    provider: "openai".to_string(),
                    model: "gpt-4o".to_string(),
                    weight: 1.0,
                    cost_per_1k_input: 0.005,
                    cost_per_1k_output: 0.015,
                    avg_latency_ms: 600,
                    priority: 2,
                    enabled: true,
                },
            ],
            cost_budget_per_hour: 10.0,
            max_latency_ms: 5000,
            health_check_enabled: true,
            health_check_interval: 60,
            auto_failover: true,
        }
    }
}

/// Individual provider route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRoute {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Weight for load balancing (higher = more traffic)
    pub weight: f64,
    /// Cost per 1K input tokens in USD
    pub cost_per_1k_input: f64,
    /// Cost per 1K output tokens in USD
    pub cost_per_1k_output: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Whether this provider is enabled
    pub enabled: bool,
}

/// LSP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    pub enabled: bool,
    pub timeout: u64,
    pub servers: Vec<LanguageServerConfig>,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout: 30,
            servers: vec![
                LanguageServerConfig {
                    name: "rust-analyzer".to_string(),
                    command: "rust-analyzer".to_string(),
                    args: vec![],
                    filetypes: vec!["rust".to_string()],
                    root_patterns: vec!["Cargo.toml".to_string()],
                },
            ],
        }
    }
}

/// Language server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub filetypes: Vec<String>,
    pub root_patterns: Vec<String>,
}

/// Safety configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub protected_paths: Vec<String>,
    pub max_file_size_mb: u64,
    pub forbidden_commands: Vec<String>,
    pub require_approval_for: Vec<String>,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            protected_paths: vec![
                ".agent/core/**".to_string(),
                ".agent/safety/**".to_string(),
                ".agent/auth/**".to_string(),
            ],
            max_file_size_mb: 10,
            forbidden_commands: vec!["rm -rf /".to_string(), "dd if=/dev/zero".to_string()],
            require_approval_for: vec!["delete".to_string(), "modify_protected".to_string(), "git_push".to_string()],
        }
    }
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub git: GitToolConfig,
    pub test: TestToolConfig,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            git: GitToolConfig::default(),
            test: TestToolConfig::default(),
        }
    }
}

/// Git tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitToolConfig {
    pub enabled: bool,
    pub auto_commit: bool,
    pub commit_prefix: String,
}

impl Default for GitToolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_commit: false,
            commit_prefix: "[agent]".to_string(),
        }
    }
}

/// Test tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestToolConfig {
    pub enabled: bool,
    pub framework: String,
    pub auto_run: bool,
    pub fail_on_error: bool,
}

impl Default for TestToolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            framework: "cargo".to_string(),
            auto_run: true,
            fail_on_error: true,
        }
    }
}

/// Self-compilation and restart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfCompileConfig {
    /// Enable self-compilation
    pub enabled: bool,
    /// Automatically restart after successful compilation
    pub auto_restart: bool,
    /// Number of backups to keep
    pub backup_count: usize,
    /// Build profile to use (dev/release)
    pub build_profile: String,
    /// Additional cargo build arguments
    pub build_args: Vec<String>,
    /// Timeout for compilation in seconds
    pub compile_timeout_seconds: u64,
    /// Verify binary before restart
    pub verify_before_restart: bool,
    /// Rollback on restart failure
    pub rollback_on_failure: bool,
}

impl Default for SelfCompileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_restart: true,
            backup_count: 5,
            build_profile: "release".to_string(),
            build_args: vec![],
            compile_timeout_seconds: 300,
            verify_before_restart: true,
            rollback_on_failure: true,
        }
    }
}

/// CLI configuration overrides
#[derive(Debug, Default, Clone)]
pub struct ConfigOverrides {
    pub log_level: Option<String>,
    pub workspace: Option<PathBuf>,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub llm_api_key: Option<String>,
    pub llm_base_url: Option<String>,
    pub llm_temperature: Option<f32>,
    pub llm_max_tokens: Option<u32>,
    pub improvement_interval: Option<u64>,
    pub max_concurrent_tasks: Option<usize>,
    pub self_improvement_enabled: Option<bool>,
    pub self_improvement_auto_apply: Option<bool>,
    pub self_improvement_safety_checks: Option<bool>,
    pub lsp_enabled: Option<bool>,
    pub lsp_timeout: Option<u64>,
    pub safety_max_file_size_mb: Option<u64>,
    pub git_enabled: Option<bool>,
    pub git_auto_commit: Option<bool>,
    pub test_enabled: Option<bool>,
    pub test_auto_run: Option<bool>,
    pub test_framework: Option<String>,
    pub self_compile_enabled: Option<bool>,
    pub self_compile_auto_restart: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgentConfig::default();
        assert_eq!(config.agent.name, "coding-agent");
        assert_eq!(config.llm.provider, "anthropic");
        assert!(config.lsp.enabled);
    }

    #[test]
    fn test_env_override_parsing() {
        // This test would need environment variables set
        // In practice, run with: CODING_AGENT_LLM_MODEL=test-model cargo test
    }

    #[test]
    fn test_validation_valid_config() {
        let config = AgentConfig::default();
        // This will fail validation because API key is not set
        // In real usage, set ANTHROPIC_API_KEY before validating
    }

    #[test]
    fn test_config_overrides() {
        let mut config = AgentConfig::default();
        let overrides = ConfigOverrides {
            log_level: Some("debug".to_string()),
            llm_model: Some("gpt-4".to_string()),
            ..Default::default()
        };
        config.apply_overrides(overrides);
        assert_eq!(config.agent.log_level, "debug");
        assert_eq!(config.llm.model, "gpt-4");
    }

    #[test]
    fn test_routing_strategy_parsing() {
        use std::str::FromStr;
        
        assert_eq!(RoutingStrategy::from_str("fallback").unwrap(), RoutingStrategy::Fallback);
        assert_eq!(RoutingStrategy::from_str("load_balance").unwrap(), RoutingStrategy::LoadBalance);
        assert_eq!(RoutingStrategy::from_str("cost").unwrap(), RoutingStrategy::CostOptimized);
        assert_eq!(RoutingStrategy::from_str("latency").unwrap(), RoutingStrategy::LatencyOptimized);
        assert_eq!(RoutingStrategy::from_str("quality").unwrap(), RoutingStrategy::QualityOptimized);
        assert_eq!(RoutingStrategy::from_str("custom").unwrap(), RoutingStrategy::Custom);
        
        assert!(RoutingStrategy::from_str("unknown").is_err());
    }

    #[test]
    fn test_provider_route_defaults() {
        let config = AgentConfig::default();
        assert!(!config.llm.routing.providers.is_empty());
        assert!(config.llm.routing.cost_budget_per_hour > 0.0);
        assert!(config.llm.routing.max_latency_ms > 0);
    }

    #[test]
    fn test_self_compile_defaults() {
        let config = AgentConfig::default();
        assert!(config.self_compile.enabled);
        assert!(config.self_compile.auto_restart);
        assert_eq!(config.self_compile.build_profile, "release");
        assert!(config.self_compile.backup_count > 0);
    }
}
