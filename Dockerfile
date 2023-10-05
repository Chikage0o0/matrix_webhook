FROM rust:1.72.1-alpine3.18 as builder
COPY . /app
WORKDIR /app
RUN apk add --no-cache --virtual .build-deps \
        make \
        musl-dev \
        openssl-dev \
        perl \
        pkgconfig \
        openssl-libs-static \
    && cargo build --release

FROM alpine:3.18
LABEL maintainer="Chikage <chikage@939.me>" \
      org.opencontainers.image.source="https://github.com/Chikage0o0/matrix_webhook" \
COPY --from=builder /app/target/release/matrix_webhook \
                    /usr/local/bin/matrix_webhook
USER nobody
ENTRYPOINT ["/usr/local/bin/matrix_webhook"]