# Sword Web Application Example

This example demonstrates a user management API using Sword's web controllers. It includes CRUD operations for users, request validation, and PostgreSQL integration.

## Running the Example

1. Ensure Docker is installed.
2. Navigate to the example's directory: `cd examples/web`
3. Start PostgreSQL: `docker compose up -d`
4. Run the application: `cargo run`

The API will be available at http://localhost:8081/api/users.

## Browser Client

A simple vanilla JS client is included in `public/` and served by the app itself:

- Open http://localhost:8081/static/ to load `index.html` and `app.js`.

It consumes the API from the browser (same origin, no CORS):

- `GET /api/users` - renders the user list
- `POST /api/users` - creates a user via the form
- `DELETE /api/users/{id}` - deletes a user
- `GET /api/sse/countdown` - streams the countdown events via `EventSource`

> **Note:** the user CRUD operations require PostgreSQL (`docker compose up -d`). The SSE countdown works without a database.

## Server-Sent Events

The example also exposes an SSE stream at `GET /api/sse/countdown`:

```bash
curl -N http://localhost:8081/api/sse/countdown
```

This returns a `text/event-stream` response with a `countdown` event every 250ms followed by a final `done` event.

> **Note:** SSE connections are long-lived. If `request-timeout` is enabled in your config, the global timeout layer will terminate the stream, so leave it disabled or set a generous timeout for routes that stream events.
