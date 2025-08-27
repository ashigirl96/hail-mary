use anthropic_client::{complete, load_auth};
use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Get message from command-line arguments
    let args: Vec<String> = env::args().collect();
    let message = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "What is 1+1?".to_string()
    };

    // Load OAuth authentication
    let mut auth = load_auth().await?;

    // Set model and make request
    let model = "claude-3-5-sonnet-20241022";
    let response = complete(model, &message, &mut auth).await?;
    println!("{}", response);

    Ok(())
}
