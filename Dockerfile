FROM rust:1.31

COPY . .

RUN cargo install --path .

CMD ["rust-docker-action"]
