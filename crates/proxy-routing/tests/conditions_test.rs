use proxy_routing::conditions::{CompiledCondition, RequestContext};
#[test]
fn overnight_uses_previous_calendar_date_across_spring_dst() {
    let condition=CompiledCondition::compile(&serde_json::from_value(serde_json::json!({"type":"time_window","timezone":"Europe/Berlin","days_of_week":["sun"],"start":"22:00","end":"02:00"})).unwrap()).unwrap();
    let context = RequestContext {
        now: "2026-03-29T22:30:00Z".parse().unwrap(),
        user_agent: "",
        query: "",
    };
    assert!(condition.matches(&context));
}
