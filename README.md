<div align="center">
  <img src="https://sword-web.github.io/logo-new.png" width="500" />
</div>

Server-side application framework for Rust, built on top of the Tokio ecosystem.
Designed to write code with less boilerplate, more simplicity, and cleaner patterns.
It leverages the Tokio ecosystem to give you performance with great DX.

> Sword is under active development; expect breaking changes.

## Features

- **Multiple application types** - Web, Socket.IO, gRPC, events, and jobs, all able to run together
- **Macro-based approach** - Clean, intuitive, and declarative macros for defining the application structure
- **Built-in validation** - Support for the `validator` crate and an extensible validation system
- **Dependency Injection** - Built-in DI support with declarative macros
- **TOML configuration loader** - Built-in support for loading configuration from TOML files

## Web Controller Example

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

            Err(AppError::UserConflictError("username", &user.username))?;
        }

        self.users.save(&user).await?;

        Ok(JsonResponse::Created().message("User created").data(user))
    }
}
```

## Full Examples

- [Controllers API](./examples/web)
- [Socket.IO Controllers Chat](./examples/socketio)
- [Interceptors (Web + Socket.IO controllers)](./examples/interceptors)
- [gRPC Application with error handling](./examples/grpc)

## Cargo Features

- `web`: enables the web application type
- `socketio`: enables Socket.IO controllers for web applications (implies `web`)
- `grpc`: enables the gRPC application type
- `events-in-memory`: enables in-memory event publishing and handling
- `jobs`: enables cron/repeated/once job scheduling
- `web-multipart`: enables `multipart/form-data` support in web controllers
- `web-swagger-ui`: enables Swagger UI for OpenAPI specs
- `grpc-reflection`: enables gRPC reflection
- `grpc-error-details`: enables detailed gRPC errors with `GrpcStatus`
- `validation-validator`: enables input validation with the `validator` crate
- `hot-reload`: enables hot reload during development

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for more details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request. See [CONTRIBUTING.md](./CONTRIBUTING.md) for more details.
