// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AI Assistant module for narrative generation
//!
//! This module is feature-gated behind `ai-assistant` and provides
//! LLM-powered narrative generation for forensic reports.
//!
//! # Security
//!
//! - Ollama URLs are validated to prevent SSRF attacks
//! - Only localhost HTTP or HTTPS connections are allowed
//! - All AI interactions are logged for audit trail
//!
//! # Usage
//!
//! Enable the feature in Cargo.toml:
//! ```toml
//! [dependencies]
//! ffx-check = { features = ["ai-assistant"] }
//! ```
//!
//! Then use the AI assistant:
//! ```rust,ignore
//! use report::ai::{AiAssistant, AiProvider};
//!
//! let ai = AiAssistant::new(AiProvider::Ollama {
//!     model: "llama3.2".to_string(),
//!     base_url: None,
//! });
//!
//! let summary = ai.generate_narrative(
//!     &evidence_context,
//!     NarrativeType::ExecutiveSummary
//! ).await?;
//! ```

use super::error::{ReportError, ReportResult};
use super::NarrativeType;

/// Validate a URL for Ollama connections to prevent SSRF attacks.
///
/// # Security Rules
/// - HTTP is only allowed for localhost/127.0.0.1
/// - HTTPS is allowed for any host
/// - Other schemes are rejected
fn validate_ollama_url(url: &str) -> ReportResult<()> {
    // Parse the URL
    let parsed = url::Url::parse(url)
        .map_err(|e| ReportError::AiError(format!("Invalid Ollama URL: {}", e)))?;

    let scheme = parsed.scheme();
    let host = parsed.host_str().unwrap_or("");

    match scheme {
        "https" => {
            // HTTPS is always allowed (encrypted)
            tracing::info!(target: "ai_audit", "Ollama connection to HTTPS endpoint: {}", host);
            Ok(())
        }
        "http" => {
            // HTTP only allowed for localhost
            let is_localhost =
                host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "[::1]";

            if is_localhost {
                tracing::info!(target: "ai_audit", "Ollama connection to local endpoint: {}", url);
                Ok(())
            } else {
                tracing::warn!(target: "ai_audit", 
                    "SECURITY: Blocked HTTP connection to non-localhost Ollama: {}", url);
                Err(ReportError::AiError(format!(
                    "Security error: HTTP connections to Ollama are only allowed for localhost. \
                     Use HTTPS for remote servers or connect to http://localhost:11434. \
                     Attempted URL: {}",
                    url
                )))
            }
        }
        _ => {
            tracing::warn!(target: "ai_audit", "SECURITY: Blocked invalid URL scheme: {}", scheme);
            Err(ReportError::AiError(format!(
                "Invalid URL scheme '{}'. Only HTTP (localhost) and HTTPS are allowed.",
                scheme
            )))
        }
    }
}

/// Log AI interaction for forensic audit trail
fn log_ai_interaction(provider: &str, model: &str, narrative_type: &str, prompt_length: usize) {
    tracing::info!(
        target: "ai_audit",
        provider = provider,
        model = model,
        narrative_type = narrative_type,
        prompt_chars = prompt_length,
        timestamp = %chrono::Utc::now().to_rfc3339(),
        "AI narrative generation requested"
    );
}

/// Log AI response for forensic audit trail
fn log_ai_response(provider: &str, model: &str, success: bool, response_length: usize) {
    if success {
        tracing::info!(
            target: "ai_audit",
            provider = provider,
            model = model,
            response_chars = response_length,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            "AI narrative generation completed"
        );
    } else {
        tracing::warn!(
            target: "ai_audit",
            provider = provider,
            model = model,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            "AI narrative generation failed"
        );
    }
}

/// AI provider configuration
#[derive(Debug, Clone)]
pub enum AiProvider {
    /// Ollama (local LLM)
    Ollama {
        /// Model name (e.g., "llama3.2", "mistral")
        model: String,
        /// Base URL (default: http://localhost:11434)
        base_url: Option<String>,
    },
    /// OpenAI API
    OpenAi {
        /// Model name (e.g., "gpt-4", "gpt-3.5-turbo")
        model: String,
        /// API key (will use OPENAI_API_KEY env var if not provided)
        api_key: Option<String>,
    },
    /// Azure OpenAI
    AzureOpenAi {
        /// Deployment name
        deployment: String,
        /// Azure endpoint
        endpoint: String,
        /// API key
        api_key: Option<String>,
    },
}

/// AI Assistant for generating report narratives
pub struct AiAssistant {
    provider: AiProvider,
}

impl AiAssistant {
    /// Create a new AI Assistant with the specified provider
    pub fn new(provider: AiProvider) -> Self {
        Self { provider }
    }

    /// Create an AI Assistant using Ollama with default settings
    pub fn ollama(model: impl Into<String>) -> Self {
        Self::new(AiProvider::Ollama {
            model: model.into(),
            base_url: None,
        })
    }

    /// Create an AI Assistant using OpenAI
    pub fn openai(model: impl Into<String>) -> Self {
        Self::new(AiProvider::OpenAi {
            model: model.into(),
            api_key: None,
        })
    }

    /// Generate a narrative for the given context and type
    pub async fn generate_narrative(
        &self,
        context: &str,
        narrative_type: NarrativeType,
    ) -> ReportResult<String> {
        let prompt = self.build_prompt(context, narrative_type);

        match &self.provider {
            AiProvider::Ollama { model, base_url } => {
                self.generate_with_ollama(model, base_url.as_deref(), &prompt)
                    .await
            }
            AiProvider::OpenAi { model, api_key } => {
                self.generate_with_openai(model, api_key.as_deref(), &prompt)
                    .await
            }
            AiProvider::AzureOpenAi {
                deployment,
                endpoint,
                api_key,
            } => {
                self.generate_with_azure(deployment, endpoint, api_key.as_deref(), &prompt)
                    .await
            }
        }
    }

    /// Build a prompt for the given narrative type
    fn build_prompt(&self, context: &str, narrative_type: NarrativeType) -> String {
        let system_prompt = self.get_system_prompt();
        let task_prompt = self.get_task_prompt(narrative_type);

        format!(
            "{}\n\n{}\n\nContext:\n{}\n\nGenerate the narrative:",
            system_prompt, task_prompt, context
        )
    }

    /// Get the system prompt for forensic report writing
    fn get_system_prompt(&self) -> &'static str {
        r#"You are an expert digital forensics report writer. Your role is to help forensic examiners write clear, professional, and accurate report sections.

CRITICAL RULES:
1. NEVER fabricate evidence or facts - only describe what is provided in the context
2. Use precise, professional language appropriate for legal proceedings
3. Be objective and avoid speculation
4. Use passive voice where appropriate for formal reporting
5. Include relevant technical details while remaining accessible
6. Avoid definitive conclusions about intent unless explicitly stated in evidence
7. Use qualifying language ("appears to", "indicates", "suggests") where appropriate

FORMATTING:
- Write in clear, well-structured paragraphs
- Use professional forensic terminology
- Be concise but thorough
- Avoid first-person pronouns"#
    }

    /// Get the task-specific prompt for different narrative types
    fn get_task_prompt(&self, narrative_type: NarrativeType) -> &'static str {
        match narrative_type {
            NarrativeType::ExecutiveSummary => {
                r#"Write an executive summary for non-technical readers. This should:
- Summarize the key findings in plain language
- Highlight the most significant discoveries
- Be 2-3 paragraphs maximum
- Avoid technical jargon where possible
- State the scope of the examination
- NOT include specific technical details or file paths"#
            }
            NarrativeType::FindingDescription => {
                r#"Write a detailed finding description that:
- Explains what was discovered
- Describes the forensic significance
- References specific evidence items
- Includes relevant timestamps if available
- Explains the relevance to the investigation
- Uses proper forensic terminology"#
            }
            NarrativeType::TimelineNarrative => {
                r#"Create a narrative description of the timeline events that:
- Tells the story chronologically
- Connects related events
- Highlights significant temporal patterns
- Identifies gaps or anomalies in the timeline
- Maintains factual accuracy"#
            }
            NarrativeType::EvidenceDescription => {
                r#"Describe the evidence item including:
- Physical or logical description
- Acquisition details
- Integrity verification (hash values)
- Relevant technical specifications
- Chain of custody considerations"#
            }
            NarrativeType::Methodology => {
                r#"Describe the forensic methodology used:
- Tools and techniques employed
- Industry standards followed
- Examination procedures
- Verification methods
- Quality assurance measures"#
            }
            NarrativeType::Conclusion => {
                r#"Write a conclusion section that:
- Summarizes the examination results
- States findings clearly and objectively
- Avoids speculation about intent
- Notes any limitations of the examination
- Provides professional opinions where appropriate"#
            }
        }
    }

    /// Generate using Ollama
    async fn generate_with_ollama(
        &self,
        model: &str,
        base_url: Option<&str>,
        prompt: &str,
    ) -> ReportResult<String> {
        use langchain_rust::language_models::llm::LLM;
        use langchain_rust::llm::ollama::client::{Ollama, OllamaClient};
        use std::sync::Arc;

        let url = base_url.unwrap_or("http://localhost:11434");

        // Security: Validate URL to prevent SSRF
        validate_ollama_url(url)?;

        // Audit: Log the AI interaction
        log_ai_interaction("ollama", model, "narrative", prompt.len());

        // Create the Ollama client with custom URL
        let ollama_client = OllamaClient::try_new(url)
            .map_err(|e| ReportError::AiError(format!("Invalid Ollama URL: {}", e)))?;

        let ollama = Ollama::new(Arc::new(ollama_client), model, None);

        let result = ollama
            .invoke(prompt)
            .await
            .map_err(|e| ReportError::AiError(e.to_string()));

        // Audit: Log the response
        log_ai_response(
            "ollama",
            model,
            result.is_ok(),
            result.as_ref().map(|s: &String| s.len()).unwrap_or(0),
        );

        result
    }

    /// Generate using OpenAI
    async fn generate_with_openai(
        &self,
        model: &str,
        api_key: Option<&str>,
        prompt: &str,
    ) -> ReportResult<String> {
        use async_openai::{
            types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
            Client,
        };

        // Security: Log usage (but NEVER log the API key!)
        log_ai_interaction("openai", model, "narrative", prompt.len());

        // Security note: API key should ideally come from secure storage
        // For now, we accept it from the frontend but log a warning if present
        if api_key.is_some() {
            tracing::debug!(target: "ai_audit", 
                "OpenAI API key provided via parameter (consider using secure storage)");
        }

        let client = if let Some(key) = api_key {
            Client::with_config(async_openai::config::OpenAIConfig::new().with_api_key(key))
        } else {
            // Falls back to OPENAI_API_KEY env var
            Client::new()
        };

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages([ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()
                .map_err(|e| ReportError::AiError(e.to_string()))?
                .into()])
            .build()
            .map_err(|e| ReportError::AiError(e.to_string()))?;

        let result = client
            .chat()
            .create(request)
            .await
            .map_err(|e| ReportError::AiError(e.to_string()));

        let response = match result {
            Ok(resp) => resp,
            Err(e) => {
                log_ai_response("openai", model, false, 0);
                return Err(e);
            }
        };

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| ReportError::AiError("No response from OpenAI".to_string()));

        log_ai_response(
            "openai",
            model,
            content.is_ok(),
            content.as_ref().map(|s| s.len()).unwrap_or(0),
        );

        content
    }

    /// Generate using Azure OpenAI
    ///
    /// Uses `async_openai::config::AzureConfig` to connect to an Azure OpenAI Service deployment.
    /// The endpoint should be the Azure resource URL (e.g., `https://your-resource.openai.azure.com`).
    /// API version defaults to `2024-02-01` if not specified in the endpoint.
    async fn generate_with_azure(
        &self,
        deployment: &str,
        endpoint: &str,
        api_key: Option<&str>,
        prompt: &str,
    ) -> ReportResult<String> {
        use async_openai::{
            config::AzureConfig,
            types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
            Client,
        };

        // Security: Log usage (but NEVER log the API key!)
        log_ai_interaction("azure_openai", deployment, "narrative", prompt.len());

        if api_key.is_some() {
            tracing::debug!(target: "ai_audit",
                "Azure OpenAI API key provided via parameter (consider using secure storage)");
        }

        // Build AzureConfig
        let mut config = AzureConfig::new()
            .with_api_base(endpoint)
            .with_deployment_id(deployment)
            .with_api_version("2024-02-01");

        if let Some(key) = api_key {
            config = config.with_api_key(key);
        }
        // If no explicit key, AzureConfig falls back to OPENAI_API_KEY env var

        let client = Client::with_config(config);

        let request = CreateChatCompletionRequestArgs::default()
            .model(deployment)
            .messages([ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()
                .map_err(|e| {
                    ReportError::AiError(format!("Failed to build Azure OpenAI message: {}", e))
                })?
                .into()])
            .build()
            .map_err(|e| {
                ReportError::AiError(format!("Failed to build Azure OpenAI request: {}", e))
            })?;

        let result = client
            .chat()
            .create(request)
            .await
            .map_err(|e| ReportError::AiError(format!("Azure OpenAI API error: {}", e)));

        let response = match result {
            Ok(resp) => resp,
            Err(e) => {
                log_ai_response("azure_openai", deployment, false, 0);
                return Err(e);
            }
        };

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| ReportError::AiError("No response from Azure OpenAI".to_string()));

        log_ai_response(
            "azure_openai",
            deployment,
            content.is_ok(),
            content.as_ref().map(|s| s.len()).unwrap_or(0),
        );

        content
    }

    /// Generate multiple narratives for a report
    pub async fn enhance_report(
        &self,
        context: &ReportContext,
    ) -> ReportResult<EnhancedNarratives> {
        let executive_summary = if context.generate_executive_summary {
            Some(
                self.generate_narrative(&context.evidence_summary, NarrativeType::ExecutiveSummary)
                    .await?,
            )
        } else {
            None
        };

        let methodology = if context.generate_methodology {
            Some(
                self.generate_narrative(&context.tools_used, NarrativeType::Methodology)
                    .await?,
            )
        } else {
            None
        };

        let conclusion = if context.generate_conclusion {
            Some(
                self.generate_narrative(&context.findings_summary, NarrativeType::Conclusion)
                    .await?,
            )
        } else {
            None
        };

        Ok(EnhancedNarratives {
            executive_summary,
            methodology,
            conclusion,
        })
    }
}

/// Context for AI-enhanced report generation
#[derive(Debug, Clone)]
pub struct ReportContext {
    /// Summary of evidence items
    pub evidence_summary: String,
    /// Summary of findings
    pub findings_summary: String,
    /// Tools used (for methodology generation)
    pub tools_used: String,
    /// Whether to generate executive summary
    pub generate_executive_summary: bool,
    /// Whether to generate methodology
    pub generate_methodology: bool,
    /// Whether to generate conclusion
    pub generate_conclusion: bool,
}

/// AI-generated narratives
#[derive(Debug, Clone)]
pub struct EnhancedNarratives {
    /// Generated executive summary
    pub executive_summary: Option<String>,
    /// Generated methodology description
    pub methodology: Option<String>,
    /// Generated conclusion
    pub conclusion: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_building() {
        let ai = AiAssistant::ollama("llama3.2");
        let prompt = ai.build_prompt("Test context", NarrativeType::ExecutiveSummary);

        assert!(prompt.contains("executive summary"));
        assert!(prompt.contains("Test context"));
        assert!(prompt.contains("NEVER fabricate"));
    }

    #[test]
    fn test_provider_creation() {
        let ollama = AiAssistant::ollama("mistral");
        let openai = AiAssistant::openai("gpt-4");
        let azure = AiAssistant::new(AiProvider::AzureOpenAi {
            deployment: "gpt-4".to_string(),
            endpoint: "https://my-resource.openai.azure.com".to_string(),
            api_key: Some("test-key".to_string()),
        });

        // Just verify they can be created without panicking
        drop(ollama);
        drop(openai);
        drop(azure);
    }

    #[test]
    fn test_azure_provider_without_key() {
        let azure = AiAssistant::new(AiProvider::AzureOpenAi {
            deployment: "gpt-4o".to_string(),
            endpoint: "https://test.openai.azure.com".to_string(),
            api_key: None,
        });
        // Azure without explicit key falls back to OPENAI_API_KEY env var
        let prompt = azure.build_prompt("Test context", NarrativeType::Methodology);
        assert!(prompt.contains("methodology"));
        assert!(prompt.contains("Test context"));
    }

    #[test]
    fn test_all_narrative_types_produce_prompts() {
        let ai = AiAssistant::ollama("llama3.2");
        let types = [
            NarrativeType::ExecutiveSummary,
            NarrativeType::FindingDescription,
            NarrativeType::TimelineNarrative,
            NarrativeType::EvidenceDescription,
            NarrativeType::Methodology,
            NarrativeType::Conclusion,
        ];
        for nt in types {
            let prompt = ai.build_prompt("ctx", nt);
            assert!(!prompt.is_empty());
            assert!(prompt.contains("ctx"));
            assert!(prompt.contains("NEVER fabricate"));
        }
    }
}
