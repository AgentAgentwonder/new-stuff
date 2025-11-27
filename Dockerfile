FROM rust:latest

WORKDIR /app
COPY . .

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

RUN cargo build --release

CMD ["./src-tauri/target/release/app"]
