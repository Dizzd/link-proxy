use std::future::Future;
tokio::task_local! {static ACTOR:String;}
pub async fn scope<F: Future>(actor: String, future: F) -> F::Output {
    ACTOR.scope(actor, future).await
}
pub fn current_actor() -> String {
    ACTOR
        .try_with(Clone::clone)
        .unwrap_or_else(|_| "system".into())
}
