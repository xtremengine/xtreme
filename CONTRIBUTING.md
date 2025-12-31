# Contributing to Xtreme Engine

Thank you for your interest in contributing to Xtreme Engine! This document provides guidelines for contributions.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/your-username/xtreme-engine/issues)
2. If not found, create a new issue with:
   - Clear and descriptive title
   - Steps to reproduce the problem
   - Expected behavior vs actual behavior
   - Rust version and operating system
   - Error logs (if applicable)

### Suggesting Features

1. Open an issue with the `enhancement` tag
2. Describe the feature in detail
3. Explain why it would be useful for the project
4. If possible, include usage examples

### Pull Requests

1. Fork the repository
2. Create a branch for your feature:
   ```bash
   git checkout -b feature/my-feature
   ```
3. Make your changes following the code style
4. Write or update tests
5. Ensure all tests pass:
   ```bash
   cargo test --lib
   cargo clippy
   cargo fmt --check
   ```
6. Commit your changes:
   ```bash
   git commit -m "feat: feature description"
   ```
7. Push to your branch:
   ```bash
   git push origin feature/my-feature
   ```
8. Open a Pull Request

## Code Style

### Rust

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### Commits

We use [Conventional Commits](https://www.conventionalcommits.org/):

| Prefix | Description |
|--------|-------------|
| `feat:` | New feature |
| `fix:` | Bug fix |
| `docs:` | Documentation |
| `style:` | Formatting (no code change) |
| `refactor:` | Refactoring |
| `test:` | Adding/fixing tests |
| `chore:` | General maintenance |

### Module Structure

- Keep files between **300-500 lines**
- One module = one responsibility
- Prefer composition over inheritance
- Document public APIs with `///`

## Architecture

### ECS (Entity Component System)

```
src/core/
├── entity.rs     # Entity management
├── component.rs  # Component storage
├── world.rs      # Main container
├── query.rs      # Query system
└── system.rs     # Scheduler
```

### Principles

1. **Modularity** - Each module should be independent
2. **Zero-cost abstractions** - Performance and ergonomics
3. **Data-oriented design** - Contiguous data in memory
4. **Isolation** - Easy to maintain and test

## Tests

### Running Tests

```bash
# Unit tests
cargo test --lib

# Tests with output
cargo test --lib -- --nocapture

# Specific test
cargo test --lib test_name
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let input = ...;

        // Act
        let result = function(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

## Documentation

- Document all public APIs
- Include code examples when useful
- Update the README if necessary

```rust
/// Brief description of the function.
///
/// # Arguments
///
/// * `param` - Parameter description
///
/// # Returns
///
/// Return description
///
/// # Examples
///
/// ```
/// let result = my_function(42);
/// assert_eq!(result, 84);
/// ```
pub fn my_function(param: i32) -> i32 {
    param * 2
}
```

## Review Process

1. At least 1 approval required
2. CI must pass (tests, clippy, fmt)
3. No conflicts with main branch
4. Documentation updated

## Code of Conduct

- Be respectful and inclusive
- Accept constructive criticism
- Focus on what is best for the project
- Show empathy towards other contributors

## Questions?

Open an issue with the `question` tag or contact the maintainers.

---

Thank you for contributing!
