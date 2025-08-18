## M3 – GUI Implementation

### Epic: App Shell & Architecture
- [x] M3-300: Choose framework and scaffold (egui/eframe native GUI)
  - Acceptance: Hello-world window using eframe/egui; build scripts; integrates Rust core crate directly; CI builds app
- [x] M3-301: IPC layer and API surface to core (queue, contacts, inbox, send/listen)
  - Acceptance: Typed requests/responses; errors mapped; versioned IPC contract documented

### Epic: Onboarding & Identity
- [x] M3-310: First-run wizard (data directory, passphrase, identity generate/import)
  - Acceptance: Wizard completes and persists choices; identity preview screen works
- [x] M3-311: Unlock flow (Argon2id) with retry/lockout and optional env unlock
  - Acceptance: Locked state blocks sensitive actions; unlock transitions the app without restart

### Epic: Contacts & Inbox UI
- [x] M3-320: Contacts CRUD UI (validate name, multiaddr, pubkey) with search/sort
  - Acceptance: Add/List/Show/Remove work and reflect backend; validation errors shown inline
- [x] M3-321: Inbox list and details (virtualized list, preview pane)
  - Acceptance: Paginated or virtualized list; `show` view with metadata; export from UI
- [x] M3-322: Inbox search (case-insensitive) with highlighting
  - Acceptance: Matches align with backend; keyboard navigation between results

### Epic: Messaging UI
- [x] M3-330: Compose & send with priority toggle and contact resolution
  - Acceptance: Send path enqueues and updates status; errors surfaced non-blocking
- [x] M3-331: Real-time receive display and desktop notifications
  - Acceptance: Running listener updates UI; notifications clickable to open message
- [x] M3-332: Queue view with retry/backoff visibility and dead-letter browsing
  - Acceptance: Shows per-message attempts/next_due; dead-letter tab supports export

### Epic: Settings & Network
- [x] M3-340: Network settings (listen address, mDNS, backoff/retries) with validation
  - Acceptance: Apply without restart when possible; otherwise prompts; persisted to config
- [x] M3-341: Security settings (passphrase, rotation, unlock on startup)
  - Acceptance: Rotate at-rest key workflow; unlock prompts integrated with wizard

### Epic: Observability
- [x] M3-350: Metrics dashboard (consume /metrics; render counters/graphs)
  - Acceptance: Sent/Delivered/Failed/Received update live; queue depth visible
- [x] M3-351: Log viewer with level filters and tailing
  - Acceptance: Follows logs; level switches apply immediately; copy-to-clipboard works

### Epic: Packaging & Updates
- [x] M3-360: Installers for Windows/macOS/Linux (platform tools)
  - Acceptance: MSI/PKG/DEB/RPM artifacts from CI; code-signing where applicable
- [x] M3-361: Auto-update (if supported) with release channel selection
  - Acceptance: Checks for updates; verifies signatures; prompts before apply

### Epic: Accessibility
- [ ] M3-370: Accessibility pass (keyboard navigation, screen-reader labels)
  - Acceptance: Navigable without mouse; key controls documented; basic ARIA where applicable

### Epic: QA & CI for GUI
- [ ] M3-380: GUI smoke tests and automation (tauri-driver/playwright)
  - Acceptance: CI runs headless UI smoke; artifacts (screenshots/logs) on failure
- [ ] M3-381: End-to-end test harness (two peers via GUI) with flaky guard
  - Acceptance: Automated send/receive between two app instances passes consistently

### Exit Criteria (M3)
- [ ] Users can install a desktop app and complete onboarding
- [ ] Contacts, inbox, search, and compose/send are fully functional via GUI
- [ ] Live receive, notifications, and queue monitoring work reliably
- [ ] Core security options (passphrase, unlock, rotation) are accessible and tested
- [ ] CI produces installers and runs GUI smoke tests on all platforms