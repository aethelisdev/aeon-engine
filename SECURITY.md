# Security Policy

## Supported Versions

Aeon Engine is under active development. Security patches and vulnerability fixes are provided exclusively for the `main` branch and the latest official release:

| Version | Supported |
| :--- | :--- |
| `main` | :white_check_mark: |
| Latest Release | :white_check_mark: |
| Older Releases | :x: |

---

## Reporting a Vulnerability

The Aeon Engine project takes software security, memory integrity, and stability seriously. If you discover a potential security vulnerability, **please do not disclose it publicly** (e.g. via public GitHub Issues, pull requests, discussions, or social media).

Please report security vulnerabilities through either of the following private channels:

1. **GitHub Private Vulnerability Reporting (Preferred):**
   - Navigate to the repository's **Security** tab on GitHub (`https://github.com/aethelisdev/aeon-engine/security`).
   - Click **Advisories** on the left menu, then click **"Report a vulnerability"**.
   - This creates a confidential, private advisory visible only to you and the repository maintainers.

2. **Direct Security Email:**
   - If GitHub Private Reporting is unavailable, send an email directly to **`aethelismedia@gmail.com`**.
   - Use the subject line: `[SECURITY: Aeon Engine] Vulnerability Report`.

---

## What to Include in Your Report

To help us investigate, reproduce, and resolve the issue quickly, please provide:
- A clear description of the vulnerability and its potential impact (e.g., Denial of Service, parser crash, unbounded memory consumption, directory traversal during asset loading).
- Minimal reproducible sample or Proof of Concept (PoC) file/script.
- The operating system, GPU model/driver, and Rust toolchain version used.
- Any suggested remediation or patch, if available.

---

## Response & Disclosure Process

- **Acknowledgment:** We strive to review and acknowledge all received reports within **48 hours**.
- **Investigation:** Maintainers will verify the vulnerability and collaborate with you privately to develop a fix.
- **Coordinated Release:** A patch will be published with the next release or hotfix. You will be credited in the release notes and advisory (unless you prefer to remain anonymous).

---

## Memory Safety & Scope

Aeon Engine enforces a strict **100% Safe Rust Mandate** (`unsafe` is strictly forbidden across the codebase). Consequently, classic memory corruption vulnerabilities (such as buffer overflows, use-after-free, double-free, and dangling pointers) are mathematically prevented by the Rust compiler.

Security assessments and vulnerability reports should focus on:
- Malicious file/asset deserialization (glTF, scenes, prefabs, shaders) causing resource exhaustion or unexpected panics.
- Sandbox/path traversal vulnerabilities when resolving file paths or asset URLs.
- Logic denial-of-service (infinite loops, unbounded thread spawning, CPU starvation).

