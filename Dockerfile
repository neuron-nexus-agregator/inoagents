# Этап 1: сборка
FROM rustlang/rust:nightly AS builder

WORKDIR /app

# Копируем Cargo файлы
COPY Cargo.toml Cargo.lock ./

# Подготавливаем зависимости (кэширование)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Теперь копируем исходники
COPY src ./src
COPY assets ./assets

# Собираем проект
RUN cargo build --release

# Этап 2: "чистый" образ
FROM debian:bookworm-slim

# Устанавливаем SSL и сертификаты
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    libsqlite3-0 \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app


COPY --from=builder /app/target/release/service /app/app

# Копируем папку assets
COPY --from=builder /app/assets ./assets

EXPOSE 8080

CMD ["./app"]
