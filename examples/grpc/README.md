# Sword gRPC Example

Minimal gRPC users CRUD example for Sword using `grpc-controllers` and an in-memory store.

## Run

```bash
cargo run -p grpc-controllers
```

## Available services

- `users.UserService`
- `grpc.health.v1.Health`
- `grpc.reflection.v1.ServerReflection` (enabled by `grpc-reflection` feature)

## Available RPC methods

- `users.UserService/ListUsers`
- `users.UserService/StreamUsers`
- `users.UserService/CreateUser`
- `users.UserService/GetUser`
- `users.UserService/UpdateUser`
- `users.UserService/DeleteUser`
- `grpc.health.v1.Health/Check`

## Notes

- UserService methods expect `authorization` metadata.
- Server default address is `127.0.0.1:50051`.
- If the binary is built with the `grpc-reflection` feature, `grpcurl list` includes health and users services.
- Reflection metadata is registered automatically by Sword from `build.rs` when generating `sword_descriptor_set.bin`.

## Richer errors (feature `grpc-error-details`)

With the `grpc-error-details` feature enabled, handlers can return errors that
implement the [gRPC Richer Error Model](https://grpc.io/docs/guides/error/)
using the `GrpcStatus` builder, chaining standard error details on any status
code:

```rust
return Err(
    GrpcStatus::InvalidArgument()
        .message("invalid request")
        .bad_request("username", "username cannot be empty")
        .into(),
);
```

Simple errors keep propagating with `?` through the `#[derive(GrpcError)]` type.
Clients can read the details back with `tonic_types::StatusExt`:

```rust
use sword::grpc::*;

let details = status.get_error_details();
if let Some(bad_request) = details.bad_request() {
    // ...
}
```
