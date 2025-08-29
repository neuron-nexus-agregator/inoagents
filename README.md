# README.md

# INO Agents API

Простой сервер на Rust с использованием Actix для поиска иноагентов и других запрещенных имен в тексте. Проект предоставляет API для проверки данных, добавления новых записей и обновления справочников.

---

## Стек технологий

* **Rust** – основной язык
* **Actix-web** – веб-сервер и маршрутизация
* **Tokio** – асинхронная обработка
* **SQLite** – база данных
* **YandexEmbedding** – векторизация текста
* **PythonEntities** – интеграция с Python для извлечения сущностей

---

## Установка и запуск

1. Склонировать репозиторий:

```bash
git clone https://github.com/neuron-nexus-agregator/inoagents.git
cd inoagents
```

2. Установить зависимости Rust:

```bash
cargo build
```

3. Настроить переменные окружения в `.env`:

```
YANDEX_SECRET=<токен от Yandex GPT>
YANDEX_MODEL=emb://<yandex bucket>/text-search-query/latest
YANDEX_URL=https://llm.api.cloud.yandex.net/foundationModels/v1/textEmbedding
ENTITIES_URL=<url для сервиса извлечения именованных сущностей>
FULL_DATA=<true / false для получения полной информации об одобренных именах>
```

4. Создать базу данных `assets/db/ino.sqlite` (если она ещё не создана).

5. Запустить сервер:

```bash
cargo run
```

Сервер будет слушать адрес `0.0.0.0:8080`.

---

## API

### 1. Проверка по ID

```
GET /check/{id}
```

Возвращает информацию по конкретному ID.

---

### 2. Проверка по тексту

```
POST /check
Content-Type: application/json
Body: { "text": "текст для проверки" }
```

Возвращает результаты анализа текста.

---

### 3. Обновление справочников

```
GET /update
```

Обновляет warning names из базы данных.

---

### 4. Добавление новых записей

```
POST /add
Content-Type: application/json
Body: { "records": [ {...}, {...} ] }
```

Добавляет новые записи в Checker.

---

## Структура проекта

```
src/
 ├─ db/              # Работа с базой данных
 ├─ embedding/       # Векторизация текста
 ├─ ino_api/         # API и хендлеры
 ├─ ino_checker/     # Логика проверки
 ├─ ner/             # Извлечение сущностей
 ├─ rv/              # Для работы с сайтом RV
 ├─ utils/           # Вспомогательные функции
 └─ main.rs          # Точка входа
```

---

## Примечания

* Все mutable поля Checker защищены через `tokio::sync::Mutex` для безопасной работы с несколькими запросами.
* Для корректной работы API необходимо, чтобы типы `web::Data` совпадали с хендлерами.
