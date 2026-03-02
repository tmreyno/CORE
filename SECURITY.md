# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes       |

Current release: **v0.1.14**

## Reporting a Vulnerability

Please report security issues privately via [GitHub Security Advisories](https://github.com/tmreyno/CORE/security/advisories/new) or by contacting the maintainer directly. Avoid filing public issues until the team confirms a fix timeline.

We aim to acknowledge reports within **48 hours** and provide a fix timeline within **7 days**.

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
- No telemetry or analytics data is collected or transmitted

### Dependencies\n\nNative C libraries (libarchive, libewf, LZMA SDK) are statically linked from audited, pinned versions. Rust and npm dependencies should be audited regularly:\n\n```bash\nnpm audit\ncargo audit\n```\n\n### Best Practices\n\n- Verify hashes before analysis\n- Store evidence on encrypted media\n- Restrict access to case workstations
