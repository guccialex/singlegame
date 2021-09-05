FROM rustlang/rust:nightly
WORKDIR /
COPY . .
RUN cargo install --path .

CMD ["singlegame"]