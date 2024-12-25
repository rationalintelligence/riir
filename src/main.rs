mod opts;

use anyhow::{Error, Result};
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
    CreateChatCompletionRequestArgs,
};
use async_openai::Client;
use clap::Parser;
use opts::Opts;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    let opts = Opts::parse();
    let config = OpenAIConfig::default();
    let client = Client::with_config(config);
    let path = &opts.file;

    println!("Reading file: {}", path.display());

    let contents = fs::read(path).await?;
    let code = String::from_utf8(contents)?;

    println!("Asking AI...");

    const SYSTEM: &str = "You are an experienced Rust developer who writes idiomatic code.
    Refactor files that is provided to you, according to provided instructions,
    but an output has to be a code only.";

    let request = format!(
        "Do the following with the code: {}. \nThe code: ```{}```",
        "", code
    );

    let request = CreateChatCompletionRequestArgs::default()
        .model("o3")
        .messages([
            ChatCompletionRequestSystemMessage::from(SYSTEM).into(),
            ChatCompletionRequestUserMessage::from(request).into(),
        ])
        .build()?;
    let response = client.chat().create(request).await?;
    let contents = response
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| Error::msg("No responses provided by the model."))?
        .message
        .content
        .ok_or_else(|| Error::msg("The response doesn't contain a message."))?;

    println!("Writing the diff...");
    fs::write(path, contents).await?;

    println!("Done 🦀");
    Ok(())
}
