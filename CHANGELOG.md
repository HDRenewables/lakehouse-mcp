# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-06

Initial release of the EOMC datacenter MCP server.

### Added

- **Dual mode MCP server**:
  - **stdio** (default): for a client that spawns the binary as a subprocess.
  - **Streamable HTTP** (`--serve`): the standard MCP remote transport, served
    over axum at `POST/GET /mcp`, binding `BIND_ADDR` (default `0.0.0.0`) :
    `BIND_PORT` (default `8000`).
- **Six essential Starcharger datacenter endpoints**:
  - `bill_revenue`: period-bucketed revenue totals (success-only): AC/DC split,
    discount ratio, legacy per-kW, and period-over-period growth.
  - `station_revenue_ranking`: per-station ranking with the clean
    `revenue_per_kw`, ordered by `total_revenue` (takes `limit`, rejects
    `freq=day`).
  - `bill_charge`: period-bucketed charging activity (sessions, seconds, kWh);
    includes refunds/cancellations, so it does not reconcile with `bill_revenue`.
  - `member_analysis`: member acquisition and active-member counts per period
    (new, cumulative, bill count, and distinct active members).
  - `business_metrics`: station and pile growth per period (new and cumulative
    footprint).
  - `bill_member_analysis`: success-only order counts cross-segmented by member
    cohort × AC/DC mode × station venue.
