use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::CreateEmbeddingRequestArgs;
use std::error::Error;

use crate::tui::app::LlmConfig;

#[derive(Debug, Clone)]
pub struct LlmClient {
    pub api_base: String,
    pub api_key: String,
    pub embedding_model: String,
}

impl From<&LlmConfig> for LlmClient {
    fn from(config: &LlmConfig) -> Self {
        LlmClient {
            api_base: config.api_base.clone(),
            api_key: config.api_key.clone(),
            embedding_model: config.embeddings_model.clone(),
        }
    }
}

impl LlmClient {
    pub async fn embed(&mut self, content: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        let client = Client::with_config(
            OpenAIConfig::new()
                .with_api_base(self.api_base.clone())
                .with_api_key(self.api_key.clone()),
        );

        let request = CreateEmbeddingRequestArgs::default()
            .model(self.embedding_model.clone())
            .input([content])
            .build()?;

        let response = client.embeddings().create(request).await?;

        let choice: Vec<f32> = response.data[0].embedding.clone();

        Ok(choice)
    }
}
