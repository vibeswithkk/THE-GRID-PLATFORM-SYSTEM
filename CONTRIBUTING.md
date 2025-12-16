# Contributing to TGP

Thank you for your interest in contributing to TGP (The Grid Platform)! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Contributions](#making-contributions)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Community](#community)

---

## Code of Conduct

We are committed to providing a welcoming and inclusive experience for everyone. Please be respectful and constructive in all interactions.

**Expected behavior:**
- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on what is best for the community
- Show empathy towards other community members

---

## Getting Started

### Prerequisites

- **Rust** 1.75 or higher ([rustup.rs](https://rustup.rs/))
- **Docker** for job execution testing
- **Protocol Buffers** compiler (`protoc`)

### Quick Setup

```bash
# Clone the repository
git clone https://github.com/vibeswithkk/TGP.git
cd TGP

# Build all components
cargo build

# Run tests
cargo test

# Check code style
cargo clippy
cargo fmt --check
```

---

## Development Setup

### Project Structure

```
TGP/
├── core/
│   ├── scheduler/       # Economic Scheduler (Rust)
│   ├── cost-engine/     # Formula 4.1 TCO calculator
│   └── optimizer/       # Placement optimization
├── worker/              # Worker agent
├── test-client/         # gRPC test client
├── proto/               # Protocol buffer definitions
├── docs/                # Documentation
│   ├── WHITEPAPER.md    # Technical whitepaper
│   └── blog/            # Blog posts
└── THE-GRID-PLATFORM-website/  # Next.js dashboard
```

### Building Components

```bash
# Build scheduler
cargo build --package tgp-scheduler

# Build worker
cargo build --package tgp-worker

# Build all in release mode
cargo build --release
```

---

## Making Contributions

### Types of Contributions

We welcome all types of contributions:

| Type | Description |
|------|-------------|
| **Bug fixes** | Fix issues in existing code |
| **Features** | Implement new functionality |
| **Documentation** | Improve or add documentation |
| **Tests** | Add or improve tests |
| **Performance** | Optimize existing code |

### Finding Issues

- Check [GitHub Issues](https://github.com/vibeswithkk/TGP/issues) for open tasks
- Look for `good first issue` label for beginner-friendly tasks
- Look for `help wanted` label for priority items

### Creating Issues

Before creating a new issue:
1. Search existing issues to avoid duplicates
2. Use the appropriate issue template
3. Provide clear reproduction steps for bugs
4. Include relevant system information

---

## Pull Request Process

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/TGP.git
cd TGP
git remote add upstream https://github.com/vibeswithkk/TGP.git
```

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 3. Make Changes

- Write clean, documented code
- Add tests for new functionality
- Update documentation as needed

### 4. Test Your Changes

```bash
# Run all tests
cargo test

# Run linter
cargo clippy --all-targets

# Check formatting
cargo fmt --check
```

### 5. Commit with Descriptive Messages

```bash
git commit -m "feat: add spot instance pricing support"
# or
git commit -m "fix: correct TCO calculation for idle resources"
```

**Commit message prefixes:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Adding or updating tests
- `refactor:` - Code refactoring
- `perf:` - Performance improvements

### 6. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

---

## Coding Standards

### Rust Style Guide

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Document public APIs with doc comments

### Example

```rust
/// Calculate total cost using Formula 4.1
///
/// # Arguments
/// * `compute_cost` - The compute cost in USD
/// * `data_cost` - The data transfer cost in USD
/// * `idle_cost` - The idle opportunity cost in USD
///
/// # Returns
/// Total cost as TotalCost struct
pub fn calculate_total_cost(
    compute_cost: f64,
    data_cost: f64,
    idle_cost: f64,
) -> TotalCost {
    TotalCost::new(compute_cost, data_cost, idle_cost)
}
```

### Testing

- Write unit tests for new functions
- Write integration tests for new features
- Aim for meaningful test coverage

---

## Community

### Getting Help

- **GitHub Discussions**: Ask questions and share ideas
- **GitHub Issues**: Report bugs or request features

### Recognition

All contributors are recognized in our release notes. Significant contributors may be invited to become maintainers.

---

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

---

Thank you for contributing to TGP! Together we're building the future of economic job scheduling.
