use crate::router::*;
pub(crate) async fn upload(
    State(s): State<ApiState>,
    mut multipart: Multipart,
) -> ApiResult<(StatusCode, Json<Value>)> {
    // Reserve bounded upload capacity before reading multipart file bytes.
    let _upload_slot = s
        .control
        .upload_slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| anyhow::anyhow!("upload capacity reached"))?;
    let (mut file, mut name, mut format, mut policy) = (None, None, None, None);
    while let Some(field) = multipart.next_field().await? {
        match field.name().unwrap_or_default() {
            "file" => {
                ensure!(file.is_none(), "only one file per job");
                name = field.file_name().map(str::to_owned);
                let bytes = field.bytes().await?;
                ensure!(
                    bytes.len() <= s.control.settings.imports.max_upload_bytes,
                    "upload exceeds limit"
                );
                file = Some(bytes.to_vec());
            }
            "format" => {
                format = Some(serde_json::from_value::<Format>(Value::String(
                    field.text().await?,
                ))?)
            }
            "conflict_policy" => {
                policy = Some(serde_json::from_value::<ConflictPolicy>(Value::String(
                    field.text().await?,
                ))?)
            }
            _ => return Err(anyhow::anyhow!("unknown multipart field").into()),
        }
    }
    let name = name.ok_or_else(|| anyhow::anyhow!("file name required"))?;
    ensure!(
        name.len() <= 255 && !name.chars().any(char::is_control),
        "invalid file name"
    );
    let id = s
        .control
        .submit_import(
            name,
            format.ok_or_else(|| anyhow::anyhow!("format required"))?,
            policy.ok_or_else(|| anyhow::anyhow!("conflict_policy required"))?,
            file.ok_or_else(|| anyhow::anyhow!("file required"))?,
        )
        .await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"id":id,"status":"pending"})),
    ))
}
pub(crate) async fn list_imports(
    State(s): State<ApiState>,
    Query(page): Query<Page>,
) -> ApiResult<Json<Value>> {
    let ids: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM import_jobs ORDER BY created_at DESC,id LIMIT $1 OFFSET $2",
    )
    .bind(page.limit() as i64)
    .bind(page.offset as i64)
    .fetch_all(&s.control.store.pool)
    .await?;
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM import_jobs")
        .fetch_one(&s.control.store.pool)
        .await?;
    let mut jobs = Vec::new();
    for id in ids {
        jobs.push(s.control.import_job(&id).await?);
    }
    Ok(Json(json!({"items":jobs,"total":total})))
}
pub(crate) async fn get_import(
    State(s): State<ApiState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    Ok(Json(json!(s.control.import_job(&id).await?)))
}
pub(crate) async fn preview_import(
    State(s): State<ApiState>,
    Path(id): Path<String>,
    Query(page): Query<Page>,
) -> ApiResult<Json<Value>> {
    let job = s.control.import_job(&id).await?;
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT data FROM import_staging_records WHERE job_id=$1 ORDER BY ordinal LIMIT $2 OFFSET $3",
    )
    .bind(&id)
    .bind(page.limit() as i64)
    .bind(page.offset as i64)
    .fetch_all(&s.control.store.pool)
    .await?;
    let records: Vec<proxy_import::StagedRecord> = rows
        .iter()
        .map(|v| serde_json::from_str(v))
        .collect::<Result<_, _>>()?;
    Ok(Json(json!(proxy_import::preview::ImportPreview {
        job,
        items: records
    })))
}
pub(crate) async fn import_errors(
    State(s): State<ApiState>,
    Path(id): Path<String>,
    Query(page): Query<Page>,
) -> ApiResult<Response> {
    let job = s.control.import_job(&id).await?;
    let limit = if matches!(page.format, Some(Format::Csv)) {
        s.control.settings.imports.max_records + 1
    } else {
        page.limit()
    };
    let rows: Vec<(String,String,String,String,String)>=sqlx::query_as("SELECT location,field,code,description,COALESCE(suggestion,'') FROM import_job_errors WHERE job_id=$1 ORDER BY id LIMIT $2 OFFSET $3").bind(&id).bind(limit as i64).bind(page.offset as i64).fetch_all(&s.control.store.pool).await?;
    let errors: Vec<_> = rows
        .into_iter()
        .map(
            |(location, field, code, description, suggestion)| ImportJobError {
                file_name: job.file_name.clone(),
                location,
                field,
                code,
                description,
                suggestion,
            },
        )
        .collect();
    if matches!(page.format, Some(Format::Csv)) {
        let mut csv = csv::Writer::from_writer(Vec::new());
        csv.write_record([
            "file_name",
            "location",
            "field",
            "code",
            "description",
            "suggestion",
        ])?;
        for e in errors {
            csv.write_record(
                [
                    e.file_name,
                    e.location,
                    e.field,
                    e.code,
                    e.description,
                    e.suggestion,
                ]
                .iter()
                .map(|v| spreadsheet_safe(v)),
            )?;
        }
        return Ok(download(
            Format::Csv,
            "import-errors.csv",
            csv.into_inner()?,
        )?);
    }
    Ok(Json(json!({"total":job.error_count,"items":errors})).into_response())
}
pub(crate) async fn commit_import(
    State(s): State<ApiState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<CommitImport>,
) -> ApiResult<Json<Value>> {
    ensure!(id == input.import_job_id, "import_job_id must match URL");
    let key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("Idempotency-Key header required"))?;
    Ok(Json(json!(
        s.control
            .commit_import(
                &id,
                input.expected_revision,
                input.conflict_policy,
                input.confirmation,
                key
            )
            .await?
    )))
}
pub(crate) async fn cancel_import(
    State(s): State<ApiState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    s.control.cancel_import(&id).await?;
    Ok(Json(json!({"status":"cancelled"})))
}
pub(crate) async fn import_template(Query(page): Query<Page>) -> ApiResult<Response> {
    let format = page.format.unwrap_or(Format::Csv);
    Ok(download(
        format,
        &format!("mappings-template.{}", format.as_str()),
        template(format)?,
    )?)
}
