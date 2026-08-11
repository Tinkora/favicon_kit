# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.0 (current) | Yes |

## Reporting a Vulnerability

Do not include vulnerability details in a public issue. When GitHub private
vulnerability reporting is enabled for this repository, use
[Report a vulnerability](https://github.com/Tinkora/favicon_kit/security/advisories/new)
to submit the report.

That GitHub control is the project's intended private reporting channel. If the
control is unavailable, private reporting has not yet been enabled for the
repository. Do not publish a proof of concept; open a public issue requesting a
private reporting channel without disclosing the vulnerability. This project
does not publish a separate security email address or response-time commitment.

### Scope

The following areas are within scope:

- WASM sandbox escapes
- Malformed or resource-intensive image inputs that affect the application
- ZIP output that introduces unsafe or unexpected paths
- XSS vectors in the HTML UI
- Vulnerabilities in dependencies when they materially affect Favicon Kit

### Out of Scope

- Issues already documented as known limitations
- Theoretical attacks requiring physical access
- Behavior outside the application code, including a deployment host, browser,
  extension, or browser telemetry service
- Reports without a reproducible security impact

## Security Model

Favicon Kit uses the following security boundaries:

1. **Browser-local processing**: The editor processes browser-decoded image
   data in Rust/WASM and has no application endpoint for source images or
   generated files.

2. **Untrusted input handling**: Source dimensions are limited before image
   processing, and the ZIP export uses fixed archive member names.

3. **Content security policy**: The current static editor contains inline
   styles and an inline module script. A deployment with a strict CSP must use
   suitable hashes or nonces, or extract those assets before enforcing a policy
   that forbids inline content.

4. **Deployment boundary**: Hosting configuration, browser extensions, browser
   telemetry, and browser crash reporting are not controlled by this repository.
