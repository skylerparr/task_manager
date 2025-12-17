use crate::access;
use crate::access::StatusResult;
use crate::models::*;
use actix::prelude::*;
use futures::future;
use rand::Rng;

pub async fn foo(id: String) {
    actix_rt::time::sleep(std::time::Duration::from_secs(3)).await;
    println!("foo {}", id);
}

pub async fn bar() {
    match reqwest::get("https://www.whattimeisitrightnow.com/").await {
        Ok(resp) => println!("bar status code: {:?}", resp.status().as_u16()),
        Err(_) => return,
    };
}

pub async fn baz() {
    let rng = rand::thread_rng().gen_range(0..=343);
    println!("Baz {}", rng);
}

async fn sart_task(task: &Task) {
    let t = access::start(&task);
    match t {
        StatusResult::Ok(task) => match task.get_type() {
            TaskType::Foo => {
                foo(task.id.to_string()).await;
                access::complete(&task);
            }
            TaskType::Bar => {
                bar().await;
                access::complete(&task);
            }
            TaskType::Baz => {
                baz().await;
                access::complete(&task);
            }
        },
        StatusResult::Err(_) => {}
    }
}

pub struct EventLoop;

impl Actor for EventLoop {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        System::new().block_on(async {
            loop {
                access::reset_stale_tasks();
                let pending_tasks = access::get_pending_tasks();
                if pending_tasks.is_empty() {
                    actix_rt::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
                let mut jobs = vec![];
                for task in pending_tasks {
                    let job = actix_rt::spawn(async move {
                        sart_task(&task).await;
                    });
                    jobs.push(job);
                }
                future::join_all(jobs).await;
            }
        });
    }
}
