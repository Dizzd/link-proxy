use crate::service::*;
pub(crate) async fn respond(
    session: &mut Session,
    action: &Action,
    host: &str,
    path: &str,
    query: Option<&str>,
) -> Result<()> {
    let Action::Redirect { status, .. } = action else {
        return Err(bad_request("Redirect action required"));
    };
    let location = redirect_location(action, host, path, query).map_err(|_| {
        Error::explain(
            ErrorType::HTTPStatus(508),
            "Redirect loop or unsafe Location",
        )
    })?;
    let mut response = ResponseHeader::build(*status, Some(3))?;
    response.insert_header("Location", location)?;
    response.insert_header("Content-Length", "0")?;
    response.insert_header("Cache-Control", "no-store")?;
    session
        .write_response_header(Box::new(response), true)
        .await?;
    Ok(())
}
