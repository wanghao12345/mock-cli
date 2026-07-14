<div align="center">

# 🚀 mock-cli

**Blazing fast OpenAPI Mock Server in Rust.**  
Start mocking in <50ms. Zero config. Dynamic fake data.

[![Crates.io](https://img.shields.io/crates/v/mock-cli.svg)](https://crates.io/crates/mock-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![CI](https://github.com/your-username/mock-cli/actions/workflows/release.yml/badge.svg)](https://github.com/your-username/mock-cli/actions)

[Installation](#-installation) • [Quick Start](#-quick-start) • [Docs](https://docs.mock-cli.dev) • [Pro Version](#-pro-version)

</div>

---

## ✨ Why mock-cli?

Most OpenAPI mock servers are slow, require Docker, or return static stubs. `mock-cli` is different:

- ⚡ **<50ms Startup** — Native Rust binary, no JVM/Node/Docker overhead
- 🎲 **Dynamic Fake Data** — Generates realistic random data based on your schema types
- 🔥 **Hot Reload** — Edit your spec file and see changes instantly without restart
- 📦 **Zero Dependencies** — Single binary, works offline, no runtime required
- 🌍 **Cross-Platform** — macOS / Linux / Windows with native installers


## 📦 Installation

Choose your preferred method:

```bash
# Homebrew (macOS/Linux)
brew install your-username/tap/mock-cli

# npm (requires Node.js)
npm install -g @yourname/mock-cli

# Shell script (Linux/macOS)
curl -sL https://github.com/your-username/mock-cli/releases/latest/download/mock-cli-installer.sh | sh

# PowerShell (Windows)
irm https://github.com/your-username/mock-cli/releases/latest/download/mock-cli-installer.ps1 | iex

# Cargo (Rust developers)
cargo install mock-cli
```
Or download pre-built binaries from [GitHub Releases](https://github.com/your-username/mock-cli/releases).

## 🏃 Quick Start

**1. Create a minimal OpenAPI spec:**

```yaml
# petstore.yaml
openapi: 3.0.3
info: { title: Petstore, version: 1.0.0 }
paths:
  /pets/{id}:
    get:
      responses:
        '200':
          description: OK
          content:
            application/json:
              schema:
                type: object
                properties:
                  id: { type: integer }
                  name: { type: string }
                  vaccinated: { type: boolean }
```

**2. Start the mock server:**

```bash
mock-cli petstore.yaml
```

```text
✓ Loaded "petstore.yaml" (1 paths)
✓ Listening on http://localhost:3333
```

**3. Get dynamic fake data:**

```bash
curl http://localhost:3333/pets/123
# {"id":482,"name":"Dr. Margret Kihn","vaccinated":true}

curl http://localhost:3333/pets/456
# {"id":73,"name":"Sarah Connor","vaccinated":false}
```

Every request returns **different, schema-compliant** fake data. No static stubs.

## ⚙️ CLI Reference

```text
USAGE: mock-cli [OPTIONS] 

ARGS:
      OpenAPI spec file path (YAML or JSON)

OPTIONS:
  -p, --port    Port to listen on [default: 3333]
  -h, --help          Print help
  -V, --version       Print version
```

## 🗺️ Roadmap

| Feature | Status | Target |
|---------|--------|--------|
| Basic GET mocking with dynamic data | ✅ Done | v0.1.0 |
| Hot reload on spec change | 🔜 Next | v0.2.0 |
| `$ref` resolution & circular ref safety | 📋 Planned | v0.3.0 |
| Request validation against spec | 📋 Planned | v0.3.0 |
| Custom faker rules via config | 📋 Planned | v1.0.0 |
| Multi-spec composition | 📋 Planned | v1.0.0 |
| Cloud-hosted mock endpoints | 📋 Planned | v1.0.0 |

## 💎 Pro Version

A commercial **mock-cli Pro** is planned for teams that need advanced features:

- Custom data generation rules (e.g., specific email domains, ID formats)
- Multiple spec composition & override layers
- Cloud-hosted shareable mock endpoints
- Team collaboration & access control
- Priority support & SLA

👉 **[Join the waitlist](https://mock-cli.dev/pro)** to get early access and shape the feature set.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
git clone https://github.com/your-username/mock-cli.git
cd mock-cli
cargo run -- examples/petstore.yaml
```

### Project Structure

```text
crates/
├── cli/      # Binary entry point & argument parsing
├── core/     # Spec parsing & data generation (no IO)
└── server/   # HTTP routing & request handling
```

## 📄 License

Licensed under the [MIT License](./LICENSE).

---

<div align="center">

**If mock-cli saves you time, please ⭐ star this repo!**

Made with ❤️ by [Your Name](https://github.com/your-username)

</div>
```
