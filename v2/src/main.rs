#[tokio::main(worker_threads = 2)]
async fn main() {
    if let Err(error) = tezbar_extension_sdk::serve(tezbar_json_format::Adapter::default()).await {
        eprintln!("json-format: {error}");
        std::process::exit(1);
    }
}
