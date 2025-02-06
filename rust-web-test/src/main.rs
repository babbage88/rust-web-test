use clap::{ArgAction, Parser};
use reqwest::Client;
use colored::Colorize;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target URL to send requests to (Positional Argument)
    #[arg(value_name = "URL", index = 1)]
    url: String,

    /// Bearer Auth token for protected routes (Optional Positional Argument)
    #[arg(value_name = "TOKEN", index = 2)]
    token: Option<String>,

    /// Flag to send requests without authentication (Keyword Argument)
    #[arg(long, action = ArgAction::SetTrue)]
    unauthenticated: bool,

    /// Positional argument for total number of requests
    #[arg(value_name = "TOTAL_REQUESTS", index = 3)]
    total_requests_positional: Option<usize>,

    /// Positional argument for batch size
    #[arg(value_name = "BATCH_SIZE", index = 4)]
    batch_size_positional: Option<usize>,

    /// Flag-based total requests
    #[arg(long = "total-requests")]
    total_requests_flag: Option<usize>,

    /// Flag-based batch size
    #[arg(long = "batch-size")]
    batch_size_flag: Option<usize>,
}

async fn send_get_request(client: &Client, id: usize, uri: String, token: Option<String>) {
    let mut request = client.get(&uri).header("accept", "application/json");

    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    match request.build() {
        Ok(req) => match client.execute(req).await {
            Ok(resp) => {
                let status = resp.status();
                let status_text = if status == 200 {
                    format!("{}", status).green().bold()
                } else {
                    format!("{}", status).red().bold()
                };

                println!(
                    "Request {}, URL: {}, Status: {}",
                    id.to_string().yellow().bold(),
                    uri.blue().bold(),
                    status_text
                );
            }
            Err(e) => {
                eprintln!("Error in request {}: {}", id, e);
            }
        },
        Err(e) => {
            eprintln!("Error creating request {}: {}", id, e);
        }
    }
}

async fn send_batch(client: &Client, start_id: usize, num_requests: usize, uri: String, token: Option<String>) {
    let mut tasks = vec![];

    for i in 0..num_requests {
        let client_ref = client.clone();
        let uri_clone = uri.clone();
        let token_clone = token.clone();
        let task = tokio::spawn(async move {
            send_get_request(&client_ref, start_id + i, uri_clone, token_clone).await;
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }
}

async fn send_concurrent_requests(total_requests: usize, batch_size: usize, uri: String, token: Option<String>) {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(5000)
        .build()
        .unwrap();

    let num_batches = (total_requests + batch_size - 1) / batch_size;
    let mut total_sent = 0;

    for batch in 0..num_batches {
        let start_id = batch * batch_size;
        let requests_in_batch = std::cmp::min(batch_size, total_requests - start_id);
        println!("Starting batch {} with {} requests...", batch + 1, requests_in_batch);

        send_batch(&client, start_id, requests_in_batch, uri.clone(), token.clone()).await;
        total_sent += requests_in_batch;

        println!("Batch {} completed.", batch + 1);
    }

    println!("Total requests sent: {}", total_sent);
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let start = Instant::now();
    let args = Args::parse();

    // Resolve total_requests: Prefer the flag if provided, otherwise use positional, fallback to default
    let total_requests = args.total_requests_flag.or(args.total_requests_positional).unwrap_or(10);
    
    // Resolve batch_size: Prefer the flag if provided, otherwise use positional, fallback to default
    let batch_size = args.batch_size_flag.or(args.batch_size_positional).unwrap_or(2);

    let token = if args.unauthenticated { None } else { args.token };

    send_concurrent_requests(total_requests, batch_size, args.url, token).await;

    let duration = start.elapsed();
    println!(
        "All requests completed. Total time: {:.2?} for {} requests",
        duration, total_requests
    );
}

