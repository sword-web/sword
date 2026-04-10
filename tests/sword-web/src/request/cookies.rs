use sword::prelude::*;
use sword::web::*;

use crate::application_builder;
use crate::test_server;

#[derive(Interceptor)]
struct SetCookieMw {}

impl OnRequest for SetCookieMw {
    async fn on_request(&self, req: Request) -> WebInterceptorResult {
        let cookies = req.cookies()?;

        let cookie = CookieBuilder::new("session_id", "abc123")
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .build();

        cookies.add(cookie);

        req.next().await
    }
}

#[controller(kind = Controller::Web, path = "/cookies")]
struct CookieController {}

impl CookieController {
    #[get("/set")]
    async fn set_cookie(&self, req: Request) -> WebResult {
        let cookies = req.cookies()?;

        let cookie = CookieBuilder::new("username", "sword_user")
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .build();

        cookies.add(cookie);

        Ok(JsonResponse::Ok())
    }

    #[get("/with_middleware")]
    #[interceptor(SetCookieMw)]
    async fn with_middleware(&self, req: Request) -> WebResult {
        let cookies = req.cookies()?;

        let session_cookie = cookies
            .get("session_id")
            .ok_or(JsonResponse::Unauthorized().message("Session cookie not found"))?;

        Ok(JsonResponse::Ok().message(format!("Session ID: {}", session_cookie.value())))
    }
}

struct CookieModule;

impl Module for CookieModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<CookieController>();
    }
}

#[tokio::test]
async fn test_set_cookie() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let app = application_builder().with_module::<CookieModule>().build();
    let server = test_server(app);

    let response = server.get("/cookies/set").await;
    assert_eq!(response.status_code(), 200);

    let cookies = response.cookies();

    let username_cookie = cookies
        .iter()
        .find(|cookie| cookie.name() == "username")
        .expect("Cookie 'username' not found");

    assert_eq!(username_cookie.value(), "sword_user");

    assert_eq!(username_cookie.path(), Some("/"));
    assert!(username_cookie.http_only().unwrap_or(false));
    assert_eq!(username_cookie.same_site(), Some(SameSite::Lax));

    Ok(())
}

#[tokio::test]
async fn test_with_middleware() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let app = application_builder().with_module::<CookieModule>().build();
    let server = test_server(app);

    let response = server.get("/cookies/with_middleware").await;
    assert_eq!(response.status_code(), 200);

    let cookies = response.cookies();

    let session_cookie = cookies
        .iter()
        .find(|cookie| cookie.name() == "session_id")
        .expect("Cookie 'session_id' not found");

    assert_eq!(session_cookie.value(), "abc123");

    assert_eq!(session_cookie.path(), Some("/"));
    assert!(session_cookie.http_only().unwrap_or(false));
    assert_eq!(session_cookie.same_site(), Some(SameSite::Lax));

    Ok(())
}
