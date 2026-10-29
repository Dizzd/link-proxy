mod common;
use common::*;
use serde_json::{Value, json};
#[tokio::test]
#[ignore = "Requires real API and proxy: python3 scripts/e2e.py"]
async fn csv_xlsx_json_preview_explicit_commit_and_idempotency() {
    for format in ["csv", "xlsx", "json"] {
        let before = revision().await;
        let bytes = match format {
            "csv" => b"external_id,mapping_key,source_host,source_path,default_target_url
e2e-csv,e2e-csv,abc.example.com,/e2e-csv,https://csv.example/
"
            .to_vec(),
            "xlsx" => api("GET", "/imports/templates?format=xlsx", None)
                .await
                .bytes()
                .await
                .unwrap()
                .to_vec(),
            _ => serde_json::to_vec(
                &json!({"mappings":[mapping("e2e-json","https://json.example/")]}),
            )
            .unwrap(),
        };
        let part = reqwest::multipart::Part::bytes(bytes).file_name(format!("mappings.{format}"));
        let form = reqwest::multipart::Form::new()
            .text("format", format.to_string())
            .text("conflict_policy", "reject")
            .part("file", part);
        let uploaded: Value = client()
            .post(format!("{}/api/v1/imports", admin()))
            .bearer_auth(token())
            .multipart(form)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let id = uploaded["id"].as_str().unwrap();
        let mut ready = false;
        for _ in 0..300 {
            let job: Value = api("GET", &format!("/imports/{id}"), None)
                .await
                .json()
                .await
                .unwrap();
            if job["status"] == "ready" {
                ready = true;
                break;
            }
            assert_ne!(job["status"], "failed", "{job}");
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert!(ready);
        assert_eq!(revision().await, before);
        assert!(
            api("GET", &format!("/imports/{id}/preview"), None)
                .await
                .status()
                .is_success()
        );
        let body = json!({"import_job_id":id,"expected_revision":before,"conflict_policy":"reject","confirmation":true});
        for _ in 0..2 {
            let r = client()
                .post(format!("{}/api/v1/imports/{id}/commit", admin()))
                .bearer_auth(token())
                .header("Idempotency-Key", format!("e2e-{format}"))
                .json(&body)
                .send()
                .await
                .unwrap();
            assert!(r.status().is_success(), "{}", r.text().await.unwrap());
        }
        assert_eq!(revision().await, before + 1);
        let path = match format {
            "csv" => "/e2e-csv",
            "xlsx" => "/QR001",
            _ => "/e2e-json",
        };
        assert_eq!(get(path).await.status(), 302);
        let exported = api("GET", &format!("/exports/mappings?format={format}"), None).await;
        assert!(exported.status().is_success());
        assert!(!exported.bytes().await.unwrap().is_empty());
    }
}
