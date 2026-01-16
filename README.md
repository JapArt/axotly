# Axotly CLI

**Fast, reliable, and expressive API testing — designed for developer happiness.**

Axotly is a fast, developer-first CLI tool for testing HTTP (and future GraphQL) APIs using a clear, declarative DSL. Tests focus on what you expect from an API rather than how to assert it, making them readable, diff-friendly, and easy to review.

Tests are written in plain .ax files, organized in folders alongside your code. Axotly automatically discovers and runs them concurrently, producing clear pass/fail output with actionable error messages. It’s designed to feel like curl with structure—simple, deterministic, and local-first.

Key features include REST API testing, JSON assertions and expressive expectations. Planned features include GraphQL support, variables, environment configuration, and CI integrations.

Axotly is a Rust-based CLI (installed via Cargo), runs entirely locally with no accounts required, and is currently in Beta. It’s open source under the MIT license and focused on one goal: making API behavior explicit, testable, and trustworthy.

![Axotly demo](./demo.gif)
---

---

## Installation

Axotly is distributed as a Rust CLI.

### Prerequisites

- Rust (1.75+)
- Cargo

If you don’t have Rust installed, get it from [https://rustup.rs](https://rustup.rs)

### Install with Cargo

```bash
cargo install axotly
```

Once installed, verify it works:

```bash
axotly --help
```

### Install for Linux x86_64

```bash
curl -fsSL https://raw.githubusercontent.com/JapArt/axotly/main/install.sh | bash

```

Axotly runs completely locally. No accounts, no logins, no network calls beyond the APIs you test.

---

## How to use

```

/// Assuming you have a folder `examples/` with Axotly test files:

> axotly -f examples

Axotly — API tests
Running 6 tests...

examples/test2.ax
✓ GET request with query (660ms)
✗ POST create a resource (687ms)
✗ PUT update a resource (863ms)
✗ PATCH partial update (1.07s)
✓ DELETE a resource (659ms)
✓ GET with headers (1.38s)

Failures

1) POST create a resource (687ms)

- Path 'body.name' not found
- Path 'body.role' not found

2) PUT update a resource (863ms)

- Path 'body.role' not found

3) PATCH partial update (1.07s)

- Path 'body.active' not found

────────────────────────────────────
Results
✓ Passed: 3
✗ Failed: 3
⏱ Duration: 4.17s
────────────────────────────────────
Completed in: 1.42s

```

---

## Test Files & Project Structure

Axotly tests are written in plain text **files** with the .ax extension.

- Tests live alongside your code
- Files can be organized into **folders and subfolders**
- Everything is **Git-friendly** (diffable, reviewable, versioned)

This makes it easy to:

- Review test changes in pull requests
- Share tests across teams
- Keep API tests close to the services they validate

A typical structure might look like:

```

api-tests/
  users/
    get_user.ax
    create_user.ax
  auth/
    login.ax
    refresh_token.ax

```

Axotly will automatically discover and run all `.ax` files under the given directory.

---

## Example

A simple Axotly test looks like this:

```axotly
TEST Create a resource           <--- Test name
  POST https://httpbin.org/post  <--- Request (METHOD url)
  Content-Type: application/json <--- Headers, one on each line

  BODY                           <--- Body of the request 
  {
    "name": "Axotly",
    "role": "tester"
  }
  BODYEND
  
  EXPECT status == 200          <--- Expects, one on each line
  EXPECT body.name == "Axotly"
  EXPECT body.role == "tester"
END
```

You can find more examples in the `examples/` folder of the repository.

---

## Why Axotly?

This is a personal project aimed to use the simplicity of curl, with the structure and assertions needed for real API testing.

### Core Principles

- **Expressive DSL** – Tests read like specifications
- **Fast by default** – Minimal overhead with concurrent test execution
- **Deterministic** – No hidden magic, no shared mutable state
- **Local-first** – Runs entirely on your machine, no logins or accounts required
- **Developer-friendly** – Clear failures, clean output, simple mental model

---

## Project Status

Axotly is currently in **Beta**.

- The tool is usable and stable for real-world API testing
- APIs and DSL syntax may still evolve
- Feedback from early users is highly encouraged

Axotly is already being used locally and in development workflows, and your input will directly shape its future.

---

## What’s Next

The immediate focus is on features that make Axotly practical for real-world teams and CI pipelines:

- **Variables in the DSL** – Reuse values, parameterize requests, and reduce duplication
- **Environment-based support** – Environment-based configuration for local, staging, and production setups
- **CI integration** – First-class support for running Axotly in automated pipelines

These improvements will keep Axotly simple while making it production-ready.

---

## Open Source

Axotly is open source under the **MIT License**.

You are free to:

- Use it
- Modify it
- Distribute it

Attribution is appreciated ❤️

---

## Contributing

Contributions, ideas, and feedback are welcome.

- Open issues for bugs or ideas
- Propose DSL improvements
- Discuss design decisions

Please keep discussions constructive and developer-focused.

---

## Name & Inspiration

**Axotly** is inspired by the *axolotl*, a unique animal known for its ability to regenerate and adapt. I used to have one.

---

## License

MIT © Juan Artau
