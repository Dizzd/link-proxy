use crate::conditions::{CompiledCondition, RequestContext};
use anyhow::{Result, ensure};
use proxy_domain::*;
use std::collections::{HashMap, HashSet};

pub mod conditions;
pub mod rewrite;
pub mod validator;
pub use rewrite::*;
pub use validator::*;
pub struct CompiledMapping {
    pub mapping: Mapping,
    rules: Vec<(CompiledCondition, Action)>,
}

#[derive(Default)]
struct RouteGroup {
    routes: Vec<usize>,
    keys: HashMap<String, Vec<usize>>,
}
#[derive(Default)]
struct PrefixNode {
    children: HashMap<u8, PrefixNode>,
    routes: Vec<usize>,
}
#[derive(Default)]
struct HostRoutes {
    exact: HashMap<String, Vec<usize>>,
    templates: matchit::Router<usize>,
    groups: Vec<RouteGroup>,
    template_patterns: HashMap<String, usize>,
    prefix: PrefixNode,
}

mod compiler;
mod matcher;
mod selector;
pub mod template;
pub use template::redirect_location;
pub mod snapshot;
pub use snapshot::*;
pub mod error;
pub mod simulator;
