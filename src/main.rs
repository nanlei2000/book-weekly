extern crate job_scheduler;
extern crate lettre;
extern crate lettre_email;
extern crate scraper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use job_scheduler::{Job, JobScheduler};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::extension::ClientId;
use lettre::smtp::ConnectionReuseParameters;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;
use regex::Regex;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;

const NEW_BOOK_URL: &str = "https://book.douban.com/latest?icn=index-latestbook-all";
const SEP_TAG: &str = "<!-- SEP-1511151742953336 -->";

fn fetch_book_html() -> String {
  let agent = ureq::Agent::new();
  let resp = agent
    .get(NEW_BOOK_URL)
    .set( "User-Agent",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36")
    .call()
    .unwrap();
  assert_eq!(resp.status(), 200);
  let html = resp.into_string().unwrap();
  return html;
}

fn clean_html(html: &str) -> String {
  let document = Html::parse_document(html);
  let mut mail_html: String = "<html>".to_owned();
  mail_html += "<h1>虚构类</h1><hr/>";
  let mut selector = Selector::parse("#content > div > div.article > ul").unwrap();
  for element in document.select(&selector) {
    mail_html += &remove_img_tag(&element.html());
  }
  mail_html += "<h1>非虚构类</h1><hr/>";
  selector = Selector::parse("#content > div > div.aside > ul").unwrap();
  for element in document.select(&selector) {
    mail_html += &remove_img_tag(&element.html());
  }
  mail_html += "</html>";
  return mail_html;
}

// TODO: use this regexp: <a class="cover"[^*]+?<\/a>
fn remove_img_tag(html: &str) -> String {
  let re = Regex::new(r"<img").unwrap();
  let result = re.replace_all(&html, "<disableimg").to_string();
  return result;
}

fn send_mail(html: &str, config: MailConfig) {
  let email = EmailBuilder::new()
    .to((config.to.clone(), "一周新书推荐"))
    .from((config.to.clone(), "rust client"))
    .subject("一周新书推荐")
    .html(html)
    .build()
    .unwrap();

  // Connect to a remote server on a custom port
  let mut mailer = SmtpClient::new_simple("smtp.qq.com")
    .unwrap()
    // Set the name sent during EHLO/HELO, default is `localhost`
    .hello_name(ClientId::Domain("smtp.qq.com".to_string()))
    // Add credentials for authentication
    .credentials(Credentials::new(config.auth.user, config.auth.pass))
    // Enable SMTPUTF8 if the server supports it
    .smtp_utf8(true)
    // Configure expected authentication mechanism
    .authentication_mechanism(Mechanism::Plain)
    // Enable connection reuse
    .connection_reuse(ConnectionReuseParameters::NoReuse)
    .transport();

  let result = mailer.send(email.into());
  assert!(result.is_ok());
  // Explicitly close the SMTP transaction as we enabled connection reuse
  mailer.close();
}

#[derive(Debug, Serialize, Deserialize)]
struct MailAuthConfig {
  user: String,
  pass: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MailConfig {
  auth: MailAuthConfig,
  to: String,
}

/// read `.env.json`
/// ```{
///   "auth": {
///       "user": "xxx@qq.com",
///       "pass": "xxxx"
///   },
///   "to": "xxx@qq.com"
/// }```
fn read_config() -> MailConfig {
  let f = File::open(".env.json").unwrap();
  let config: MailConfig = serde_json::from_reader(f).unwrap();
  println!("{:?}", config);
  return config;
}

fn append_to_html(part: &str) -> String {
  let mut html_file = File::open("./index.html").unwrap();
  let mut html: String = String::new();
  html_file.read_to_string(&mut html).unwrap();
  let res: Vec<&str> = html.split(SEP_TAG).collect();
  let mut new_html = res[0].to_owned();
  new_html = new_html + SEP_TAG + "\n" + part + res[1];
  let mut buffer = File::create("index.html").unwrap();
  buffer.write(new_html.as_bytes()).unwrap();
  return new_html;
}

// commit html then the online website will refresh
fn commit_changes() {
  let mut git = Command::new("git");
  let mut output = git.args(&["pull", "origin", "master"]).output().unwrap();
  println!("status: {}", output.status);
  io::stdout().write_all(&output.stdout).unwrap();
  io::stderr().write_all(&output.stderr).unwrap();
  assert!(output.status.success());
  output = git.args(&["add", "."]).output().unwrap();
  println!("status: {}", output.status);
  io::stdout().write_all(&output.stdout).unwrap();
  io::stderr().write_all(&output.stderr).unwrap();
  assert!(output.status.success());
  output = git
    .args(&["commit", "-m", "'html change'"])
    .output()
    .unwrap();
  println!("status: {}", output.status);
  io::stdout().write_all(&output.stdout).unwrap();
  io::stderr().write_all(&output.stderr).unwrap();
  assert!(output.status.success());
  output = git.args(&["push", "origin", "master"]).output().unwrap();
  println!("status: {}", output.status);
  io::stdout().write_all(&output.stdout).unwrap();
  io::stderr().write_all(&output.stderr).unwrap();
  assert!(output.status.success());
}

#[test]
fn test_commit_changes() {
  commit_changes()
}

fn do_weekly_job() {
  let html = fetch_book_html();
  let cleaned_html = clean_html(&html);
  let config = read_config();
  send_mail(&cleaned_html, config);
  append_to_html(&cleaned_html);
  commit_changes();
}

fn schedule_job() {
  let mut scheduler = JobScheduler::new();
  // 每周六 十点
  scheduler.add(Job::new("00 00 10 * * SAT".parse().unwrap(), || {
    println!("fetch this url: {}", NEW_BOOK_URL);
    do_weekly_job();
  }));
  loop {
    scheduler.tick();
    std::thread::sleep(Duration::from_millis(500));
  }
}

fn main() {
  schedule_job();
}
