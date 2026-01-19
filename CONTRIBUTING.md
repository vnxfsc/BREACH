# Contributing to BREACH

Thank you for your interest in contributing to BREACH! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Commit Guidelines](#commit-guidelines)

---

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment. We expect all contributors to:

- Be respectful and considerate in all interactions
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Accept responsibility for mistakes and learn from them

---

## Getting Started

### Prerequisites

- **Rust** 1.75+ (for smart contracts)
- **Node.js** 18+ (for website)
- **pnpm** 8+ (package manager)
- **Solana CLI** 2.0+ (for blockchain development)
- **Flutter** 3.16+ (for mobile app)

### Repository Structure

```
BREACH/
├── docs/                    # Documentation
│   ├── BREACH_DESIGN_DOCUMENT.md
│   ├── TECHNICAL_SPECIFICATION.md
│   └── SMART_CONTRACT_SPECIFICATION.md
├── website/                 # Next.js website
│   ├── src/
│   └── public/
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md
└── CHANGELOG.md
```

---

## Development Setup

### Website Development

```bash
# Clone the repository
git clone https://github.com/vnxfsc/BREACH.git
cd BREACH/website

# Install dependencies
pnpm install

# Start development server
pnpm dev

# Build for production
pnpm build
```

### Smart Contract Development

```bash
# Navigate to contracts (when available)
cd BREACH/contracts

# Build contracts
cargo build-sbf

# Run tests
cargo test
```

---

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Use the bug report template
3. Include:
   - Clear description of the issue
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, browser, versions)

### Suggesting Features

1. Check existing feature requests
2. Use the feature request template
3. Explain the use case and benefits
4. Consider implementation complexity

### Contributing Code

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Write/update tests
5. Submit a pull request

---

## Pull Request Process

### Before Submitting

- [ ] Code follows project style guidelines
- [ ] All tests pass locally
- [ ] Documentation is updated (if needed)
- [ ] Commit messages follow conventions
- [ ] Branch is up to date with `main`

### PR Requirements

1. **Title**: Use conventional commit format
   - `feat: add new feature`
   - `fix: resolve bug`
   - `docs: update documentation`

2. **Description**: Include
   - What changes were made
   - Why changes were needed
   - How to test the changes

3. **Review**: At least one approval required

### Review Process

1. Automated checks must pass
2. Code review by maintainers
3. Address feedback promptly
4. Squash commits if requested

---

## Coding Standards

### Rust (Smart Contracts)

```rust
// Use descriptive names
pub fn calculate_damage(attacker: &Titan, defender: &Titan) -> u32 {
    // Implementation
}

// Document public functions
/// Calculates damage dealt in combat
/// 
/// # Arguments
/// * `attacker` - The attacking Titan
/// * `defender` - The defending Titan
/// 
/// # Returns
/// Damage value as u32
pub fn calculate_damage(...) -> u32 { ... }
```

### TypeScript (Website)

```typescript
// Use TypeScript types
interface TitanProps {
  id: string;
  name: string;
  class: ThreatClass;
}

// Use functional components
export function TitanCard({ id, name, class }: TitanProps) {
  return (
    <div className="titan-card">
      {/* ... */}
    </div>
  );
}
```

### General Guidelines

- Write self-documenting code
- Keep functions small and focused
- Handle errors gracefully
- Write meaningful comments (not obvious ones)
- Use consistent naming conventions

---

## Commit Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/):

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Formatting, no code change |
| `refactor` | Code restructuring |
| `perf` | Performance improvement |
| `test` | Adding/updating tests |
| `chore` | Maintenance tasks |

### Examples

```
feat(titan): add fusion mechanic

fix(website): resolve navbar mobile menu bug

docs: update smart contract specification

chore(deps): upgrade Next.js to 16.2
```

---

## Questions?

- Open a [Discussion](https://github.com/vnxfsc/BREACH/discussions)
- Create an [Issue](https://github.com/vnxfsc/BREACH/issues)

---

Thank you for contributing to BREACH! 🦖
