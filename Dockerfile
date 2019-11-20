FROM rustlang/rust:nightly

WORKDIR /usr/src/rust-docker-action
COPY . .

RUN cargo +nightly install --path .

CMD ["GITHUB_API_TOKEN=f7c45dae699893991cabdb06bfca658f7bb5baa5 rust-docker-action"]
