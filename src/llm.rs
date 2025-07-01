use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, CreateEmbeddingRequestArgs};
use std::error::Error;

pub async fn embed(thought: &str) -> Result<Vec<f32>, Box<dyn Error>> {
    let api_key = "ollama";
    let api_base = "http://localhost:11434/v1";

    let client = Client::with_config(
        OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base),
    );

    let request = CreateEmbeddingRequestArgs::default()
        .model("all-minilm:latest")
        .input([
            thought,
        ])
        .build()?;

    let response = client.embeddings().create(request).await?;

    let choice: Vec<f32> = response.data[0].embedding.clone();

    Ok(choice)
}

pub async fn chat(prompt: &str) -> Result<(), Box<dyn Error>> {
    let api_key = "ollama";
    let api_base = "http://localhost:11434/v1";

    let client = Client::with_config(
        OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base),
    );

    let model = "llama3.2:1b";

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u32)
        .model(model)
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
