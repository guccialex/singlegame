FROM rust:nightly

WORKDIR /home

COPY . .


RUN cargo build --release

CMD cargo run --release