// Copyright 2026 Wayne Hong (h-alice) <contact@halice.art>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
  by design. Use `bill_charge` for network USAGE (energy/sessions), `bill_revenue`
  for MONEY billed.

## Picking the right MEMBER tool (member_analysis vs bill_member_analysis)
• `member_analysis` — a TIME SERIES of member COUNTS, one row per period: new
  sign-ups, cumulative base, bills issued, and DISTINCT active members
  (`unique_charge_member`). Use it for "how many members / how many distinct
  people charged over time". NOTE: `charge_member` is a BILL count, not distinct
  members.
• `bill_member_analysis` — a SEGMENT BREAKDOWN of success-only ORDER COUNTS,
  cross-tabbed by member cohort × AC/DC mode × station venue. MANY rows per
  period (one per segment cell), Chinese labels. Use it for "which segments
  (tiers / venues / AC vs DC) drive orders".
• PITFALL — these are different data sources; do not confuse them. For a count
  over time → `member_analysis`. For a cross-segment mix → `bill_member_analysis`.
• SIZE PITFALL — `bill_member_analysis` is a cross-product and can be EXTREMELY
  large. ALWAYS bound it with a TIGHT `start`/`end` (e.g. a single month) and a
  coarse `freq`. Never query it unbounded or over many quarters at fine `freq`.
"#;
