//! The shared MCP instructions for the server.
//!
//! This string is sent **once** at the MCP handshake via
//! `ServerInfo.instructions`, so per-tool descriptions don't have to repeat it.

/// Shared-conventions block.
pub const INSTRUCTIONS: &str = r#"
This server exposes read-only tools over the datacenter unified API.

# Starcharger Tools Conventions
• Time semantics — every `label` in a response is an Asia/Taipei wall-clock
  timestamp at the start of the period. Pass `start`/`end` as ISO-8601 naive
  datetimes (e.g. `2025-01-01T00:00:00`); they bound `label`, not raw UTC
  timestamps.
• Growth fields — every `*_growth` value is a RATIO, not a percentage. `1.2`
  means +120 % growth (2.2× the previous period). The first row of each
  partition reports `0`; division-by-zero collapses to `0`.
• Nullable fields — any nullable metric is OMITTED ENTIRELY from the JSON when
  null or NaN upstream. Handle missing keys, not null values.
• `freq` parameter — time-bucket granularity. MUST be exactly one of:
  `day`, `week`, `week_sun`, `week_sat`, `month`, `quarter`, `year`.
  Defaults to `week_sun` when omitted. Any other value (e.g. `monthly`,
  `month_start`) is REJECTED with an `invalid_params` error — pick from the
  list above. (`station_revenue_ranking` additionally rejects `day`.)
• `limit` parameter — only honoured by `station_revenue_ranking`; silently
  ignored by every other tool. When calling `station_revenue_ranking`, ALWAYS
  pass `limit` (e.g. `10` for a top-10) UNLESS you explicitly want every station
  — its rows are ordered by `total_revenue` DESC, so omitting `limit` returns the
  full unbounded list.
• `seller_id` — when present, the `_seller` view variant is queried and the
  response carries a `seller_id` key. When absent, the network-wide variant is
  used.
• Reconciliation gotcha — `bill_revenue` is success-only (`bill_status = 14`);
  `bill_charge` counts cancellations and refunds. Their totals will not match
  by design.
"#;
