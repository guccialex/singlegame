FROM rust

WORKDIR /home

COPY . .


RUN cargo build --release

CMD ROCKET_PORT=8000 cargo run --release