## [0.2.0] - 2026-07-08

### 🚀 Features

- *(ui)* Enable dark mode with theme toggle
- *(server)* Add /api/stats endpoint with real aggregates
- *(ui)* Dashboard with accurate stats, activity and service charts
- *(ui)* Waterfall with time axis, aligned gutter and a11y
- Correlate logs to traces via trace_id filter
- *(models)* Add full-fidelity domain models with schema v1 migration
- *(collector)* Convert full OTLP fidelity for traces, logs and metrics
- *(collector)* Add OTLP/HTTP logs and metrics with JSON, gzip and spec responses
- *(ui)* Render span events, links, resource attributes and distribution charts

### 🐛 Bug Fixes

- Avoid panic in ui asset response
- Avoid panic in cli path handling
- Persist model enums with stable strings instead of debug format
- Metrics error handling, log copy-paste, non-portable release flags
- Patch bytes and anyhow advisories, grant checks permission to audit
- *(ui)* Scope auto-refresh, add query boundary, settings flow
- *(ui)* Align enum types with API wire format
- *(collector)* Accept span/link flags and unknown OTLP/JSON fields
- *(ui)* A11y, theming, states, dataviz and perf overhaul

### 🚜 Refactor

- Harden storage with lock helper and transactional batches
- Remove faze-tui stub, dead API, and unused dependencies
- Dedupe enum mappings, batch collector inserts, storage and API helpers
- Split storage and routes modules, tighten server crate visibility
- Return io::Result from ApiServer::serve

### 📚 Documentation

- Add crate and module documentation

### ⚡ Performance

- Hex-encode bytes via lookup table and tighten convert tests
- Drop redundant clones in api routes

### 🎨 Styling

- Apply cargo fmt
- Apply cargo fmt

### ⚙️ Miscellaneous Tasks

- Use ubuntu for ci
- Inherit workspace lints in member crates
- Satisfy pedantic and nursery clippy lints across workspace
- Add security audit and MSRV jobs, drop no-op all-features flag
- *(ui)* Remove dead components and unused dependencies
## [0.1.1] - 2025-12-03

### 🚜 Refactor

- *(attributes)* Fix unknown in atrributes

### ⚙️ Miscellaneous Tasks

- Includes installs, for unix-like and windows
- *(scripts)* Update readme to installs
- *(readme)* Add screenshot in readme
- Release
## [0.1.0] - 2025-12-03

### 🚀 Features

- *(sqlite)* Migrate to sqlite
- Add ui in release and server
- Add ui in release and server
- Add ui in release and server
- *(collector)* Add a logs and metrics collector
- *(ui)* First usable ui
- *(ui)* First usable ui

### 💼 Other

- Alter name to 'faze'
- Update release build

### 🚜 Refactor

- *(cli)* Separation of responsibilities to maintenance
- *(cli)* Add a colored was added to make it prettier
- *(response-time-chart)* Fix chartbar in response time, btw: remove mock project-dir, remove dark-theme-toggle, waterfall always open, vitest for testing.

### 📚 Documentation

- Update readme

### ⚙️ Miscellaneous Tasks

- *(clippy)* Fix clippy warnings
- Add self-hosted github runner
- Run fmt
- Add ci for build ui
- Add ci for build ui
- Add ci for build ui
- Add ci for build ui
- *(cli)* Run clippy
- Add initial changelog and release.toml
- Update release
- Update release
- Update changelog
- Release
- Update ci workflow
- Add build ui in release.yml
- Add mold in build release
- *(crates)* Update description and license in all crates
- *(crates)* Fix problem with build .deb
- Release
