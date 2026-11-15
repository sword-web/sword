# Sword Web Application Example

A users API built with Sword web controllers: CRUD, validation, error handling,
interceptors, server-sent events, OpenAPI, and a small browser client.

It uses an **in-memory SQLite database**, so it runs with no Docker and no
external service. The data lives as long as the process does.

## Run

```bash
cd examples/web
cargo run
```

The API is served under the `/api` prefix:

```bash
curl http://localhost:8081/api/users
```

Other entry points:

| URL | What it is |
| --- | --- |
| `http://localhost:8081/api/users` | Users CRUD |
| `http://localhost:8081/api/sse/countdown` | Server-sent events stream |
| `http://localhost:8081/api/docs` | Swagger UI for the OpenAPI specs in `config/` |
| `http://localhost:8081/api/static/` | Browser client that consumes the API |

## Create a user

```bash
curl -X POST http://localhost:8081/api/users \
  -H 'content-type: application/json' \
  -d '{"username": "luciano", "password": "secret123"}'
```

Validation errors come back with the standard error envelope:

```bash
curl -X POST http://localhost:8081/api/users \
  -H 'content-type: application/json' \
  -d '{"username": "luciano", "password": "123"}'
```

```json
{
  "code": 400,
  "success": false,
  "message": "Invalid request body",
  "errors": {
    "password": [{ "code": "length", "message": "Password must be between 6 and 100 characters" }]
  }
}
```

## What this example shows

- **Modules**: `UsersModule`, `SharedModule`, `MailerModule`, `SseModule`.
- **Providers and components**: `Database` and `Hasher` are providers;
  `UserRepository` is a component built from them.
- **Validation**: `req.body_validator::<CreateUserDto>()` with `validator`.
- **Error handling**: `AppError` mapped to HTTP responses with `HttpError`.
- **Interceptors**: `RequestLoggingInterceptor` runs for every route of the
  controller; `AuditInterceptor` runs only on the route that declares it and
  reads the value the first one left in the request extensions.
- **Events**: creating a user publishes `UserCreatedEvent`, which the mailer
  handler consumes through the in-memory transport.
- **SSE**: `#[sse]` route streamed from an `EventStream`.
- **Static files**: the client in `public/` is served by the `serve-dir` layer.

Job scheduling lives in its own example: [`examples/jobs`](../jobs).

## Database

`Database::new()` opens `sqlite::memory:` with a single connection, so every
query shares the same in-memory database. The table is created on startup:

```sql
CREATE TABLE IF NOT EXISTS users (
    id BLOB PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);
```

To move to a real database, replace the pool with your own connection string and
keep the rest of the layers unchanged.
