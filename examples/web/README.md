# Sword Web Application Example

This example demonstrates a user management API using Sword's web controllers. It includes CRUD operations for users, request validation, and PostgreSQL integration.

## Running the Example

1. Ensure Docker is installed.
2. Navigate to the example's directory: `cd examples/web`
3. Start PostgreSQL: `docker compose up -d`
4. Run the application: `cargo run`

The API will be available at http://localhost:8081/api/users.

## Server-Sent Events

The example also exposes an SSE stream at `GET /api/sse/countdown`:

```bash
curl -N http://localhost:8081/api/sse/countdown
```

This returns a `text/event-stream` response with a `countdown` event every 250ms followed by a final `done` event.

> **Note:** SSE connections are long-lived. If `request-timeout` is enabled in your config, the global timeout layer will terminate the stream, so leave it disabled or set a generous timeout for routes that stream events.
