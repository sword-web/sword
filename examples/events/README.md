# Sword Events Example

One application that publishes and consumes events through **two transports at
the same time**: in-memory for events that stay inside the process, and NATS for
events that leave it.

A controller can mix transports. When it does, the transport is declared per
handler instead of on the controller:

```rust
#[controller(kind = Controller::Events)]
pub struct OrderEventsHandler;

impl OrderEventsHandler {
    #[subscribe(key = "order.created", transport = EventTransport::Memory)]
    async fn on_created(&self, event: &OrderCreated) -> EventHandlerResult<()> {
        // ...
        Ok(())
    }

    #[subscribe(key = "order.shipped", transport = EventTransport::Nats)]
    async fn on_shipped(&self, event: &OrderShipped) -> EventHandlerResult<()> {
        // ...
        Ok(())
    }
}
```

## Run

The NATS transport needs a broker. Start one first:

```bash
docker run -d --rm --name nats -p 4222:4222 nats:2
```

Then run the example:

```bash
cd examples/events
cargo run
```

Without a broker the application fails at startup with a diagnostic that points
to the NATS connection, which is the expected behaviour when the transport is
configured.

## Try it

```bash
# Published to the in-memory transport, handled inside the process
curl -X POST http://127.0.0.1:8085/api/orders \
  -H 'content-type: application/json' \
  -d '{"total": 42}'

# Published to NATS, handled through the broker
curl -X POST http://127.0.0.1:8085/api/orders/ship \
  -H 'content-type: application/json' \
  -d '{"order_id": "order-1", "tracking": "TRACK-9"}'
```

Expected output:

```text
INFO  order.created handled in memory  order_id: order-1  total: 42
INFO  order.shipped handled through NATS  order_id: order-1  tracking: TRACK-9
INFO  order.* received from NATS  subject: order.shipped
```

## What each transport adds

| | In-memory | NATS |
| --- | --- | --- |
| Leaves the process | No | Yes |
| External service | No | Broker |
| Wildcards (`order.*`) | No | Yes |
| Queue groups | No | Yes |
| Delivery | In-process only | At-most-once (core NATS) |

The `order.*` handler in this example only receives events published to NATS.
That is why `order.shipped` shows up twice (the concrete handler and the
wildcard one) and `order.created` only once.
