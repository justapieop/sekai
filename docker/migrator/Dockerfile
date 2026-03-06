FROM rust:1.94.0-alpine3.23
RUN apk add pkg-config && cargo install sqlx-cli
CMD ["sqlx", "migrate", "run"]