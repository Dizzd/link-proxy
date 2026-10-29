mod common;
use common::*;
use proxy_domain::*;
use proxy_routing::{conditions::*, *};
use serde_json::json;

fn snapshot(mappings: Vec<Mapping>) -> anyhow::Result<RoutingSnapshot> {
    RoutingSnapshot::compile(1, configuration(mappings), &[])
}
fn ctx(now: &str, ua: &'static str, query: &'static str) -> RequestContext<'static> {
    RequestContext {
        now: now.parse().unwrap(),
        user_agent: ua,
        query,
    }
}
fn condition(value: serde_json::Value) -> CompiledCondition {
    CompiledCondition::compile(&serde_json::from_value(value).unwrap()).unwrap()
}

#[test]
fn exact_paths_share_a_destination() {
    let s = snapshot(vec![mapping("a", "/a"), mapping("b", "/b")]).unwrap();
    for path in ["/a", "/b"] {
        assert!(s.lookup("EXAMPLE.COM.:80", path, "GET").is_some());
    }
}
#[test]
fn specificity_precedes_mapping_priority() {
    let mut prefix = mapping("prefix", "/");
    prefix.source_matcher.path_type = PathType::Prefix;
    prefix.priority = 1000;
    let mut template = mapping("template", "/items/{id}");
    template.source_matcher.path_type = PathType::Template;
    let s = snapshot(vec![prefix, template, mapping("exact", "/items/1")]).unwrap();
    assert_eq!(
        s.lookup("example.com", "/items/1", "GET")
            .unwrap()
            .mapping
            .mapping
            .mapping_key,
        "exact"
    );
    assert_eq!(
        s.lookup("example.com", "/items/2", "GET")
            .unwrap()
            .parameters["id"],
        "2"
    );
    assert_eq!(
        s.lookup("example.com", "/other", "GET")
            .unwrap()
            .mapping
            .mapping
            .mapping_key,
        "prefix"
    );
}
#[test]
fn template_mapping_key_index_selects_correct_mapping() {
    let maps = (0..10_000)
        .map(|i| {
            let mut m = mapping(&format!("key{i}"), "/{mapping_id}");
            m.source_matcher.path_type = PathType::Template;
            m
        })
        .collect();
    let s = snapshot(maps).unwrap();
    assert_eq!(
        s.lookup("example.com", "/key9999", "GET")
            .unwrap()
            .mapping
            .mapping
            .mapping_key,
        "key9999"
    );
    assert!(s.lookup("example.com", "/missing", "GET").is_none());
}
#[test]
fn methods_and_priority_are_deterministic() {
    let mut a = mapping("a", "/a");
    a.source_matcher.methods = vec!["get".into()];
    let mut b = mapping("b", "/a");
    b.priority = 10;
    b.source_matcher.methods = vec!["POST".into()];
    let s = snapshot(vec![a, b]).unwrap();
    assert_eq!(
        s.lookup("example.com", "/a", "GET")
            .unwrap()
            .mapping
            .mapping
            .mapping_key,
        "a"
    );
    assert_eq!(
        s.lookup("example.com", "/a", "POST")
            .unwrap()
            .mapping
            .mapping
            .mapping_key,
        "b"
    );
    assert!(s.lookup("example.com", "/a", "DELETE").is_none());
}
#[test]
fn ambiguous_same_priority_routes_are_rejected() {
    assert!(snapshot(vec![mapping("a", "/x"), mapping("b", "/x")]).is_err());
}
#[test]
fn template_equivalent_patterns_are_rejected() {
    let mut a = mapping("a", "/{id}");
    a.source_matcher.path_type = PathType::Template;
    let mut b = mapping("b", "/{name}");
    b.source_matcher.path_type = PathType::Template;
    assert!(snapshot(vec![a, b]).is_err());
}
#[test]
fn disabled_mapping_and_domain_do_not_route() {
    let mut m = mapping("a", "/a");
    m.enabled = false;
    assert!(
        snapshot(vec![m])
            .unwrap()
            .lookup("example.com", "/a", "GET")
            .is_none()
    );
    let mut c = configuration(vec![mapping("a", "/a")]);
    c.domains
        .push(serde_json::from_value(json!({"hostname":"example.com","enabled":false})).unwrap());
    assert!(
        RoutingSnapshot::compile(1, c, &[])
            .unwrap()
            .lookup("example.com", "/a", "GET")
            .is_none()
    );
}
#[test]
fn enabled_rules_use_priority_then_default() {
    let mut m = mapping("a", "/a");
    for (p, target, enabled) in [(1, "low", true), (2, "high", true), (3, "disabled", false)] {
        m.rules.push(serde_json::from_value(json!({"priority":p,"enabled":enabled,"condition":{"type":"query_parameter_equals","name":"x","value":"1"},"action":{"type":"redirect","status":302,"target_url":format!("https://{target}.example/")}})).unwrap());
    }
    let s = snapshot(vec![m]).unwrap();
    let route = s.lookup("example.com", "/a", "GET").unwrap();
    assert!(
        matches!(route.mapping.action(&ctx("2026-10-29T00:00:00Z","","x=1")),Action::Redirect {target_url,..} if target_url.contains("high"))
    );
    assert!(
        matches!(route.mapping.action(&ctx("2026-10-29T00:00:00Z","","")),Action::Redirect {target_url,..} if target_url.contains("destination"))
    );
}
#[test]
fn equal_enabled_rule_priorities_rejected() {
    let mut m = mapping("a", "/a");
    let rule:MappingRule=serde_json::from_value(json!({"priority":1,"condition":{"type":"device_type","in":["mobile"]},"action":{"type":"redirect","status":302,"target_url":"https://a.example"}})).unwrap();
    m.rules = vec![
        rule.clone(),
        MappingRule {
            id: "other".into(),
            ..rule
        },
    ];
    assert!(snapshot(vec![m]).is_err());
}
#[test]
fn timezone_start_inclusive_end_exclusive() {
    let c = condition(
        json!({"type":"time_window","timezone":"Asia/Ho_Chi_Minh","days_of_week":["thu"],"start":"08:00","end":"18:00"}),
    );
    assert!(!c.matches(&ctx("2026-10-29T00:59:59Z", "", "")));
    assert!(c.matches(&ctx("2026-10-29T01:00:00Z", "", "")));
    assert!(!c.matches(&ctx("2026-10-29T11:00:00Z", "", "")));
    assert!(!c.matches(&ctx("2026-10-31T02:00:00Z", "", "")));
}
#[test]
fn overnight_weekday_is_window_start_day() {
    let c = condition(
        json!({"type":"time_window","timezone":"UTC","days_of_week":["fri"],"start":"22:00","end":"02:00"}),
    );
    assert!(c.matches(&ctx("2026-10-31T01:59:59Z", "", "")));
    assert!(!c.matches(&ctx("2026-10-31T02:00:00Z", "", "")));
}
#[test]
fn date_range_includes_both_local_dates() {
    let c = condition(
        json!({"type":"date_range","timezone":"Asia/Ho_Chi_Minh","start":"2026-10-29","end":"2026-10-30"}),
    );
    assert!(c.matches(&ctx("2026-10-28T17:00:00Z", "", "")));
    assert!(!c.matches(&ctx("2026-10-30T17:00:00Z", "", "")));
}
#[test]
fn boolean_nested_conditions_and_decoded_query() {
    let c = condition(
        json!({"type":"all","conditions":[{"type":"any","conditions":[{"type":"device_type","in":["mobile"]},{"type":"device_type","in":["tablet"]}]},{"type":"not","condition":{"type":"query_parameter_equals","name":"skip","value":"yes"}},{"type":"query_parameter_equals","name":"a b","value":"c+d"}]}),
    );
    assert!(c.matches(&ctx("2026-10-29T00:00:00Z", "iPhone Mobile", "a+b=c%2Bd")));
    assert!(!c.matches(&ctx(
        "2026-10-29T00:00:00Z",
        "iPhone Mobile",
        "a+b=c%2Bd&skip=yes"
    )));
}
#[test]
fn device_types_include_unknown_and_bot() {
    for (ua, expected) in [
        ("Mozilla Windows NT", Device::Desktop),
        ("", Device::Unknown),
        ("CustomAgent", Device::Unknown),
        ("Googlebot Mobile", Device::Bot),
        ("iPad", Device::Tablet),
        ("Android", Device::Tablet),
        ("iPhone Mobile", Device::Mobile),
    ] {
        assert_eq!(detect_device(ua), expected);
    }
}
#[test]
fn malicious_conditions_and_destinations_are_rejected() {
    let c = Condition::TimeWindow {
        timezone: "invalid/timezone".into(),
        days_of_week: vec!["mon".into()],
        start: "08:00".into(),
        end: "18:00".into(),
    };
    assert!(CompiledCondition::compile(&c).is_err());
    let mut deep = Condition::DeviceType {
        r#in: vec![Device::Mobile],
    };
    for _ in 0..14 {
        deep = Condition::Not {
            condition: Box::new(deep),
        };
    }
    assert!(CompiledCondition::compile(&deep).is_err());
    for target in [
        "javascript:alert(1)",
        "https://user:secret@example.com/",
        "https://example.com/\r\nInjected: yes",
    ] {
        let mut m = mapping("a", "/a");
        m.default_action = Action::Redirect {
            status: 302,
            target_url: target.into(),
            path_handling: PathHandling::Replace,
            query_policy: QueryPolicy::Drop,
        };
        assert!(snapshot(vec![m]).is_err());
    }
}
#[test]
fn redirect_query_policies_and_append() {
    let a = Action::Redirect {
        status: 302,
        target_url: "https://target.example/base?fixed=1".into(),
        path_handling: PathHandling::Append,
        query_policy: QueryPolicy::Merge,
    };
    assert_eq!(
        redirect_location(&a, "example.com", "/x", Some("a=2")).unwrap(),
        "https://target.example/base/x?fixed=1&a=2"
    );
}
#[test]
fn direct_redirect_loop_is_rejected() {
    let a = Action::Redirect {
        status: 302,
        target_url: "http://example.com/a".into(),
        path_handling: PathHandling::Replace,
        query_policy: QueryPolicy::Drop,
    };
    assert!(redirect_location(&a, "example.com", "/a", None).is_err());
}
#[test]
fn proxy_reference_and_rewrite_are_validated() {
    let mut m = mapping("a", "/items/{id}");
    m.source_matcher.path_type = PathType::Template;
    m.default_action = Action::ReverseProxy {
        upstream_id: "u1".into(),
        rewrite: Some("/api/{id}".into()),
        preserve_host: false,
    };
    assert!(snapshot(vec![m.clone()]).is_err());
    let mut c = configuration(vec![m]);
    c.upstreams.push(serde_json::from_value(json!({"id":"u1","name":"backend","scheme":"https","hostname":"backend.example","port":443})).unwrap());
    let s = RoutingSnapshot::compile(1, c, &[]).unwrap();
    let r = s.lookup("example.com", "/items/12", "GET").unwrap();
    assert_eq!(rewrite_path("/api/{id}", &r.parameters).unwrap(), "/api/12");
}
#[test]
fn clock_abstraction_is_deterministic() {
    let clock = FixedClock("2026-10-29T00:00:00Z".parse().unwrap());
    assert_eq!(clock.now(), clock.now());
}
