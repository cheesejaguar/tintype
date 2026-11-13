# syntax=docker/dockerfile:1
FROM rust:bookworm AS build
WORKDIR /source
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY site/assets/demo.png site/assets/demo.png
RUN cargo build --release --locked --bin tintype

FROM debian:bookworm-slim AS runtime-base
USER 65532:65532
ENV TINTYPE_MODEL_DIR=/models
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/tintype"]
CMD ["serve", "--bind", "0.0.0.0:8080", "--device", "cpu"]

FROM runtime-base AS runtime
COPY --from=build /source/target/release/tintype /usr/local/bin/tintype
COPY LICENSE-APACHE LICENSE-MIT NOTICE.md /usr/local/share/doc/tintype/

FROM runtime AS bundled
COPY models/ /models/

FROM runtime-base AS release-code-only
COPY container-release/tintype /usr/local/bin/tintype
COPY container-release/LICENSE-APACHE container-release/LICENSE-MIT container-release/NOTICE.md /usr/local/share/doc/tintype/

FROM release-code-only AS release-bundled
COPY container-release/models/ /models/

FROM runtime AS code-only
