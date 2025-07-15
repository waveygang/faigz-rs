# Contributing to faigz-rs

Thank you for your interest in contributing to faigz-rs! This document provides guidelines for contributing to the project.

## Development Setup

1. **Clone the repository:**
   ```bash
   git clone --recursive https://github.com/waveygang/faigz-rs
   cd faigz-rs
   ```

2. **Install dependencies:**
   ```bash
   # On Ubuntu/Debian
   sudo apt-get install build-essential libhts-dev pkg-config libclang-dev
   ```

3. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup component add clippy rustfmt
   ```

4. **Build and test:**
   ```bash
   cargo build
   cargo test
   ```

## Code Style

- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Follow Rust naming conventions
- Add documentation for public APIs
- Write tests for new functionality

## Testing

- Run `cargo test` for all tests
- Run `cargo test --release` for release mode testing
- Add unit tests for new functions
- Add integration tests for complex workflows
- Test both with and without HTSlib when possible

## Submitting Changes

1. **Fork the repository** on GitHub
2. **Create a feature branch** from main:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make your changes** following the code style guidelines
4. **Add tests** for your changes
5. **Run the full test suite:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   cargo build --examples
   ```
6. **Commit your changes** with a descriptive message
7. **Push to your fork** and create a pull request

## Pull Request Guidelines

- Include a clear description of the problem and solution
- Reference any related issues
- Add tests for new functionality
- Update documentation if needed
- Make sure CI passes

## Reporting Issues

- Use the GitHub issue tracker
- Include system information (OS, Rust version, HTSlib version)
- Provide a minimal example that reproduces the issue
- Include relevant log output

## License

By contributing to faigz-rs, you agree that your contributions will be licensed under the MIT License.