## TaskManager 

Creates tasks that can be executed in the future.

## Build Instructions
- install rust
- install and run postgres
- install diesel_cli
  - `cargo install diesel_cli --no-default-features --features postgres`
- setup database
  - `diesel setup`
  - `diesel migration run`
- setup database connection
  - `echo "DATABASE_URL=postgres://username:password@localhost/task_manager" > .env`
- run tests
  - `cargo test`
- run the server
  - `cargo run`

## API
### Create a task

Job type can be:

- `foo`
- `bar`
- `baz`

Execution time is in seconds. 

```http
 curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"task_type":"foo","execution_time":10}'
  http://localhost:8080/tasks/create
```

### Get a task

Task id is the id of the task created in the previous step.

```http
curl --header "Content-Type: application/json" \
  --request GET \
  http://localhost:8080/tasks/{id}
```

### Delete a task

Task id is the id of the task created in the previous step.

```http
curl --header "Content-Type: application/json" \
  --request POST \
  http://localhost:8080/tasks/delete/{id}
```

### Get all tasks
Fetch tasks by job type and/or status.

status can be:
- `pending`
- `running`
- `complete`

```http
curl --header "Content-Type: application/json" \
  --request GET \
  --data '{"status":"pending"}' \
  http://localhost:8080/tasks
```
or
```http
curl --header "Content-Type: application/json" \
  --request GET \
  --data '{"status":"complte", "task_type:"baz"}' \
  http://localhost:8080/tasks
```
