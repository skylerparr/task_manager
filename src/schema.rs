// @generated automatically by Diesel CLI.

diesel::table! {
    tasks (id) {
        id -> Uuid,
        job_type -> Varchar,
        status -> Varchar,
        result -> Nullable<Varchar>,
        lock_version -> Int4,
        start_job_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
