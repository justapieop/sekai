FROM rust:1.94.0-alpine3.23 AS build_stage
WORKDIR /build
COPY ../.. .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13
WORKDIR /app
COPY --from=build_stage /build/target/release/sekai .
CMD ["./sekai"]