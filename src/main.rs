mod opts;

use anyhow::{anyhow, Result};
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
    CreateChatCompletionRequestArgs,
};
use async_openai::Client;
use clap::Parser;
use itertools::Itertools;
use opts::Opts;
use std::collections::VecDeque;
use tokio::{
    fs,
    io::{self, AsyncBufReadExt, BufReader},
};

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

    println!("What should I do with the file?");

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut prompt = String::new();
    reader.read_line(&mut prompt).await?;

    println!("Asking AI...");

    const SYSTEM: &str = "You are an experienced Rust developer who writes idiomatic code.
    Refactor files that is provided to you, according to provided instructions,
    but an output has to be a code only.";

    let request = format!(
        "Do the following with the code: {}. \nThe code:\n\n```{}```",
        prompt, code
    );

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o")
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
        .ok_or_else(|| anyhow!("No responses provided by the model."))?
        .message
        .content
        .ok_or_else(|| anyhow!("The response doesn't contain a message."))?;

    let mut lines: VecDeque<_> = contents.lines().collect();
    if let Some(front) = lines.front() {
        if front.starts_with("```") {
            lines.pop_front();
        }
    }
    if let Some(back) = lines.back() {
        if back.starts_with("```") {
            lines.pop_back();
        }
    }
    let contents = lines.into_iter().join("\n");

    println!("Writing the diff...");
    fs::write(path, contents).await?;

    println!("Done 🦀");
    Ok(())
}
