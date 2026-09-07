# Sword gRPC Example

Minimal gRPC users CRUD example for Sword using `grpc-controllers` and an in-memory store.

## Run

```bash
cargo run -p sword-grpc-example
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

## Testing with grpcurl

Start the server (see [Run](#run)) and use [grpcurl](https://github.com/fullstorydev/grpcurl).
Reflection is enabled, so services are discovered automatically.

List services and methods:

```bash
grpcurl -plaintext 127.0.0.1:50051 list
grpcurl -plaintext 127.0.0.1:50051 list users.UserService
```

`users.UserService` methods require an `authorization` metadata value — replace `<token>` (any value works):

```bash
grpcurl -plaintext -H "authorization: <token>" -d '{}' 127.0.0.1:50051 users.UserService/ListUsers
grpcurl -plaintext -H "authorization: <token>" \
  -d '{"username": "<username>", "password": "<password>"}' \
  127.0.0.1:50051 users.UserService/CreateUser
grpcurl -plaintext -H "authorization: <token>" -d '{"id": "<id>"}' 127.0.0.1:50051 users.UserService/GetUser
grpcurl -plaintext -H "authorization: <token>" -d '{}' 127.0.0.1:50051 users.UserService/StreamUsers
```

Health check (no authorization required):

```bash
grpcurl -plaintext -d '{}' 127.0.0.1:50051 grpc.health.v1.Health/Check
```

## Notes

- UserService methods expect `authorization` metadata.
- Server default address is `127.0.0.1:50051`.
- If the binary is built with the `grpc-reflection` feature, `grpcurl list` includes health and users services.
- Reflection metadata is registered automatically by Sword from `build.rs` when generating `sword_descriptor_set.bin`.
