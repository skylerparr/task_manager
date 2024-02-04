use actix::prelude::*;

#[derive(Default)]
pub struct TaskWorker;

impl Actor for TaskWorker {
    type Context = SyncContext<Self>;
}

impl actix::Supervised for TaskWorker {}

impl ArbiterService for TaskWorker {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        println!("TaskWorker started");
    }
}
