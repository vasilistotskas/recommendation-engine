$env:RATE_LIMIT_MAX_REQUESTS='100000'
$env:RATE_LIMIT_WINDOW_SECS='60'
cargo run --release --bin recommendation-api
