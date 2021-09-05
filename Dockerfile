FROM rustlang/rust:nightly

WORKDIR /home

COPY . .


RUN cargo build --release

CMD ./target/release/singlegame