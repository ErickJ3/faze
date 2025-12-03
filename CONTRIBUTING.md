# Contributing to Faze

Thanks for considering contributing to Faze! We're building a local-first observability tool that's actually pleasant to use, and we'd love your help.

## Getting Started

### Prerequisites

You'll need:
- Rust 1.91 or later
- Protobuf compiler (`protoc`)
- Bun (for the UI - install from [bun.sh](https://bun.sh))

The easiest way to get everything set up is with Nix:

```bash
nix develop
```

This drops you into a shell with all the dependencies ready to go.

### First Build

```bash
# Clone the repo
git clone https://github.com/ErickJ3/faze
cd faze

# Build everything
cargo build

# Or use just for convenience
just build
```

### Project Structure

Faze is split into several crates:

- `faze` - Core library with storage and models
- `faze-collector` - OTLP protocol implementation (gRPC and HTTP)
- `faze-server` - Web server and API
- `faze-cli` - Command-line interface
- `faze-tui` - Terminal UI (work in progress)
- `ui/` - React-based web interface

## Development Workflow

### Running Locally

The quickest way to test your changes:

```bash
# Start the server with hot reload
just dev-server

# Or run the full stack (server + UI)
just dev
```

The server will be at `http://localhost:7070` and the OTLP collector at ports 4317 (gRPC) and 4318 (HTTP).

### Making Changes

1. **Create a branch** - Branch off from `main`:
   ```bash
   git checkout -b your-feature-name
   ```

2. **Make your changes** - Write code, fix bugs, add features. Keep commits focused and atomic.

3. **Test your changes**:
   ```bash
   # Run all tests (Rust + UI)
   cargo test --workspace
   cd ui && bun run test
   # or
   just test

   # For UI tests with interactive interface
   just test-ui

   # For UI tests with coverage
   just test-ui-coverage
   ```

4. **Check code quality**:
   ```bash
   # Format code
   cargo fmt --all

   # Run clippy
   cargo clippy --workspace --all-targets

   # Or run both
   just check
   ```

5. **Commit** - Write clear commit messages that explain what and why:
   ```
   feat: add filtering by service name to traces endpoint

   Users can now filter traces by service name using the ?service=name
   query parameter. This makes it easier to debug specific services.
   ```

### Sending a Pull Request

1. Push your branch to your fork
2. Open a PR against `main`
3. Fill in the PR template (if we have one)
4. Wait for CI to run - it checks formatting, linting, and tests
5. Address any review comments

We'll try to review PRs quickly, but keep in mind this is a side project and response times might vary.

## What to Work On

### Good First Issues

Look for issues tagged with `good first issue`. These are usually:
- Small, well-defined tasks
- Good for getting familiar with the codebase
- Have clear acceptance criteria

### Ideas for Contributions

Here are some areas where we'd love help:

**Features:**
- Improve HTTP OTLP support (it's partial right now)
- Add metrics visualization
- Implement log filtering and search
- Better error handling and user feedback
- TUI implementation

**Performance:**
- Query optimization
- Better indexing strategies
- Reduce memory usage for large traces

**Developer Experience:**
- Better documentation
- More examples
- Integration guides for common frameworks

**UI/UX:**
- Make the web UI prettier and more intuitive
- Better mobile support
- Dark mode improvements

Don't see something you want to work on? Open an issue to discuss it first.

## Code Guidelines

### Rust Code

- Follow standard Rust conventions
- Use `cargo fmt` for formatting
- Keep clippy happy (we run with `-D warnings`)
- Write tests for new functionality
- Add doc comments for public APIs

We're not super strict, but try to:
- Keep functions focused and small
- Avoid unnecessary complexity
- Prefer readability over cleverness
- Use meaningful variable names

### TypeScript/React Code

For the UI:
- Use TypeScript, not JavaScript
- Follow the existing component structure
- Keep components small and focused
- Use Tailwind for styling

### Commit Messages

We don't enforce a strict format, but good commit messages help:

```
type: brief summary (50 chars or less)

More detailed explanation if needed. Wrap at 72 characters.
Explain the problem you're solving and why you chose this approach.

Fixes #123
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`

## Testing

### Running Tests

```bash
# All tests (Rust + UI)
just test

# Only Rust tests
cargo test --workspace

# Specific Rust package
cargo test -p faze-collector

# With output
cargo test -- --nocapture

# UI tests only
cd ui && bun run test

# UI tests with interactive UI
just test-ui

# UI tests with coverage
just test-ui-coverage
```

### Writing Tests

**Rust tests:**

- Add unit tests in the same file as the code (`#[cfg(test)]` modules)
- Add integration tests in `tests/` directories
- Test happy paths and error cases
- Keep tests fast and deterministic

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new("test-span", "trace-123", "span-456");
        assert_eq!(span.name, "test-span");
    }
}
```

**UI tests:**

We use Vitest with React Testing Library. Place tests next to your components:

```typescript
// MyComponent.test.tsx
import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { MyComponent } from './MyComponent'

describe('MyComponent', () => {
  it('renders the title', () => {
    render(<MyComponent title="Hello" />)
    expect(screen.getByText('Hello')).toBeInTheDocument()
  })
})
```

Tips for UI tests:
- Test user interactions, not implementation details
- Use `screen.getByRole` and `screen.getByLabelText` when possible
- Mock API calls to keep tests fast
- Use `just test-ui` for the interactive interface while developing

## Database Changes

If you're modifying the database schema:

1. Update `faze/src/storage/schema.rs`
2. Consider migration impact (we don't have migrations yet, but we should)
3. Update tests
4. Document breaking changes

## OTLP Protocol

When working on the collector:

- Follow the [OTLP specification](https://opentelemetry.io/docs/specs/otlp/)
- Test with real OTLP clients when possible
- Both gRPC and HTTP should behave the same way

## Getting Help

Stuck? Have questions?

- Open a discussion on GitHub
- Ask in issues (even if you're not reporting a bug)
- Tag maintainers if you need specific help

We're happy to help and answer questions. There are no stupid questions.

## Code of Conduct

Be nice. That's it.

More specifically:
- Be respectful and inclusive
- Welcome newcomers
- Focus on the code, not the person
- Assume good intentions

We want Faze to be a welcoming project where everyone feels comfortable contributing.

## License

By contributing to Faze, you agree that your contributions will be licensed under both MIT and Apache-2.0 licenses, matching the project's license.
