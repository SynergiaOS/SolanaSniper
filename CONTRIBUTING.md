# 🤝 Contributing to SniperBot 2.0

Thank you for your interest in contributing to SniperBot 2.0! This document provides guidelines and information for contributors.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Guidelines](#contributing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)

## 📜 Code of Conduct

### Our Pledge

We are committed to making participation in this project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

**Positive behavior includes:**
- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable behavior includes:**
- The use of sexualized language or imagery
- Trolling, insulting/derogatory comments, and personal or political attacks
- Public or private harassment
- Publishing others' private information without explicit permission
- Other conduct which could reasonably be considered inappropriate

## 🚀 Getting Started

### Prerequisites

- **Rust 1.75+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: Version control system
- **Docker**: For containerized development (optional)
- **Basic knowledge**: Rust, async programming, trading concepts

### First Contribution

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Create a feature branch** from `main`
4. **Make your changes** following our guidelines
5. **Test your changes** thoroughly
6. **Submit a pull request**

### Good First Issues

Look for issues labeled with:
- `good first issue`: Perfect for newcomers
- `help wanted`: Community help needed
- `documentation`: Documentation improvements
- `bug`: Bug fixes needed

## 💻 Development Setup

### Local Environment

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/SniperBot.git
cd SniperBot

# Add upstream remote
git remote add upstream https://github.com/SynergiaOS/SniperBot.git

# Install dependencies and build
cargo build

# Run tests
cargo test

# Run in development mode
cargo run -- --dry-run --log-level debug
```

### Development Tools

```bash
# Install useful tools
cargo install cargo-watch cargo-tarpaulin cargo-audit

# Auto-rebuild on changes
cargo watch -x check -x test -x run

# Generate test coverage
cargo tarpaulin --out Html

# Security audit
cargo audit
```

### Environment Configuration

```bash
# Copy environment template
cp .env.template .env

# Edit with your development keys
nano .env
```

## 📝 Contributing Guidelines

### Types of Contributions

#### 🐛 **Bug Reports**
- Use the bug report template
- Include steps to reproduce
- Provide system information
- Include relevant logs

#### ✨ **Feature Requests**
- Use the feature request template
- Explain the use case
- Provide implementation ideas
- Consider backward compatibility

#### 🔧 **Code Contributions**
- Follow coding standards
- Include comprehensive tests
- Update documentation
- Ensure CI passes

#### 📚 **Documentation**
- Fix typos and grammar
- Improve clarity and examples
- Add missing documentation
- Update outdated information

### Branch Naming Convention

```
feature/description-of-feature
bugfix/description-of-bug
docs/description-of-docs-change
refactor/description-of-refactor
```

Examples:
- `feature/add-meteora-integration`
- `bugfix/fix-websocket-reconnection`
- `docs/update-api-documentation`

## 🔄 Pull Request Process

### Before Submitting

1. **Sync with upstream**:
   ```bash
   git fetch upstream
   git checkout main
   git merge upstream/main
   ```

2. **Create feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes** following our guidelines

4. **Test thoroughly**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

5. **Commit with clear messages**:
   ```bash
   git commit -m "feat: add meteora DLMM integration
   
   - Add MeteoraClient for DLMM API access
   - Implement pool detection and monitoring
   - Add comprehensive tests
   - Update documentation
   
   Closes #123"
   ```

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
```

### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by maintainers
3. **Testing** in development environment
4. **Approval** by at least one maintainer
5. **Merge** into main branch

## 🎨 Coding Standards

### Rust Style Guide

Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/) and use:

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings
```

### Code Organization

```rust
// File structure
src/
├── main.rs                 // Application entry point
├── lib.rs                  // Library root (if applicable)
├── models/                 // Data structures
├── data_fetcher/          // Data acquisition
├── strategy/              // Trading strategies
├── risk_management/       // Risk controls
├── execution/             // Order execution
├── analytics_aggregator/  // AI/ML integration
└── utils/                 // Utilities

// Module structure
pub mod module_name {
    // Public types first
    pub struct PublicStruct { }
    
    // Private types
    struct PrivateStruct { }
    
    // Public functions
    pub fn public_function() { }
    
    // Private functions
    fn private_function() { }
    
    // Tests at the end
    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[test]
        fn test_function() { }
    }
}
```

### Naming Conventions

```rust
// Types: PascalCase
struct MarketData { }
enum SignalType { }
trait Strategy { }

// Functions and variables: snake_case
fn calculate_signal_strength() -> f64 { }
let market_cap = 100000.0;

// Constants: SCREAMING_SNAKE_CASE
const MAX_POSITION_SIZE: f64 = 10000.0;

// Modules: snake_case
mod data_fetcher;
mod risk_management;
```

### Error Handling

```rust
// Use Result types for fallible operations
pub type TradingResult<T> = Result<T, TradingError>;

// Implement proper error types
#[derive(Debug, thiserror::Error)]
pub enum TradingError {
    #[error("Exchange error: {0}")]
    ExchangeError(String),
    
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: f64, available: f64 },
}

// Use ? operator for error propagation
fn process_order(order: &Order) -> TradingResult<String> {
    let validated_order = validate_order(order)?;
    let order_id = submit_order(&validated_order)?;
    Ok(order_id)
}
```

### Documentation

```rust
/// Calculate signal strength based on market data
/// 
/// # Arguments
/// 
/// * `market_data` - Current market data for analysis
/// * `context` - Strategy context with portfolio and conditions
/// 
/// # Returns
/// 
/// Signal strength between 0.0 and 1.0, where 1.0 is strongest
/// 
/// # Examples
/// 
/// ```
/// let strength = calculate_signal_strength(&market_data, &context);
/// assert!(strength >= 0.0 && strength <= 1.0);
/// ```
pub fn calculate_signal_strength(
    market_data: &MarketData,
    context: &StrategyContext,
) -> f64 {
    // Implementation
}
```

## 🧪 Testing Guidelines

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    // Helper functions
    fn create_test_market_data() -> MarketData {
        MarketData {
            symbol: "TEST/SOL".to_string(),
            price: 0.001,
            volume: 10000.0,
            timestamp: Utc::now(),
            source: DataSource::Test,
        }
    }
    
    // Unit tests
    #[test]
    fn test_signal_calculation() {
        let market_data = create_test_market_data();
        let result = calculate_signal(&market_data);
        assert!(result.is_ok());
    }
    
    // Async tests
    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
    
    // Integration tests
    #[tokio::test]
    async fn test_strategy_integration() {
        let strategy = TestStrategy::new();
        let context = create_test_context();
        let signal = strategy.analyze(&context).await;
        assert!(signal.is_ok());
    }
}
```

### Test Coverage

- **Unit tests**: Test individual functions and methods
- **Integration tests**: Test component interactions
- **End-to-end tests**: Test complete workflows
- **Performance tests**: Test performance characteristics

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=4

# Generate coverage report
cargo tarpaulin --out Html
```

## 📚 Documentation

### Types of Documentation

1. **Code Documentation**: Inline comments and doc comments
2. **API Documentation**: Generated from doc comments
3. **User Documentation**: Guides and tutorials
4. **Developer Documentation**: Architecture and design docs

### Documentation Standards

```rust
// Good: Explains why, not just what
/// Calculates position size based on risk management rules.
/// Uses Kelly criterion for optimal sizing when volatility data is available.
pub fn calculate_position_size(signal: &Signal, portfolio: &Portfolio) -> f64 {
    // Use Kelly criterion if we have volatility data
    if let Some(volatility) = signal.volatility {
        kelly_position_size(signal, portfolio, volatility)
    } else {
        // Fallback to fixed percentage
        portfolio.total_value * 0.02
    }
}

// Bad: States the obvious
/// Sets the value to 5
let value = 5;
```

### Updating Documentation

When making changes:

1. **Update doc comments** for modified functions
2. **Update README** if user-facing changes
3. **Update API docs** if API changes
4. **Add examples** for new features
5. **Update configuration docs** if config changes

## 🏷️ Issue Labels

### Priority Labels
- `priority/critical`: Critical issues requiring immediate attention
- `priority/high`: High priority issues
- `priority/medium`: Medium priority issues
- `priority/low`: Low priority issues

### Type Labels
- `type/bug`: Bug reports
- `type/feature`: Feature requests
- `type/enhancement`: Improvements to existing features
- `type/documentation`: Documentation improvements
- `type/question`: Questions about usage

### Status Labels
- `status/needs-triage`: Needs initial review
- `status/in-progress`: Currently being worked on
- `status/blocked`: Blocked by external dependencies
- `status/ready-for-review`: Ready for code review

## 🎯 Development Focus Areas

### High Priority
- **Performance optimization**: Reduce latency and improve throughput
- **Strategy development**: New trading strategies
- **Risk management**: Enhanced risk controls
- **Data source integration**: Additional exchanges and platforms

### Medium Priority
- **UI/UX improvements**: Better user interface
- **Documentation**: Comprehensive guides and examples
- **Testing**: Improved test coverage
- **Monitoring**: Enhanced observability

### Future Considerations
- **Mobile app**: Mobile trading interface
- **Advanced AI**: Machine learning integration
- **Multi-chain**: Support for other blockchains
- **Social features**: Community and sharing features

## 📞 Getting Help

### Communication Channels
- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Email**: synergiaos@outlook.com for private matters

### Response Times
- **Critical bugs**: Within 24 hours
- **General issues**: Within 1 week
- **Feature requests**: Within 2 weeks
- **Documentation**: Within 1 week

## 🙏 Recognition

Contributors will be recognized in:
- **README.md**: Contributors section
- **CHANGELOG.md**: Release notes
- **GitHub**: Contributor graphs and statistics

Thank you for contributing to SniperBot 2.0! 🚀
