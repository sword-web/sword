<div align="center">
  <img src="https://sword-web.github.io/logo-new.png" width="500" />
</div>

Structured web framework for Rust built on top of the Tokio ecosystem.
Designed to build server applications with less boilerplate and more simplicity.
It takes advantage of the tokio ecosystem to bring you performance with nice DX.

> Sword is in active development, expect breaking changes.

## Features

- **Web Application Type** - Axum-based application stack with web and Socket.IO controllers
- **gRPC Application Type** - Tonic based application with built-in error handling based on `thiserror`
- **Macro-based routing** - Clean and intuitive route definitions
- **JSON-first design** - Built with JSON formats as priority
- **Built-in validation** - Support `validator` crate and extensible validation system
- **HTTP response standardization** - Consistent response formats out of the box
- **Dependency Injection** - Built-in DI support with declarative macros
- **Tower layers + interceptors** - Compose global layers and typed interceptors
- **TOML config loader** - Built-in support for loading configuration from TOML files

## Controller Example

```rust
use sword::prelude::*;
use sword::web::*;

#[controller(kind = Controller::Web, path = "/users")]
pub struct UsersController {
    hasher: Arc<Hasher>,
    users: Arc<UserRepository>,
}

impl UsersController {
    #[get("/")]
    async fn get_users(&self, req: Request) -> WebResult {
        let data = self.users.find_all().await?;

        Ok(JsonResponse::Ok().data(data).request_id(req.id()))
    }

    #[post("/")]
    async fn create_user(&self, req: Request) -> WebResult {
        let body = req.body_validator::<CreateUserDto>()?;
        let user = User::new(body.username, self.hasher.hash(&body.password)?);

        if self.users.find_by_username(&user.username).await?.is_some() {
            tracing::error!(
                "Attempt to create user with existing username: {}",
                user.username
            );

            return Err(AppError::UserConflictError("username", &user.username))?;
        }

        self.users.save(&user).await?;

        Ok(JsonResponse::Created().message("User created").data(user))
    }
}
```

## Full Examples

- [Controllers API](./examples/web)
- [SocketIO Controllers Chat](./examples/socketio)
- [Interceptors (Web + Socket.IO controllers)](./examples/interceptors)
- [gRPC Application with error handling](./examples/grpc)

## Application Type Features

- `web`: enables the web application type
- `grpc`: enables the gRPC application type
- `socketio`: enables Socket.IO controllers for web applications

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for more details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request. See [CONTRIBUTING.md](./CONTRIBUTING.md) for more details.
