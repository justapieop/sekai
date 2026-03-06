FROM rust:1.94.0-alpine3.23
RUN apk add pkgconf openssl openssl-dev && cargo install sqlx-cli
CMD ["sqlx", "migrate", "run"]