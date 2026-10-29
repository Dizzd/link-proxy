use crate::service::*;
pub(crate) fn proxy_error(message: &'static str) -> Box<Error> {
    Error::explain(ErrorType::HTTPStatus(502), message)
}
pub(crate) fn bad_request(message: &'static str) -> Box<Error> {
    Error::explain(ErrorType::HTTPStatus(400), message)
}
