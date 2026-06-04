use std::sync::Arc;

use axum_test::http::StatusCode;
use serde_json::Value;

use sword::prelude::*;
use sword::web::*;

use super::di::Database;
use crate::{application_builder, test_server};

#[sword::contract]
pub trait TaskRepository {
    async fn find_all(&self) -> Vec<Value>;
    async fn create(&self, task: Value);
}

#[injectable]
pub struct DatabaseTaskRepository {
    db: Database,
}

#[sword::contract]
impl TaskRepository for DatabaseTaskRepository {
    async fn find_all(&self) -> Vec<Value> {
        self.db.get_all("tasks").await.unwrap_or_default()
    }

    async fn create(&self, task: Value) {
        self.db.insert("tasks", task).await;
    }
}

#[controller(kind = Controller::Web, path = "/traits")]
pub struct TraitsController {
    repo: Arc<dyn TaskRepository>,
}

impl TraitsController {
    #[get("/")]
    async fn get_tasks(&self) -> JsonResponse {
        let data = self.repo.find_all().await;
        JsonResponse::Ok().data(data)
    }

    #[post("/")]
    async fn create_task(&self) -> JsonResponse {
        let total = self.repo.find_all().await.len();
        let task = serde_json::json!({
            "id": total + 1,
            "title": format!("Task {}", total + 1),
        });
        self.repo.create(task.clone()).await;
        JsonResponse::Created().message("Task created").data(task)
    }
}

pub struct TraitsModule;

impl Module for TraitsModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<DatabaseTaskRepository>();
    }

    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<TraitsController>();
    }
}

#[tokio::test]
async fn test_trait_di_get_tasks() {
    let app = application_builder()
        .with_provider(Database::new())
        .with_module::<TraitsModule>()
        .build();

    let server = test_server(app);
    let response = server.get("/traits").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: JsonResponseBody = response.json();

    assert!(body.success);
    assert_eq!(body.code, 200);
    assert_eq!(body.data, Some(serde_json::json!([])));
}

#[tokio::test]
async fn test_trait_di_create_task() {
    let app = application_builder()
        .with_provider(Database::new())
        .with_module::<TraitsModule>()
        .build();

    let server = test_server(app);
    let response = server.post("/traits").await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: JsonResponseBody = response.json();

    assert!(body.success);
    assert_eq!(body.code, 201);
    assert_eq!(body.message.as_ref(), "Task created");

    let task = body.data.unwrap();

    assert_eq!(task["id"], 1);
    assert_eq!(task["title"], "Task 1");
}
