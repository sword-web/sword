use axum_test::multipart::{MultipartForm, Part};
use std::fs;
use sword::prelude::*;
use sword::web::*;

use crate::{application_builder, test_server, utils::TempFile};

#[controller(kind = Controller::Web, path = "/")]
struct TestController {}

impl TestController {
    #[post("/multipart")]
    async fn hello(&self, req: Request) -> WebResult {
        let mut fields = vec![];
        let mut multipart = req.multipart().await?;

        while let Some(field) = multipart.next_field().await.map_err(|e| {
            eprintln!("Error reading multipart field: {}", e);
            JsonResponse::BadRequest().message("Failed to read multipart field")
        })? {
            let name = field.name().unwrap_or("Unnamed").to_string();
            let file_name = field.file_name().unwrap_or("No file name").to_string();

            let content_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or("No content type".to_string());

            let data = field.bytes().await.unwrap();

            fields.push(serde_json::json!({
                "name": name,
                "file_name": file_name,
                "content_type": content_type,
                "data_length": data.len(),
            }));
        }

        Ok(JsonResponse::Ok().data(fields).message("Hello, Multipart!"))
    }
}

struct TestModule;

impl Module for TestModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<TestController>();
    }
}

#[tokio::test]
async fn exceed_limit() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let app = application_builder().with_module::<TestModule>().build();
    let app = test_server(app);

    let temp_file = TempFile::with_size(1024 * 1024 * 2); // 2 MB
    let bytes = fs::read(&temp_file.path).expect("Failed to read test file");

    let part = Part::bytes(bytes)
        .file_name("large_test_file.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_part("file", part);

    let response = app.post("/multipart").multipart(form).await;
    let json = response.json::<JsonResponseBody>();

    assert_eq!(response.status_code(), 413);

    assert_eq!(
        json.message,
        "The request body exceeds the maximum allowed size by the server".into()
    );

    Ok(())
}

/// Tests that a file exactly at the body limit (considering multipart overhead) is accepted.
/// The effective limit is ~975KB due to multipart headers/boundaries overhead.
#[tokio::test]
async fn body_limit_exactly_at_limit() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let temp_file = TempFile::with_size(975 * 1024);
    let bytes = fs::read(&temp_file.path).expect("Failed to read test file");

    let part = Part::bytes(bytes)
        .file_name("exactly_at_limit_file.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_part("file", part);

    let response = test.post("/multipart").multipart(form).await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<JsonResponseBody>();
    assert_eq!(json.message, "Hello, Multipart!".into());

    Ok(())
}

#[tokio::test]
async fn body_limit_just_under_limit() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let temp_file = TempFile::with_size(970 * 1024);
    let bytes = fs::read(&temp_file.path).expect("Failed to read test file");

    let part = Part::bytes(bytes)
        .file_name("just_under_limit_file.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_part("file", part);

    let response = test.post("/multipart").multipart(form).await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<JsonResponseBody>();
    assert_eq!(json.message, "Hello, Multipart!".into());

    Ok(())
}

#[tokio::test]
async fn body_limit_just_over_limit() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let temp_file = TempFile::with_size(976 * 1024);
    let bytes = fs::read(&temp_file.path).expect("Failed to read test file");

    let part = Part::bytes(bytes)
        .file_name("just_over_limit_file.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_part("file", part);

    let response = test.post("/multipart").multipart(form).await;

    assert_eq!(response.status_code(), 413);
    let json = response.json::<JsonResponseBody>();

    assert_eq!(
        json.message,
        "The request body exceeds the maximum allowed size by the server".into()
    );

    Ok(())
}

#[tokio::test]
async fn body_limit_multiple_fields_exceed_limit() -> Result<(), Box<dyn std::error::Error>> {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    // Create multiple smaller files that together exceed the limit
    let temp_file1 = TempFile::with_size(700 * 1024); // 700 KB
    let temp_file2 = TempFile::with_size(700 * 1024); // 700 KB
    // Total: ~1.4 MB plus multipart overhead should exceed 1MB limit

    let bytes1 = fs::read(&temp_file1.path).expect("Failed to read test file 1");
    let bytes2 = fs::read(&temp_file2.path).expect("Failed to read test file 2");

    let part1 = Part::bytes(bytes1)
        .file_name("file1.txt")
        .mime_type("text/plain");

    let part2 = Part::bytes(bytes2)
        .file_name("file2.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_part("file1", part1)
        .add_part("file2", part2);

    let response = test.post("/multipart").multipart(form).await;

    assert_eq!(response.status_code(), 413);
    let json = response.json::<JsonResponseBody>();

    assert_eq!(
        json.message,
        "The request body exceeds the maximum allowed size by the server".into()
    );

    Ok(())
}

#[tokio::test]
async fn body_limit_small_fields_within_limit() -> Result<(), Box<dyn std::error::Error>> {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    // Create multiple smaller files that together stay within the limit
    let temp_file1 = TempFile::with_size(300 * 1024); // 300 KB
    let temp_file2 = TempFile::with_size(300 * 1024); // 300 KB
    // Total: ~600 KB plus multipart overhead should be within 1MB limit

    let bytes1 = fs::read(&temp_file1.path).expect("Failed to read test file 1");
    let bytes2 = fs::read(&temp_file2.path).expect("Failed to read test file 2");

    let part1 = Part::bytes(bytes1)
        .file_name("file1.txt")
        .mime_type("text/plain");

    let part2 = Part::bytes(bytes2)
        .file_name("file2.txt")
        .mime_type("text/plain");

    let form = MultipartForm::new()
        .add_text("field1", "value1")
        .add_text("field2", "value2")
        .add_part("file1", part1)
        .add_part("file2", part2);

    let response = test.post("/multipart").multipart(form).await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<JsonResponseBody>();

    assert_eq!(json.message, "Hello, Multipart!".into());

    Ok(())
}
