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

//! Query parameter definitions

use chrono::NaiveDateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Time window + seller + granularity, common to every aggregated endpoint.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PileDataQueryWindow {
    /// Inclusive lower bound on `label`.
    ///
    /// Asia/Taipei naive datetime, e.g.`2025-01-01T00:00:00`. Omit for unbounded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<NaiveDateTime>,

    /// Inclusive lower bound on `label`.
    ///
    /// Asia/Taipei naive datetime, e.g.`2025-01-01T00:00:00`. Omit for unbounded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<NaiveDateTime>,

    /// Filter to a single seller.
    ///
    /// When present, the `_seller` view variant is queried and the response includes
    /// a `seller_id` key; When absent the network-wide variant is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_id: Option<i64>,

    /// The record limit, honoured ONLY by `station_revenue_ranking` (applied as a
    /// SQL `LIMIT` after `ORDER BY total_revenue DESC`); every other endpoint
    /// ignores it.
    ///
    /// **PITFALL**: for `station_revenue_ranking`, DO NOT omit `limit` unless you
    /// explicitly want the full, unbounded list of every station. For a top-N
    /// query (e.g. "top 10 stations") pass `limit = 10`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// Time bucket granularity.
    ///
    /// Default `week_sun`. Unknown values fall back to `week_sun` (upstream warns).
    ///
    /// **WARN**: `station_revenue_ranking` rejects `day` with HTTP 400.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freq: Option<Freq>,
}

/// Temporal aggregation frequency/granularity.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Freq {
    /// Daily aggregation.
    Day,
    /// Weekly aggregation (usually starts on Monday).
    Week,
    /// Weekly aggregation starting on Sunday.
    WeekSun,
    /// Weekly aggregation starting on Saturday.
    WeekSat,
    /// Monthly aggregation.
    Month,
    /// Quarterly aggregation.
    Quarter,
    /// Yearly aggregation.
    Year,
}
