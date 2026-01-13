# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes       |

## Reporting a Vulnerability

Please report security issues privately. Avoid filing public issues until the team confirms a fix timeline.

Include:

- Summary and impact
- Reproduction steps
- Environment details
- Suggested mitigation (if available)

## Security Considerations

### Forensic Integrity

- Evidence files are treated as read-only
- Hash verification is supported for integrity checks
- Path traversal is sanitized in backend utilities
- Audit logging captures evidence access and export operations

### Data Handling

- Evidence data is processed locally
- Report generation is local by default
- Optional AI-assisted report features may send data to configured providers when enabled

### Best Practices

- Verify hashes before analysis
- Store evidence on encrypted media
- Restrict access to case workstations

## Dependency Audits

If your environment supports them:

```bash
npm audit
cargo audit
```
