use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, CreateEmbeddingRequestArgs,
};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct LlmClient {
    pub api_base: String,
    pub api_key: String,
    pub embedding_model: String,
    pub chat_model: String,
}

impl LlmClient {
    pub fn new(
        api_base: String,
        api_key: String,
        embedding_model: String,
        chat_model: String,
    ) -> Self {
        LlmClient {
            api_base,
            api_key,
            embedding_model,
            chat_model,
        }
    }

    pub fn default() -> Result<Self, Box<dyn Error>> {
        // TODO: load from config, not env
        let api_base = std::env::var("API_BASE")?;
        let api_key = std::env::var("API_KEY")?;
        let embedding_model = std::env::var("EMBEDDINGS_MODEL")?;
        let chat_model = std::env::var("CHAT_MODEL")?;
        Ok(LlmClient {
            api_base,
            api_key,
            embedding_model,
            chat_model,
        })
    }

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

    pub async fn chat(&mut self, prompt: &str) -> Result<(), Box<dyn Error>> {
        let client = Client::with_config(
            OpenAIConfig::new()
                .with_api_base(self.api_base.clone())
                .with_api_key(self.api_key.clone()),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u32)
            .model(self.chat_model.clone())
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a helpful assistant with a Italian accent.")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?
                    .into(),
            ])
            .build()?;

        let response = client.chat().create(request).await?;

        println!("\nResponse:\n");
        for choice in response.choices {
            println!(
                "{}: Role: {}  Content: {:?}",
                choice.index, choice.message.role, choice.message.content
            );
        }

        Ok(())
    }
}
