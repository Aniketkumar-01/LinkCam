# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

The LinkCam team takes the security and privacy of our users seriously.

If you believe you have discovered a security vulnerability in LinkCam (e.g., in the binary wire framing, Windows clipboard bridge, path traversal, authentication handshake, or memory handling), please do **NOT** open a public issue.

### How to Report Privately:
1. Use GitHub's private vulnerability reporting feature:
   **[Submit a Security Advisory](https://github.com/Aniketkumar-01/LinkCam/security/advisories/new)**
2. Include the following details in your report:
   - Type of issue (command injection, memory overflow, path traversal, denial of service)
   - Step-by-step reproduction instructions or proof-of-concept payload
   - Impact assessment
   - Suggested fix (if known)

### Response Timeline:
- You will receive an initial response acknowledging your report within 48 hours.
- A fix will be developed in a private security fork, reviewed, and published with a CVE / patch release.
