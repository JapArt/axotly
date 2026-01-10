# Axotly CLI

**Fast, reliable, and expressive API testing — designed for developer happiness.**

Axotly is a modern API testing tool built for developers who value clarity, speed, and confidence. It lets you write readable, declarative tests for HTTP and GraphQL APIs using a purpose-built DSL that focuses on *what* you expect, not *how* to assert it.

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

────────────────────────────────────
Results
✓ Passed: 3
✗ Failed: 3
⏱ Duration: 4.17s
────────────────────────────────────

Failures

1) POST create a resource (687ms)
  - Path 'body.name' not found
  - Path 'body.role' not found

2) PUT update a resource (863ms)
  - Path 'body.role' not found

3) PATCH partial update (1.07s)
  - Path 'body.active' not found

Completed in: 1.42s
```

---

## Test Files & Project Structure

Axotly tests are written in plain text \*\* files\*\* with the .ax extension.

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
TEST POST create a resource
  POST https://httpbin.org/post
  Content-Type: application/json

  BODY 
  {
    "name": "Axotly",
    "role": "tester"
  }
  BODYEND
  
  EXPECT status == 200
  EXPECT body.name == "Axotly"
  EXPECT body.role == "tester"
END
```

You can find more examples in the `examples/` folder of the repository.

---

## Why Axotly?

Axotly is different.

Most API testing tools either:

- Grow into heavy, stateful workspaces that are hard to version and automate
- Or stay so low-level that even simple tests become repetitive and error-prone

Axotly is the middle ground.

The simplicity of curl, with the structure and assertions needed for real API testing.

### ✨ Core Principles

- **Expressive DSL** – Tests read like specifications
- **Fast by default** – Minimal overhead with concurrent test execution
- **Deterministic** – No hidden magic, no shared mutable state
- **Local-first** – Runs entirely on your machine, no logins or accounts required
- **Developer-first** – Clear failures, clean output, simple mental model

---

## Features

- 🚀 HTTP API testing (REST)
- 🧪 Declarative expectations
- 📦 JSON body assertions
- 🧠 Clear, actionable failure reports
- ⚡ Designed for CI and local workflows

Planned / in progress:

- GraphQL support
- Variables & value extraction
- Richer reports (JSON / HTML)

---

## How It Works

- Axotly scans your project for test files
- Each test is parsed into a request + expectations
- Requests are executed concurrently when possible
- Expectations are evaluated against each response
- Results are rendered immediately with clear feedback

Fast execution is achieved without sacrificing determinism.

No shared state. No implicit retries. No surprises.

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

> Axotly runs completely locally. No accounts, no logins, no network calls beyond the APIs you test.

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

## Philosophy

Axotly is not trying to be:

- A browser automation tool
- A general-purpose scripting framework
- A replacement for unit tests

It *is* focused on one thing:

> **Making API behavior explicit, testable, and trustworthy.**

---

## Contributing

Contributions, ideas, and feedback are welcome.

- Open issues for bugs or ideas
- Propose DSL improvements
- Discuss design decisions

Please keep discussions constructive and developer-focused.

---

## Name & Inspiration

**Axotly** is inspired by the *axolotl*, a unique animal known for its ability to regenerate and adapt.

Like the axolotl, Axotly aims to:

- Be resilient
- Stay simple
- Adapt as your API evolves

---

## License

MIT © Juan Artau
