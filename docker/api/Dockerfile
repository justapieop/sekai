FROM dhi.io/rust:1.94.0-debian13 AS build_stage
WORKDIR /build
COPY ../.. .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13
WORKDIR /app
COPY --from=build_stage /build/target/release/sekai .
CMD ["./sekai"]