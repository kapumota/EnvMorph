FROM rust:1.75-slim AS build
WORKDIR /src
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends bash gawk mawk graphviz ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/envmorph /usr/local/bin/envmorph
WORKDIR /work
COPY benchmarks ./benchmarks
COPY experiments ./experiments
COPY tests ./tests
ENV ENVMORPH_BIN=/usr/local/bin/envmorph
CMD ["bash", "tests/integration.sh"]
