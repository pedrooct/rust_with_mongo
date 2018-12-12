FROM rust:1.30-slim

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080


WORKDIR /home/rust/rest_rust_app
COPY ./rust/rest_rust_app ./

#Updates que possam ser necessários e instalação de dados
RUN rustup default nightly && rustup update && cargo update && cargo install cargo-watch

EXPOSE 8080

CMD cargo watch -x run
