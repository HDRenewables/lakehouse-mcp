# Datacenter MCP server

A [Model Context Protocol](https://modelcontextprotocol.io/) server that exposes
the datacenter API as read-only, self-documenting tools.

## Tools

| Tool | What it returns |
|---|---|
| `bill_revenue` | Period revenue totals (success-only): AC/DC split, discount ratio, growth. |
| `station_revenue_ranking` | Per-station ranking with the clean `revenue_per_kw` (takes `limit`, rejects `freq=day`). |
| `bill_charge` | Charging activity (sessions, seconds, kWh); includes refunds, does **not** reconcile with `bill_revenue`. |
| `member_analysis` | Member counts per period (new, cumulative, distinct active). |
| `business_metrics` | Station & pile growth per period (new + cumulative footprint). |
| `bill_member_analysis` | Order counts cross-segmented by member cohort × AC/DC × venue . |

Core concepts and conventions (Asia/Taipei time, growth-as-ratio, valid `freq`
values, the `limit` and `member_analysis` vs `bill_member_analysis` pitfalls) are
sent once at the MCP handshake, and every field is documented in the tool's
`outputSchema`.

## Configuration

Set via environment (a `.env` file is loaded at startup):

| Variable | Purpose | Default |
|---|---|---|
| `DATACENTER_API_BASE` | Upstream base URL (**required**) | — |
| `BIND_ADDR` | Listen address (`--serve` mode) | `0.0.0.0` |
| `BIND_PORT` | Listen port (`--serve` mode) | `8000` |
| `RUST_LOG` | Log filter | `info` |

Endpoint slug -> upstream path mappings live in [`config.toml`](config.toml).

## Running

Two transports share the same tool logic:

```bash
# stdio (default): a client spawns the binary as a subprocess
cargo run

# Streamable HTTP: standard MCP remote transport at POST/GET /mcp
cargo run -- --serve
```

For a public deployment, run `--serve` behind a reverse proxy that terminates
TLS and enforces auth.

## Verifying

Point any MCP client at the server (e.g. the
[MCP Inspector](https://modelcontextprotocol.io/docs/tools/inspector)):

```bash
npx @modelcontextprotocol/inspector ./target/debug/eomc-mcp            # stdio
npx @modelcontextprotocol/inspector --url http://127.0.0.1:8000/mcp    # HTTP (server running with --serve)
```

## Acknowledgments

Portions of this codebase were generated with the assistance of Claude Opus 4.8. The human developers maintain full authorship and have conducted rigorous testing, refactoring, and validation of the final codebase.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for project change history and release notes.
