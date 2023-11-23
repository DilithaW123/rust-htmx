FROM rust:1.74 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release


FROM rust:1.74-slim
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/rust-htmx .
COPY --from=builder /usr/src/app/static ./static
COPY --from=builder /usr/src/app/templates ./templates
COPY --from=builder /usr/src/app/.env ./.env
EXPOSE 8080
CMD ["./rust-htmx"]

