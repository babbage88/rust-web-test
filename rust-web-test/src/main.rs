use clap::Parser;
use rand::Rng;
use reqwest::Client;
use std::time::{Duration, Instant};

/// Simple program to send concurrent GET requests with random parameters
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Total number of requests to send
    #[arg(short, long, default_value_t = 5000)]
    total_requests: usize,

    /// Number of requests to send in each batch
    #[arg(short, long, default_value_t = 1000)]
    batch_size: usize,
}

// Function to generate a random float between `min` and `max`
fn random_float(min: f64, max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

// Function to generate a random integer between `min` and `max`
fn random_int(min: u32, max: u32) -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

// Function to send a GET request with random parameters
async fn send_request(client: &Client, id: usize) {
    let init_amount = random_int(500, 100000);
    let monthly_contribution = random_int(50, 5000);
    let interest_rate = format!("{:.2}", random_float(0.1, 200.0));
    let number_of_years = random_int(1, 50);

    let url = format!(
        "https://calc.test.trahan.dev/calculated?initAmount={}&monthlyContribution={}&interestRate={}&numberOfYears={}",
        init_amount, monthly_contribution, interest_rate, number_of_years
    );

    let request = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .build();

    match request {
        Ok(req) => {
            match client.execute(req).await {
                Ok(resp) => {
                    println!(
                        "Request {} - initAmount: {}, monthlyContribution: {}, interestRate: {}, numberOfYears: {}, Status: {}",
                        id, init_amount, monthly_contribution, interest_rate, number_of_years, resp.status()
                    );
                }
                Err(e) => {
                    eprintln!("Error in request {}: {}", id, e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error creating request {}: {}", id, e);
        }
    }
}

// Function to send a batch of requests asynchronously
async fn send_batch(client: &Client, start_id: usize, num_requests: usize) {
    let mut tasks = vec![];

    for i in 0..num_requests {
        let client_ref = client.clone();
        let task = tokio::spawn(async move {
            send_request(&client_ref, start_id + i).await;
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }
}

// Function to send requests concurrently in batches
async fn send_concurrent_requests(total_requests: usize, batch_size: usize) {
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

        send_batch(&client, start_id, requests_in_batch).await;
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
    send_concurrent_requests(args.total_requests, args.batch_size).await;

    // Measure total elapsed time and print it
    let duration = start.elapsed();
    println!(
        "All requests completed. Total time: {:.2?} for {} requests",
        duration, args.total_requests
    );
}
