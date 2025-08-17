# Pigeon
Pigeon is a secure, lightweight messaging client that establishes direct encrypted connections between users. Designed with security as the top priority, Pigeon ensures your communications remain private through end-to-end encryption while maintaining minimal network overhead.

## Docs
- Coding standards: `CODING_STANDARDS.md`
- Tech decisions (locked v0): `pigeon_tech_decisions_locked_v_0.md`
- Milestone tasks: `M0_tasks.md`

## Dev quickstart

### One-liners (Windows PowerShell)
- Run all checks (fmt, clippy, tests):
  - `pwsh -NoProfile -ExecutionPolicy Bypass -File ./scripts/check.ps1`
- Format:
  - `pwsh -NoProfile -ExecutionPolicy Bypass -File ./scripts/fmt.ps1`
- Lint:
  - `pwsh -NoProfile -ExecutionPolicy Bypass -File ./scripts/lint.ps1`
- Test:
  - `pwsh -NoProfile -ExecutionPolicy Bypass -File ./scripts/test.ps1`

### Make targets (Linux/macOS)
- `make check` – fmt check, clippy (deny warnings), tests
- `make fmt` – format code
- `make lint` – clippy (deny warnings)
- `make test` – run tests

## CI
GitHub Actions runs fmt, clippy (deny warnings), and tests on pushes and PRs targeting `main`.