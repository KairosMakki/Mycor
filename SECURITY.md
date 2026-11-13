# Security

## Reporting

Report vulnerabilities privately to the maintainers. Do not open public issues for security problems.

Please include:
- Affected version/commit
- Reproduction steps
- Impact assessment

## Supported versions

Security fixes target the latest release (currently 0.2.0) and `main`.

## Compiler safety

Mycor's safety guarantees (memory safety via ownership/borrowing) apply to safe code. `unsafe` blocks are the user's responsibility. The compiler itself is written in Rust and avoids unsafe code except where required.