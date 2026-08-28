# Security Policy

## Supported versions

DiskHog is currently pre-release. Security fixes are applied to the latest code on `main`.

## Reporting a vulnerability

Please do not open a public issue for a vulnerability that could put users at risk. Use GitHub's **Security** tab and its private vulnerability reporting/security advisory flow for this repository when available.

Include a clear description, affected platform, reproduction steps, and the expected security impact. Avoid including secrets or unrelated personal data.

## Security model

DiskHog is intentionally read-only. It does not delete or modify scanned files, does not use telemetry, does not contact a backend, and does not follow symbolic links while scanning.
