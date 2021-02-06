extern crate job_scheduler;
extern crate lettre;
extern crate lettre_email;
extern crate scraper;
use job_scheduler::{Job, JobScheduler};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::extension::ClientId;
use lettre::smtp::ConnectionReuseParameters;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;
use regex::Regex;
use scraper::{Html, Selector};
use std::time::Duration;

const NEW_BOOK_URL: &str = "https://book.douban.com/latest?icn=index-latestbook-all";

fn fetch_book_html() -> std::string::String {
  let agent = ureq::Agent::new();
  let resp = agent
    .get(NEW_BOOK_URL)
    .set( "User-Agent",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36")
    .call()
    .unwrap();
  assert_eq!(resp.status(), 200);
  let html = resp.into_string().unwrap();
  // println!("{:?}", html);
  return html;
}

fn clean_html(html: &str) -> std::string::String {
  let document = Html::parse_document(html);
  let mut mail_html = "<html>".to_owned();
  mail_html += "<h1>虚构类</h1><hr/>";
  let mut selector = Selector::parse("#content > div > div.article > ul").unwrap();
  for element in document.select(&selector) {
    mail_html += &remove_img_tag(element.html().to_string());
  }
  mail_html += "<h1>非虚构类</h1><hr/>";
  selector = Selector::parse("#content > div > div.aside > ul").unwrap();
  for element in document.select(&selector) {
    mail_html += &remove_img_tag(element.html().to_string());
  }
  mail_html += "</html>";
  return mail_html;
}
// TODO: use this regexp <a class="cover"[^*]+?<\/a>
fn remove_img_tag(html: String) -> std::string::String {
  let re = Regex::new(r"<img").unwrap();
  let result = re.replace_all(&html, "<disableimg").to_owned().to_string();
  return result;
}

fn send_mail(html: &str) {
  let email = EmailBuilder::new()
    // Addresses can be specified by the tuple (email, alias)
    .to(("1456958184@qq.com", "一周新书推荐"))
    // ... or by an address only
    .from(("1456958184@qq.com", "rust client"))
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
    .credentials(Credentials::new(
      "1456958184@qq.com".to_string(),
      "yeualimouwtiided".to_string(),
    ))
    // Enable SMTPUTF8 if the server supports it
    .smtp_utf8(true)
    // Configure expected authentication mechanism
    .authentication_mechanism(Mechanism::Plain)
    // Enable connection reuse
    .connection_reuse(ConnectionReuseParameters::NoReuse)
    .transport();

  let result_1 = mailer.send(email.into());
  assert!(result_1.is_ok());
  // Explicitly close the SMTP transaction as we enabled connection reuse
  mailer.close();
}

fn do_weekly_job() {
  let html = fetch_book_html();
  let cleaned_html = clean_html(&html);
  send_mail(&cleaned_html)
}

fn main() {
  let mut sched = JobScheduler::new();

  sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
    // println!("I get executed every 10 seconds!");
    println!("{}", NEW_BOOK_URL);
    do_weekly_job();
  }));

  loop {
    sched.tick();

    std::thread::sleep(Duration::from_millis(500));
  }
}
