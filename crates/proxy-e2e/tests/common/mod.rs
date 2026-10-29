#![allow(dead_code)]
use reqwest::{Client, Response};
use serde_json::{Value, json};
pub fn client() -> Client {
    Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}
pub fn admin() -> String {
    std::env::var("PROXY_E2E_ADMIN").expect("Run scripts/e2e.py")
}
pub fn proxy() -> String {
    std::env::var("PROXY_E2E_PROXY").expect("Run scripts/e2e.py")
}
pub fn token() -> String {
    std::env::var("ADMIN_API_TOKEN").unwrap()
}
pub async fn api(method: &str, path: &str, body: Option<Value>) -> Response {
    let mut req = client()
        .request(method.parse().unwrap(), format!("{}/api/v1{path}", admin()))
        .bearer_auth(token());
    if let Some(body) = body {
        req = req.json(&body)
    }
    req.send().await.unwrap()
}
pub async fn revision() -> i64 {
    api("GET", "/config/active", None)
        .await
        .json::<Value>()
        .await
        .unwrap()["revision"]
        .as_i64()
        .unwrap()
}
pub async fn publish() -> i64 {
    let rev = revision().await;
    let r = api(
        "POST",
        "/config/publish",
        Some(json!({"expected_revision":rev})),
    )
    .await;
    assert!(r.status().is_success(), "{}", r.text().await.unwrap());
    revision().await
}
pub fn mapping(key: &str, target: &str) -> Value {
    json!({"id":key,"mapping_key":key,"name":key,"source_matcher":{"hosts":["abc.example.com","second.example.com"],"path_type":"exact","path_value":format!("/{key}"),"methods":["GET"]},"default_action":{"type":"redirect","status":302,"target_url":target}})
}
pub async fn create(value: Value) {
    let r = api("POST", "/mappings", Some(value)).await;
    assert_eq!(r.status(), 201, "{}", r.text().await.unwrap());
}
pub async fn get(path: &str) -> Response {
    client()
        .get(format!("{}{path}", proxy()))
        .header("Host", "abc.example.com")
        .send()
        .await
        .unwrap()
}
