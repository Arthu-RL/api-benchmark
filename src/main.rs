use reqwest::Client;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering, AtomicU64};
use tokio::time::{Instant, Duration};
use bytes::Bytes;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    url: String,

    #[arg(long, default_value_t = 100)]
    concurrency: usize,
    #[arg(long, default_value_t = 1000)]
    requests_per_worker: usize,
    #[arg(long, default_value_t = 100)]
    pool_max_idle_per_host: usize,

    #[arg(long)]
    file_path_for_body_data: String,
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();
    let client: Client = match Client::builder().pool_max_idle_per_host(args.pool_max_idle_per_host).build() {
        Ok(client) => client,
        Err(err) => {
            panic!("Failed to build client: {}", err);
        } 
    };
    let data: Vec<u8> = match std::fs::read(args.file_path_for_body_data) {
        Ok(data) => data,
        Err(err) => {
            panic!("Failed to read file: {}", err);
        } 
    };
    let body: Bytes =  Bytes::from(data);

    let url: String = args.url.trim().to_lowercase();

    let concurrency: usize = args.concurrency;
    let requests_per_worker: usize = args.requests_per_worker;

    let start: Instant = Instant::now();

    let mut handles: Vec<_> = Vec::new();

    let success: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let errors: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    let total_latency_ns = Arc::new(AtomicU64::new(0));
    let min_latency_ns = Arc::new(AtomicU64::new(u64::MAX));
    let max_latency_ns = Arc::new(AtomicU64::new(0));
    
    for _ in 0..concurrency {
        let client: Client = client.clone();
        let body: Bytes = body.clone();
        let url_value = url.clone();

        let success: Arc<AtomicUsize> = success.clone();
        let errors: Arc<AtomicUsize> = errors.clone();

        let total_latency_ns = total_latency_ns.clone();
        let min_latency_ns = min_latency_ns.clone();
        let max_latency_ns = max_latency_ns.clone();

        handles.push(tokio::spawn(async move {
            for i in 0..requests_per_worker {
                let start_req = Instant::now();
                match client.post(&url_value).body(body.clone()).send().await {
                    Ok(response) => {
                        let latency: u64 = start_req.elapsed().as_nanos() as u64;

                        total_latency_ns.fetch_add(latency, Ordering::Relaxed);

                        min_latency_ns.fetch_min(latency, Ordering::Relaxed);
                        max_latency_ns.fetch_max(latency, Ordering::Relaxed);
                        
                        if response.status().is_success() {
                            success.fetch_add(1, Ordering::Relaxed);
                        } else {
                            errors.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Err(err) => {
                        eprintln!("Request {} failed: {}", i, err);
                        errors.fetch_add(1, Ordering::Relaxed);
                    } 
                }
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let elapsed: Duration = start.elapsed();
    let success_count = success.load(Ordering::Relaxed);
    let error_count = errors.load(Ordering::Relaxed);
    let total = success_count + error_count;
    let avg_latency_ns = total_latency_ns.load(Ordering::Relaxed) / success_count.max(1) as u64;

    println!();
    println!("==================== Benchmark Results ====================");
    println!("Target URL        : {}", url);
    println!("Concurrency       : {}", concurrency);
    println!("Requests/Worker   : {}", requests_per_worker);
    println!("-----------------------------------------------------------");

    println!("Requests");
    println!("  Total           : {}", total);
    println!("  Success         : {}", success_count);
    println!("  Errors          : {}", error_count);
    println!("-----------------------------------------------------------");

    println!("Timing");
    println!("  Elapsed         : {:.3}s", elapsed.as_secs_f64());
    println!("  Throughput      : {:.2} req/s",
        total as f64 / elapsed.as_secs_f64()
    );
    println!("-----------------------------------------------------------");

    println!("Latency (milliseconds)");
    println!("  Average         : {:>8.3}", avg_latency_ns as f64 / 1_000_000.0);
    println!("  Min             : {:>8.3}", min_latency_ns.load(Ordering::Relaxed) as f64 / 1_000_000.0);
    println!("  Max             : {:>8.3}", max_latency_ns.load(Ordering::Relaxed) as f64 / 1_000_000.0);
    println!("===========================================================");
    println!();
}