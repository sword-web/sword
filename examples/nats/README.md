# Sword NATS Events Example

This example shows a web application that publishes events to NATS and an event
handler that consumes them from NATS.

## Flow

```text
POST /api/orders       ->  publish "order.created"  ->  NATS  ->  OrderEventsHandler (queue group)
POST /api/orders/ship  ->  publish "order.shipped"  ->  NATS  ->  OrderEventsHandler (wildcard)
```

Two handlers are registered:

- `on_created` subscribes to `order.created` with the queue group
  `orders-workers`. If more than one instance of the application is running,
  each message is processed by only one of them (load balancing).
- `on_order_event` subscribes to the wildcard `order.*` with no queue group.
  It receives every event whose subject starts with `order.` (fan-out to every
  instance) and logs the subject and the raw payload. A wildcard handler takes
  `&dyn Event`, so it can also branch on `event.key()` and recover the
  concrete type with `event.cast::<T>()` when needed.

## Running the example

1. Start NATS:

   ```bash
   docker compose -f compose.yml up -d
   ```

2. Run the application:

   ```bash
   cargo run
   ```

3. Publish an order:

   ```bash
   curl -X POST http://127.0.0.1:8085/api/orders \
     -H "Content-Type: application/json" \
     -d '{"total": 42}'
   ```

4. Ship the order:

    ```bash
    curl -X POST http://127.0.0.1:8085/api/orders/ship \
      -H "Content-Type: application/json" \
      -d '{"order_id": "order-1", "tracking": "TRK-123"}'
    ```

5. Watch the application logs for `order.created received from NATS` and
   `order event payload`.

## Queue groups

`group = "orders-workers"` makes the handler part of a NATS queue group. To see
the load balancing, run two instances and publish several orders: each message is
delivered to only one instance.

Remove `group` from `#[subscribe(...)]` to fan the event out to every instance
instead.

## Notes

- Core NATS pub/sub is **at-most-once**: if a handler fails, the message is not
  redelivered.
- JetStream (durability, acknowledgements and redelivery) is not used in this
  example yet.
