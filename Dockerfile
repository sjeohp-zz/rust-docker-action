FROM rustlang/rust:nightly

WORKDIR /usr/src/rust-docker-action
COPY . .

RUN cargo +nightly install --path .

CMD ["rust-docker-action"]
