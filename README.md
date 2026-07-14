<div align="center">

# mock-cli

**Blazing fast OpenAPI Mock Server in Rust.**
Start mocking in milliseconds. Zero config. Dynamic fake data.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](#license)
[![CI](https://github.com/wanghao12345/mock-cli/actions/workflows/release.yml/badge.svg)](https://github.com/wanghao12345/mock-cli/actions)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

[Installation](#installation) • [Quick Start](#quick-start) • [Features](#features) • [CLI Reference](#cli-reference) • [Roadmap](#roadmap)

</div>

---

## Why mock-cli?

Most OpenAPI mock servers are slow, require Docker, or return static stubs. `mock-cli` is different:

- **Fast Startup** — Native Rust binary, no JVM/Node/Docker overhead
- **Dynamic Fake Data** — Generates realistic random data based on your schema types (string formats, enums, arrays, objects)
- **Path-Aware** — Path parameters are extracted and echoed back into the response when the field name matches
- **Schema-Aware** — Honors `format` (email/uuid/date-time/uri/...), `enum`, `required`, `minItems`/`maxItems`
- **Correct Status Codes** — 404 for unknown paths, 405 for unsupported methods, 200 for success
- **Cycle-Safe** — Recursive `$ref` cycles are bounded so the server never crashes
- **Single Binary** — One static binary, works offline, no runtime required
- **Cross-Platform** — macOS / Linux / Windows with native installers


## Installation

### Build from source (recommended for now)

```bash
git clone https://github.com/wanghao12345/mock-cli.git
cd mock-cli
cargo install --path crates/cli
```

### Pre-built binaries

Download the latest release from [GitHub Releases](https://github.com/wanghao12345/mock-cli/releases).

### Run directly with cargo

```bash
cargo run --release -- examples/petstore.yaml
```


## Quick Start

**1. Use the bundled example spec:**

```bash
cargo run -- examples/petstore.yaml
```

Or create your own `petstore.yaml`:

```yaml
openapi: 3.0.3
info:
  title: Petstore
  version: 1.0.0
paths:
  /pets/{petId}:
    get:
      parameters:
        - name: petId
          in: path
          required: true
          schema:
            type: integer
      responses:
        '200':
          description: OK
          content:
            application/json:
              schema:
                type: object
                properties:
                  petId:
                    type: integer
                  name:
                    type: string
                  vaccinated:
                    type: boolean
```

**2. Start the mock server:**

```bash
mock-cli petstore.yaml
```

```text
✓ Loaded "petstore.yaml" (1 paths)
✓ Listening on 127.0.0.1:3333
```

**3. Get dynamic fake data:**

```bash
curl http://localhost:3333/pets/123
# {"petId":123,"name":"Dr. Margret Kihn","vaccinated":true}

curl http://localhost:3333/pets/456
# {"petId":456,"name":"Sarah Connor","vaccinated":false}
```

Notice that `petId` in the response echoes the path parameter value `123` / `456` (coerced to the schema type `integer`). See [Path parameter injection](#path-parameter-injection). Every request returns **different, schema-compliant** fake data. No static stubs.


## Features

### Schema-driven fake data

| OpenAPI construct | Behavior |
|-------------------|----------|
| `type: string` + `format: email` | Generates realistic email addresses |
| `type: string` + `format: uuid` | Generates v4 UUIDs |
| `type: string` + `format: date` / `date-time` | Generates ISO 8601 date / datetime strings |
| `type: string` + `format: uri` | Generates example URIs |
| `type: string` + `enum` | Picks a random allowed value |
| `type: integer` / `number` | Random numeric values |
| `type: boolean` | Random `true` / `false` |
| `type: object` + `required` | Required fields always present; optional fields may be `null` (kept in payload for stable shape) |
| `type: array` + `minItems` / `maxItems` | Respects bounds, defaults to 1–5 items |
| `$ref` (schema + response) | Resolves `#/components/schemas/...` and `#/components/responses/...` |
| Recursive `$ref` (cycles) | Bounded recursion depth prevents stack overflow |

### Path parameter injection

When the response schema contains a field whose name matches a path parameter (e.g. `petId` in `/pets/{petId}`), the request value is injected into that field, coerced to the schema type. This keeps mock responses consistent with the request context.

### HTTP method coverage

All OpenAPI HTTP methods are wired up: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `OPTIONS`, `HEAD`, `TRACE`.

### Correct status codes

| Situation | Status |
|-----------|--------|
| Success | `200` |
| Path not in spec | `404` |
| Method not defined on path | `405` |
| No `200` response defined | `501` |


## CLI Reference

```text
Usage: mock-cli [OPTIONS] <SPEC>

Arguments:
  <SPEC>  Path to the OpenAPI spec file (YAML or JSON)

Options:
  -p, --port <PORT>   Port to listen on [default: 3333]
  -H, --host <HOST>   Host/IP address to bind to [default: 127.0.0.1]
  -h, --help          Print help
  -V, --version       Print version
```

Bind to all interfaces (useful inside containers):

```bash
mock-cli --host 0.0.0.0 --port 8080 spec.yaml
```

The spec format (JSON vs YAML) is detected automatically: by file extension first, then by content sniffing. Files like `spec.txt` work too.


## Roadmap

| Feature | Status |
|---------|--------|
| Basic mocking with dynamic data | Done |
| Path parameter injection | Done |
| Schema `format` / `enum` / `required` / array bounds | Done |
| `$ref` resolution & cycle safety | Done |
| Correct HTTP status codes | Done |
| Configurable bind host | Done |
| Graceful shutdown (Ctrl+C / SIGTERM) | Done |
| Hot reload on spec change | Planned |
| Request validation against spec | Planned |
| Custom faker rules via config | Planned |
| Multi-spec composition | Planned |


## Project Structure

```text
crates/
├── cli/      # Binary entry point & argument parsing
├── core/     # Spec parsing & data generation (no IO)
└── server/   # HTTP routing & request handling
```

### Development

```bash
git clone https://github.com/wanghao12345/mock-cli.git
cd mock-cli
cargo run -- examples/petstore.yaml
```


## License

Licensed under the MIT License.
