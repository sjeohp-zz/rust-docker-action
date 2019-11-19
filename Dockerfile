FROM rust:1.31

#WORKDIR /usr/src/rust-docker-action
COPY . .

RUN cargo install --path .

CMD ["rust-docker-action"]
