use diesel::internal::derives::multiconnection::chrono;
use diesel::prelude::*;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::tasks)]
pub struct NewTask<'a> {
    pub id: Uuid,
    pub job_type: &'a str,
    pub status: &'a str,
    pub start_job_at: &'a chrono::NaiveDateTime,
}

#[derive(Queryable, AsChangeset)]
#[diesel(table_name = crate::schema::tasks)]
pub struct TaskStatusUpdate<'a> {
    pub lock_version: i32,
    pub status: &'a str,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Task {
    pub id: Uuid,
    pub job_type: String,
    pub number: i32,
    pub status: String,
    pub result: Option<String>,
    pub lock_version: i32,
    pub start_job_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

pub enum TaskType {
    Foo,
    Bar,
    Baz,
}

impl FromStr for TaskType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "foo" => Ok(TaskType::Foo),
            "bar" => Ok(TaskType::Bar),
            "baz" => Ok(TaskType::Baz),
            _ => Err(()),
        }
    }
}

impl Task {
    pub fn get_type(&self) -> TaskType {
        match self.job_type.as_str() {
            "foo" => TaskType::Foo,
            "bar" => TaskType::Bar,
            "baz" => TaskType::Baz,
            _ => panic!("Unknown task type"),
        }
    }
}
