
use config::AgentConfig;
use std::fs;

fn main() {
    let config_content = fs::read_to_string("test_openrouter_config.yaml").unwrap();
    let config: AgentConfig = serde_yaml::from_str(&config_content).unwrap();
    
    println!("Successfully parsed configuration!");
    println!("LLM Provider: {:?}", config.llm.provider);
    println!("LLM Model: {:?}", config.llm.model);
    println!("OpenRouter API Key: {:?}", config.providers.openrouter.api_key);
    println!("OpenRouter Base URL: {:?}", config.providers.openrouter.base_url);
    println!("LSP Max Workers: {:?}", config.lsp.max_workers);
    println!("Safety Enabled: {:?}", config.safety.enabled);
    println!("Safety Strictness: {:?}", config.safety.strictness);
}
