use crate::access;
use crate::models::*;
use actix_web::{get, post, web, HttpResponse, Responder, Result};
use diesel::internal::derives::multiconnection::chrono;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Serialize)]
struct CreateResponse {
    job_id: String,
    status: i32,
    message: String,
}

#[derive(Deserialize)]
struct CreateTaskBody {
    task_type: String,
    execution_time: i64,
}

#[post("/tasks/create")]
async fn create(req_body: String) -> Result<impl Responder> {
    match serde_json::from_str::<CreateTaskBody>(&req_body) {
        Ok(body) => {
            if body.execution_time < 0 {
                return Ok(web::Json(CreateResponse {
                    status: 400,
                    job_id: "".to_string(),
                    message: "Invalid execution time, tasks cannot start in the past!".to_string(),
                }));
            }
            let task_type = body.task_type.to_lowercase();
            match TaskType::from_str(&task_type) {
                Ok(_) => {
                    let task = access::create(
                        task_type,
                        (chrono::Utc::now() + chrono::Duration::seconds(body.execution_time))
                            .naive_utc(),
                    );
                    let response = CreateResponse {
                        status: 200,
                        job_id: task.id.to_string(),
                        message: "Task created".to_string(),
                    };
                    Ok(web::Json(response))
                }
                Err(_) => {
                    return Ok(web::Json(CreateResponse {
                        status: 400,
                        job_id: "".to_string(),
                        message: "Invalid task type".to_string(),
                    }));
                }
            }
        }
        Err(_) => Ok(web::Json(CreateResponse {
            status: 400,
            job_id: "".to_string(),
            message: "Invalid request body".to_string(),
        })),
    }
}

#[derive(Serialize)]
struct DeleteResponse {
    status: i32,
    message: String,
}

#[post("/tasks/delete/{id}")]
async fn delete(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    match Uuid::parse_str(&id) {
        Ok(uuid) => {
            let task = access::get_by_id(uuid);
            match task {
                Ok(Some(t)) => {
                    let result = access::delete(&t);
                    match result {
                        Ok(_) => HttpResponse::Ok().json(DeleteResponse {
                            status: 200,
                            message: "Task deleted".to_string(),
                        }),
                        Err(_) => HttpResponse::InternalServerError().body("Internal server error"),
                    }
                }
                Ok(None) => HttpResponse::NotFound().json(DeleteResponse {
                    status: 404,
                    message: "Task not found".to_string(),
                }),
                Err(_) => HttpResponse::InternalServerError().body("Internal server error"),
            }
        }
        Err(_) => HttpResponse::BadRequest().json(DeleteResponse {
            status: 400,
            message: "Invalid id".to_string(),
        }),
    }
}

#[derive(Serialize)]
struct TaskResponse {
    id: String,
    job_type: String,
    status: String,
    execution_time: String,
}

#[get("/tasks/{id}")]
async fn get(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    match Uuid::parse_str(&id) {
        Ok(uuid) => {
            let task = access::get_by_id(uuid);
            match task {
                Ok(Some(task)) => {
                    let response = TaskResponse {
                        id: task.id.to_string(),
                        job_type: task.job_type.to_string(),
                        status: task.status.to_string(),
                        execution_time: task.start_job_at.to_string(),
                    };
                    HttpResponse::Ok().json(response)
                }
                Ok(None) => HttpResponse::NotFound()
                    .body("{\"status\": 404, \"message\": \"Task not found\"}"),
                Err(_) => HttpResponse::InternalServerError().body("Internal server error"),
            }
        }
        Err(_) => HttpResponse::BadRequest().body("{\"status\": 400, \"message\": \"Invalid id\"}"),
    }
}

#[derive(Deserialize)]
struct IndexQuery {
    status: Option<String>,
    task_type: Option<String>,
}

#[derive(Serialize)]
struct IndexResponse {
    tasks: Vec<TaskResponse>,
    next_key: Option<String>,
}

#[get("/tasks")]
async fn index(req_body: String) -> impl Responder {
    match serde_json::from_str::<IndexQuery>(&req_body) {
        Ok(query) => {
            let tasks = access::get_tasks_by_status_and_type(query.status, query.task_type);
            let tasks: Vec<TaskResponse> = tasks
                .iter()
                .map(|task| TaskResponse {
                    id: task.id.to_string(),
                    job_type: task.job_type.to_string(),
                    status: task.status.to_string(),
                    execution_time: task.start_job_at.to_string(),
                })
                .collect();

            let response = IndexResponse {
                tasks,
                next_key: None,
            };
            HttpResponse::Ok().json(response)
        }
        Err(_) => HttpResponse::BadRequest()
            .body("{\"status\": 400, \"message\": \"Invalid request body\"}"),
    }
}
