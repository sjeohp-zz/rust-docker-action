FROM rustlang/rust:nightly

WORKDIR /usr/src/rust-docker-action
COPY . .

RUN cargo +nightly install --path . --force 

ENTRYPOINT ["rust-docker-action"]
