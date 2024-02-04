use crate::models::*;
use crate::schema::tasks;
use diesel::internal::derives::multiconnection::chrono;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

pub enum StatusError {
    DieselError(diesel::result::Error),
    StaleEntryError,
}

pub enum StatusResult {
    Ok(Task),
    Err(StatusError),
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create(job_type: String, start_job_at: chrono::NaiveDateTime) -> Task {
    let connection = &mut establish_connection();
    let new_task = NewTask {
        id: uuid::Uuid::new_v4(),
        job_type: &job_type,
        status: &"pending",
        start_job_at: &start_job_at,
    };
    diesel::insert_into(tasks::table)
        .values(&new_task)
        .returning(Task::as_returning())
        .get_result(connection)
        .expect("Error saving new task")
}

pub fn get_by_id(uuid: uuid::Uuid) -> Result<Option<Task>, diesel::result::Error> {
    use crate::schema::tasks::dsl::*;
    let connection = &mut establish_connection();
    tasks
        .find(uuid)
        .select(Task::as_select())
        .first(connection)
        .optional()
}

pub fn start(task: &Task) -> StatusResult {
    update_status(task, "running".to_string())
}

pub fn complete(task: &Task) -> StatusResult {
    update_status(task, "complete".to_string())
}

pub fn delete(task: &Task) -> Result<Task, diesel::result::Error> {
    use crate::schema::tasks::dsl::*;
    let connection = &mut establish_connection();
    diesel::delete(tasks.find(task.id)).get_result(connection)
}

pub fn terminated(task: &Task) -> StatusResult {
    update_status(task, "terminated".to_string())
}

fn get_stale_tasks() -> Vec<Task> {
    use crate::schema::tasks::dsl::*;
    let connection = &mut establish_connection();
    let list = tasks
        .filter(status.eq("running"))
        .select(Task::as_select())
        .load(connection)
        .expect("Error loading tasks");
    let mut stale = vec![];
    for t in list {
        if t.start_job_at < chrono::Utc::now().naive_utc() - chrono::Duration::minutes(1) {
            stale.push(t);
        }
    }
    stale
}

pub fn reset_stale_tasks() {
    let stale = get_stale_tasks();
    for t in stale {
        update_status(&t, "pending".to_string());
    }
}

fn update_status(task: &Task, s: String) -> StatusResult {
    use crate::schema::tasks::dsl::*;
    let connection = &mut establish_connection();

    let update = TaskStatusUpdate {
        status: s.as_str(),
        lock_version: task.lock_version + 1,
    };

    let r = diesel::update(tasks.find(task.id))
        .filter(lock_version.eq(task.lock_version))
        .set(&update)
        .get_result(connection);

    match r {
        std::result::Result::Ok(task) => StatusResult::Ok(task),
        _ => StatusResult::Err(StatusError::StaleEntryError),
    }
}

pub fn get_pending_tasks() -> Vec<Task> {
    use crate::schema::tasks::dsl::*;
    let connection = &mut establish_connection();
    tasks
        .filter(
            status
                .eq("pending")
                .and(start_job_at.le(chrono::Utc::now().naive_utc())),
        )
        .select(Task::as_select())
        .load(connection)
        .expect("Error loading tasks")
}

pub fn get_completed_tasks() -> Vec<Task> {
    get_tasks_by_status("completed")
}

pub fn get_running_tasks() -> Vec<Task> {
    get_tasks_by_status("running")
}

fn get_tasks_by_status(s: &str) -> Vec<Task> {
    use crate::schema::tasks::dsl::*;
    let connection = &mut establish_connection();
    tasks
        .filter(status.eq(s))
        .select(Task::as_select())
        .limit(20)
        .load(connection)
        .expect("Error loading tasks")
}

pub fn get_tasks_by_status_and_type(stat: Option<String>, jtype: Option<String>) -> Vec<Task> {
    use crate::schema::tasks::dsl::*;
    let connection = &mut establish_connection();
    match (stat, jtype) {
        (Some(s), Some(jt)) => tasks
            .filter(status.eq(s).and(job_type.eq(jt)))
            .select(Task::as_select())
            .load(connection)
            .expect("Error loading tasks"),
        (Some(s), None) => tasks
            .filter(status.eq(s))
            .select(Task::as_select())
            .load(connection)
            .expect("Error loading tasks"),
        (None, Some(jt)) => tasks
            .filter(job_type.eq(jt))
            .select(Task::as_select())
            .load(connection)
            .expect("Error loading tasks"),
        (None, None) => tasks
            .select(Task::as_select())
            .load(connection)
            .expect("Error loading tasks"),
    }
}

pub fn delete_all() {
    use crate::schema::tasks::dsl::*;
    let connection = &mut establish_connection();
    diesel::delete(tasks)
        .execute(connection)
        .expect("Error deleting tasks");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn should_create_task() {
        let task = create("test".to_string(), chrono::Utc::now().naive_utc());
        assert_eq!(task.job_type, "test");
        assert_eq!(task.status, "pending");
        assert_eq!(task.start_job_at, task.start_job_at);
        assert!(task.number > 0);
    }

    #[test]
    pub fn should_get_task_by_id() {
        let task = create("test".to_string(), chrono::Utc::now().naive_utc());
        println!("task: {:?}", task.id);
        let task2 = get_by_id(task.id).unwrap().unwrap();
        assert_eq!(task.id, task2.id);
    }

    #[test]
    pub fn should_set_status_to_running() {
        let task = create("test".to_string(), chrono::Utc::now().naive_utc());
        let task2 = start(&task);

        match task2 {
            StatusResult::Ok(t) => {
                assert_eq!(t.status, "running");
            }
            _ => panic!("Error setting status to running"),
        }
    }

    #[test]
    pub fn should_return_error_if_task_is_stale() {
        let task = create("test".to_string(), chrono::Utc::now().naive_utc());
        start(&task);
        let task3 = start(&task);
        match task3 {
            StatusResult::Err(StatusError::StaleEntryError) => {
                assert!(true);
            }
            _ => panic!("Error setting status to running"),
        }
    }

    #[test]
    pub fn should_set_status_to_complete() {
        let task = create("test".to_string(), chrono::Utc::now().naive_utc());
        let task2 = complete(&task);
        match task2 {
            StatusResult::Ok(t) => {
                assert_eq!(t.status, "complete");
            }
            _ => panic!("Error setting status to complete"),
        }
    }

    #[test]
    pub fn should_delete_task() {
        let task = create("test".to_string(), chrono::Utc::now().naive_utc());
        let _ = delete(&task);
        let task3 = get_by_id(task.id);
        match task3 {
            Ok(None) => {
                assert!(true);
            }
            _ => panic!("Error deleting task"),
        }
    }

    #[test]
    pub fn should_get_pending_tasks() {
        delete_all();
        create("test".to_string(), chrono::Utc::now().naive_utc());
        create("test2".to_string(), chrono::Utc::now().naive_utc());
        let task = create("test3".to_string(), chrono::Utc::now().naive_utc());
        start(&task);
        let tasks = get_pending_tasks();
        for t in tasks {
            assert_eq!(t.status, "pending");
        }
    }
}
