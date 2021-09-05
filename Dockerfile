FROM rustlang/rust:nightly as builder
WORKDIR /
COPY . .
RUN cargo install --path .


FROM debian:bullseye-slim

RUN apt update && apt install

COPY --from=builder /usr/local/cargo/bin/singlegame /usr/local/bin/singlegame

CMD ["singlegame"]