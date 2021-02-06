extern crate job_scheduler;
use job_scheduler::{Job, JobScheduler};
use std::time::Duration;

const NEW_BOOK_URL: &str = "https://book.douban.com/latest?icn=index-latestbook-all";

fn main() {
  let mut sched = JobScheduler::new();

  sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
    // println!("I get executed every 10 seconds!");
    println!("{}", NEW_BOOK_URL)
  }));

  loop {
    sched.tick();

    std::thread::sleep(Duration::from_millis(500));
  }
}
