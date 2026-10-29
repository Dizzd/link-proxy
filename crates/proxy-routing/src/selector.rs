use super::*;
impl CompiledMapping {
    pub fn action(&self, ctx: &RequestContext<'_>) -> &Action {
        self.rules
            .iter()
            .find(|(c, _)| c.matches(ctx))
            .map(|(_, a)| a)
            .unwrap_or(&self.mapping.default_action)
    }
}
