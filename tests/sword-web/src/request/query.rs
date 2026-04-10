use serde::{Deserialize, Serialize};
use sword::prelude::*;
use sword::web::*;
use validator::Validate;

use crate::application_builder;
use crate::test_server;

#[derive(Debug, Deserialize, Serialize)]
struct QueryData {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
struct ValidableQueryData {
    #[validate(range(message = "Page must be between 1 and 1000", min = 1, max = 1000))]
    page: u32,
    #[validate(range(message = "Limit must be between 1 and 100", min = 1, max = 100))]
    limit: u32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct OptionalQueryData {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Validate)]
struct DefaultValidableQueryData {
    #[validate(range(message = "Page must be between 1 and 1000", min = 1, max = 1000))]
    page: Option<u32>,

    #[validate(range(message = "Limit must be between 1 and 100", min = 1, max = 100))]
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ComplexQueryData {
    page: Option<u32>,
    limit: Option<i32>,
    price: Option<f64>,

    search: Option<String>,
    category: Option<String>,

    active: Option<bool>,

    user_name: Option<String>,
    email_filter: Option<String>,
}

#[controller(kind = Controller::Web, path = "/users")]
pub struct UserController {}

impl UserController {
    #[get("/simple-query")]
    async fn get_users(&self, req: Request) -> WebResult {
        let query: Option<QueryData> = req.query()?;

        Ok(JsonResponse::Ok()
            .data(query)
            .message("Users retrieved successfully"))
    }

    #[get("/validate-query")]
    async fn get_users_with_validation(&self, req: Request) -> WebResult {
        let query: Option<ValidableQueryData> = req.query_validator()?;

        Ok(JsonResponse::Ok()
            .data(query)
            .message("Users retrieved successfully with validation"))
    }

    #[get("/ergonomic-optional-query")]
    async fn get_users_with_ergonomic_query(&self, req: Request) -> WebResult {
        let query: OptionalQueryData = req.query()?.unwrap_or_default();

        Ok(JsonResponse::Ok()
            .data(query)
            .message("Users retrieved with ergonomic optional query"))
    }

    #[get("/ergonomic-validated-optional-query")]
    async fn get_users_with_ergonomic_validated_optional_query(&self, req: Request) -> WebResult {
        let query: DefaultValidableQueryData = req.query_validator()?.unwrap_or_default();

        Ok(JsonResponse::Ok()
            .data(query)
            .message("Users retrieved with ergonomic validated optional query"))
    }

    #[get("/complex-query")]
    async fn get_users_with_complex_query(&self, req: Request) -> WebResult {
        let query: Option<ComplexQueryData> = req.query()?;

        Ok(JsonResponse::Ok()
            .data(query)
            .message("Users retrieved with complex query parameters"))
    }

    #[get("/pattern-match-query")]
    async fn get_users_with_pattern_match(&self, req: Request) -> WebResult {
        match req.query::<OptionalQueryData>()? {
            Some(query) => Ok(JsonResponse::Ok()
                .data(query)
                .message("Users retrieved with query parameters")),
            None => Ok(JsonResponse::Ok()
                .data(OptionalQueryData::default())
                .message("Users retrieved with default parameters")),
        }
    }
}

struct UserModule;

impl Module for UserModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<UserController>();
    }
}

#[tokio::test]
async fn unvalidated_query_test() {
    let app = application_builder().with_module::<UserModule>().build();
    let app = test_server(app);

    let response = app.get("/users/simple-query?page=1&limit=5").await;
    let json = response.json::<JsonResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.is_some());

    let data = json.data.unwrap();

    assert!(data.get("page").is_some());
    assert!(data.get("limit").is_some());

    assert_eq!(data.get("page").unwrap(), "1".parse::<u32>().unwrap());
    assert_eq!(data.get("limit").unwrap(), "5".parse::<u32>().unwrap());
}

#[tokio::test]
async fn validated_query_test() {
    let app = application_builder().with_module::<UserModule>().build();
    let app = test_server(app);

    let response = app.get("/users/validate-query?page=1&limit=5").await;

    let json = response.json::<JsonResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.is_some());

    let data = json.data.unwrap();

    assert!(data.get("page").is_some());
    assert!(data.get("limit").is_some());

    assert_eq!(data.get("page").unwrap(), "1".parse::<u32>().unwrap());
    assert_eq!(data.get("limit").unwrap(), "5".parse::<u32>().unwrap());
}

#[tokio::test]
async fn validated_query_error_test_validator() {
    let app = application_builder().with_module::<UserModule>().build();
    let app = test_server(app);

    let response = app.get("/users/validate-query?page=1001&limit=5").await;

    let json = response.json::<JsonResponseBody>();

    assert_eq!(400_u16, response.status_code().as_u16());
    assert!(json.errors.is_some());

    let data = json.errors.unwrap();
    let page_errors = data.get("page").unwrap().as_array().unwrap();

    assert_eq!(page_errors.len(), 1);

    let error = &page_errors[0];

    assert!(error.get("code").is_some());
    assert_eq!(error.get("code").unwrap(), "range");
    assert!(error.get("message").is_some());
    assert_eq!(
        error.get("message").unwrap(),
        "Page must be between 1 and 1000"
    );
}

#[tokio::test]
async fn ergonomic_optional_query_with_params_test() {
    let app = application_builder().with_module::<UserModule>().build();
    let app = test_server(app);

    let response = app
        .get("/users/ergonomic-optional-query?page=1&limit=5")
        .await;

    let json = response.json::<JsonResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.is_some());

    let data = json.data.unwrap();

    assert!(data.get("page").is_some());
    assert!(data.get("limit").is_some());

    assert_eq!(data.get("page").unwrap(), "1".parse::<u32>().unwrap());
    assert_eq!(data.get("limit").unwrap(), "5".parse::<u32>().unwrap());
}

#[tokio::test]
async fn ergonomic_optional_query_without_params_test() {
    let app = application_builder().with_module::<UserModule>().build();
    let app = test_server(app);

    let response = app.get("/users/ergonomic-optional-query").await;
    let json = response.json::<JsonResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.is_some());

    let data = json.data.unwrap();

    assert!(data.get("page").is_some());
    assert!(data.get("limit").is_some());
    assert!(data.get("page").unwrap().is_null());
    assert!(data.get("limit").unwrap().is_null());
}

#[tokio::test]
async fn ergonomic_validated_optional_query_with_params_test() {
    let app = application_builder().with_module::<UserModule>().build();
    let app = test_server(app);

    let response = app
        .get("/users/ergonomic-validated-optional-query?page=1&limit=5")
        .await;

    let json = response.json::<JsonResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.is_some());

    let data = json.data.unwrap();

    assert!(data.get("page").is_some());
    assert!(data.get("limit").is_some());
    assert_eq!(data.get("page").unwrap(), "1".parse::<u32>().unwrap());
    assert_eq!(data.get("limit").unwrap(), "5".parse::<u32>().unwrap());
}

#[tokio::test]
async fn ergonomic_validated_optional_query_without_params_test() {
    let app = application_builder().with_module::<UserModule>().build();
    let app = test_server(app);

    let response = app.get("/users/ergonomic-validated-optional-query").await;
    let json = response.json::<JsonResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.is_some());

    let data = json.data.unwrap();

    assert!(data.get("page").is_some());
    assert!(data.get("limit").is_some());
}

#[tokio::test]
async fn pattern_match_query_with_params_test() {
    let app = application_builder().with_module::<UserModule>().build();
    let app = test_server(app);

    let response = app.get("/users/pattern-match-query?page=1&limit=5").await;

    let json = response.json::<JsonResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert_eq!(
        json.message.as_ref(),
        "Users retrieved with query parameters"
    );
}

#[tokio::test]
async fn pattern_match_query_without_params_test() {
    let app = application_builder().with_module::<UserModule>().build();
    let app = test_server(app);

    let response = app.get("/users/pattern-match-query").await;

    let json = response.json::<JsonResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert_eq!(
        json.message.as_ref(),
        "Users retrieved with default parameters"
    );
}

#[tokio::test]
async fn complex_encoded_query_test() {
    let app = application_builder().with_module::<UserModule>().build();
    let app = test_server(app);

    let encoded_url = "/users/complex-query?page=1&limit=-10&price=99.99&search=hello%20world&category=electronics%26gadgets&active=true&user_name=john%2Bdoe&email_filter=test%40example.com";

    let response = app.get(encoded_url).await;
    let json = response.json::<JsonResponseBody>();

    assert_eq!(200_u16, response.status_code().as_u16());
    assert!(json.data.is_some());

    let data = &json.data.unwrap();

    if let Some(page) = data.get("page") {
        assert_eq!(page, &1u32);
    }
    if let Some(limit) = data.get("limit") {
        assert_eq!(limit, &-10i32);
    }
    if let Some(price) = data.get("price") {
        assert_eq!(price, &99.99f64);
    }

    if let Some(search) = data.get("search") {
        assert_eq!(search, "hello world");
    }
    if let Some(category) = data.get("category") {
        assert_eq!(category, "electronics&gadgets");
    }
    if let Some(user_name) = data.get("user_name") {
        assert_eq!(user_name, "john+doe");
    }
    if let Some(email_filter) = data.get("email_filter") {
        assert_eq!(email_filter, "test@example.com");
    }

    if let Some(active) = data.get("active") {
        assert_eq!(active, &true);
    }
}
