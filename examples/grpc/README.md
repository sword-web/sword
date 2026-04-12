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
