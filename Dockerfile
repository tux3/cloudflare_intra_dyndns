ARG APP_NAME=cloudflare_intra_dyndns

FROM rust:1.61 as builder
ARG APP_NAME

# Prebuild deps layer
COPY Cargo.* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
RUN rm src/ -r
RUN rm target/release/deps/${APP_NAME}*

# Build
COPY src ./src
RUN cargo build --release

FROM debian:bullseye-slim
ARG APP_NAME
ARG CONFIG_FILE=/etc/${APP_NAME}/${APP_NAME}.conf

COPY --from=builder target/release/${APP_NAME} .

ENV CONFIG_FILE ${CONFIG_FILE}
ENV APP_NAME ${APP_NAME}

CMD ./${APP_NAME} -c ${CONFIG_FILE}

