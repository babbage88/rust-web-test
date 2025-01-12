use clap::Parser;
use reqwest::Client;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target URL to send requests to (required positional argument)
    #[arg(value_name = "URL")]
    url: String,

    /// Bearer Auth token for protected routes
    #[arg(value_name = "TOKEN")]
    token: String,

    /// Total number of requests to send (optional positional argument with default)
    #[arg(value_name = "TOTAL_REQUESTS", default_value_t = 5000)]
    total_requests: usize,

    /// Number of requests to send in each batch (optional positional argument with default)
    #[arg(value_name = "BATCH_SIZE", default_value_t = 1000)]
    batch_size: usize,
}

async fn send_get_request(client: &Client, id: usize, uri: String, token: String) {
    // Construct the Bearer token
    let bearer_hdr = format!("Bearer {}", token);
    println!("Authorization Header: {}", bearer_hdr); // Debugging the token

    // Build the GET request
    let request = client
        .get(&uri)
        
        .header("accept", "application/json")
        .header("Authorization", &bearer_hdr)
        .build();

    // Debug the request
    println!("{:#?}", request);

    // Execute the request and handle the response
    match request {
        Ok(req) => match client.execute(req).await {
            Ok(resp) => {
                println!(
                    "Request {}, URL: {}, Status: {}, Response: {:?}",
                    id,
                    uri,
                    resp.status(),
                    resp.text().await.unwrap_or_else(|_| "Failed to read response text".to_string())
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


async fn send_batch(client: &Client, start_id: usize, num_requests: usize, uri: String, token: String) {
    let mut tasks = vec![];

    for i in 0..num_requests {
        let client_ref = client.clone();
        let uri_clone = uri.clone(); // Clone the URI for each task
        let token_clone = token.clone(); // Clone the token for each task
        let task = tokio::spawn(async move {
            send_get_request(&client_ref, start_id + i, uri_clone, token_clone).await;
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }
}

async fn send_concurrent_requests(total_requests: usize, batch_size: usize, uri: String, token: String) {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(5000)  // Increase the connection pool size
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
    // Start measuring time
    let start = Instant::now();

    // Parse CLI arguments using clap
    let args = Args::parse();
    

    // Execute the request sending process
    send_concurrent_requests(args.total_requests, args.batch_size, args.url, args.token).await;

    // Measure total elapsed time and print it
    let duration = start.elapsed();
    println!(
        "All requests completed. Total time: {:.2?} for {} requests",
        duration, args.total_requests
    );
}
