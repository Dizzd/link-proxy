# Conditional Routing

Conditions are strongly typed, tagged JSON trees; they are never scripts. All/Any require nonempty children and short-circuit. Not negates one child. Query values are form-decoded and compared exactly; repeated parameters match when any pair equals the configured name/value.

TimeWindow uses an IANA timezone, weekday abbreviations mon..sun and HH:MM boundaries. Start is inclusive, end exclusive. Equal start/end is rejected. Overnight intervals belong to the calendar weekday on which they start; this uses previous local calendar date, not a subtraction of 24 hours across daylight-saving transitions. DateRange includes both configured local dates. Production uses SystemClock (UTC); tests/simulator may supply deterministic UTC times.

Device heuristics classify bot before mobile, then tablet/desktop/unknown using bounded User-Agent input. Android without Mobile is tablet. Unknown/custom user agents remain unknown. These heuristics must not be used as access control.

Example: an All group combines mobile, a time_window for mon..fri 08:00–18:00 Asia/Ho_Chi_Minh, and Not(query skip=yes). Action selects https://mobile.example.com. A lower-priority weekend rule can use a sat/sun time window; default_action selects https://default.example.com. See tests/fixtures/conditional_mappings.json.

Compiler limits: 64 rules per mapping, condition depth 12, 128 nodes per condition. Management JSON is limited to 1 MiB; imports have separate configured file/record/decompression limits. Invalid disabled rules are still rejected, preventing latent unsafe configurations.
