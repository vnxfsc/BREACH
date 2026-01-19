# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| < 1.0   | :x:                |

---

## Reporting a Vulnerability

We take security seriously at BREACH. If you discover a security vulnerability, please report it responsibly.

### How to Report

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead, please report via:

1. **GitHub Security Advisory**: [Report a vulnerability](https://github.com/vnxfsc/BREACH/security/advisories/new)
2. **Private Issue**: Contact maintainers via GitHub

### What to Include

Please provide as much information as possible:

- Type of vulnerability (e.g., smart contract bug, XSS, authentication bypass)
- Affected component (website, smart contract, API)
- Step-by-step reproduction instructions
- Proof of concept (if available)
- Potential impact assessment
- Suggested fix (if any)

### Response Timeline

| Stage | Timeline |
|-------|----------|
| Initial Response | Within 48 hours |
| Status Update | Within 7 days |
| Resolution Target | Within 30 days |
| Public Disclosure | After fix is deployed |

---

## Scope

### In Scope

- Smart contracts on Solana (mainnet/devnet)
- Official website (breach.game)
- Backend API services
- Mobile application
- Authentication systems
- Token/NFT handling

### Out of Scope

- Third-party services and integrations
- Social engineering attacks
- Physical security
- Issues in dependencies (report upstream)
- Theoretical vulnerabilities without PoC

---

## Smart Contract Security

### Audit Status

| Contract | Auditor | Status | Report |
|----------|---------|--------|--------|
| Titan NFT | Pending | 🔄 In Progress | - |
| $BREACH Token | Pending | 🔄 In Progress | - |
| Marketplace | Pending | 📅 Scheduled | - |

### Known Considerations

- All smart contracts use the Pinocchio framework
- Upgradeable via program authority
- Admin functions protected by multisig

---

## Bug Bounty Program

We offer rewards for responsibly disclosed vulnerabilities.

### Reward Tiers

| Severity | Description | Reward |
|----------|-------------|--------|
| **Critical** | Loss of funds, contract takeover | $5,000 - $25,000 |
| **High** | Significant impact, data breach | $1,000 - $5,000 |
| **Medium** | Limited impact, requires conditions | $250 - $1,000 |
| **Low** | Minor issues, best practices | $50 - $250 |

### Eligibility

To be eligible for a reward:

- First reporter of the vulnerability
- Follow responsible disclosure guidelines
- Do not exploit the vulnerability
- Do not publicly disclose before fix
- Provide sufficient technical details

### Exclusions

- Issues already reported
- Issues in out-of-scope areas
- Denial of service attacks
- Spam or social engineering
- Issues requiring physical access

---

## Security Best Practices

### For Users

- Never share your wallet private key
- Verify transaction details before signing
- Use hardware wallets for large holdings
- Be cautious of phishing attempts
- Only use official BREACH links

### For Developers

- Review smart contract changes carefully
- Test on devnet before mainnet
- Use checked arithmetic operations
- Validate all user inputs
- Follow the principle of least privilege

---

## Incident Response

In case of a security incident:

1. **Identify**: Assess the scope and impact
2. **Contain**: Pause affected systems if necessary
3. **Investigate**: Determine root cause
4. **Remediate**: Deploy fixes
5. **Communicate**: Notify affected users
6. **Review**: Post-incident analysis

### Emergency Contacts

- GitHub Security Advisories: [Report Here](https://github.com/vnxfsc/BREACH/security/advisories/new)
- Maintainers will respond within 48 hours

---

## Hall of Fame

We recognize security researchers who help keep BREACH secure:

| Researcher | Contribution | Date |
|------------|--------------|------|
| *Your name here* | - | - |

---

## Updates

This security policy may be updated periodically. Check back for the latest version.

Last updated: January 2026
