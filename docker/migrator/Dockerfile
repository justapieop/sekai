FROM dhi.io/rust:1.94.0-debian13
RUN apt-get install pkg-config && cargo install sqlx-cli
CMD ["sqlx", "migrate", "run"]