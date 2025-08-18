## M3 – GUI Implementation

### Epic: App Shell & Architecture
- [ ] M3-400: Choose framework and scaffold (Tauri + React/TypeScript, or egui native)
  - Acceptance: Hello-world window, build scripts, and IPC bridge to Rust core; CI builds app
- [ ] M3-401: IPC layer and API surface to core (queue, contacts, inbox, send/listen)
  - Acceptance: Typed requests/responses; errors mapped; versioned IPC contract documented

### Epic: Onboarding & Identity
- [ ] M3-410: First-run wizard (data directory, passphrase, identity generate/import)
  - Acceptance: Wizard completes and persists choices; identity preview screen works
- [ ] M3-411: Unlock flow (Argon2id) with retry/lockout and optional env unlock
  - Acceptance: Locked state blocks sensitive actions; unlock transitions the app without restart

### Epic: Contacts & Inbox UI
- [ ] M3-420: Contacts CRUD UI (validate name, multiaddr, pubkey) with search/sort
  - Acceptance: Add/List/Show/Remove work and reflect backend; validation errors shown inline
- [ ] M3-421: Inbox list and details (virtualized list, preview pane)
  - Acceptance: Paginated or virtualized list; `show` view with metadata; export from UI
- [ ] M3-422: Inbox search (case-insensitive) with highlighting
  - Acceptance: Matches align with backend; keyboard navigation between results

### Epic: Messaging UI
- [ ] M3-430: Compose & send with priority toggle and contact resolution
  - Acceptance: Send path enqueues and updates status; errors surfaced non-blocking
- [ ] M3-431: Real-time receive display and desktop notifications
  - Acceptance: Running listener updates UI; notifications clickable to open message
- [ ] M3-432: Queue view with retry/backoff visibility and dead-letter browsing
  - Acceptance: Shows per-message attempts/next_due; dead-letter tab supports export

### Epic: Settings & Network
- [ ] M3-440: Network settings (listen address, mDNS, backoff/retries) with validation
  - Acceptance: Apply without restart when possible; otherwise prompts; persisted to config
- [ ] M3-441: Security settings (passphrase, rotation, unlock on startup)
  - Acceptance: Rotate at-rest key workflow; unlock prompts integrated with wizard

### Epic: Observability
- [ ] M3-450: Metrics dashboard (consume /metrics; render counters/graphs)
  - Acceptance: Sent/Delivered/Failed/Received update live; queue depth visible
- [ ] M3-451: Log viewer with level filters and tailing
  - Acceptance: Follows logs; level switches apply immediately; copy-to-clipboard works

### Epic: Packaging & Updates
- [ ] M3-460: Installers for Windows/macOS/Linux (Tauri bundler or platform tools)
  - Acceptance: MSI/PKG/DEB/RPM artifacts from CI; code-signing where applicable
- [ ] M3-461: Auto-update (if supported) with release channel selection
  - Acceptance: Checks for updates; verifies signatures; prompts before apply

### Epic: Accessibility & Internationalization
- [ ] M3-470: Accessibility pass (keyboard navigation, screen-reader labels)
  - Acceptance: Navigable without mouse; key controls documented; basic ARIA where applicable
- [ ] M3-471: i18n scaffolding and translations (en-US + one additional locale)
  - Acceptance: Language switch persists; core screens translated

### Epic: QA & CI for GUI
- [ ] M3-480: GUI smoke tests and automation (tauri-driver/playwright)
  - Acceptance: CI runs headless UI smoke; artifacts (screenshots/logs) on failure
- [ ] M3-481: End-to-end test harness (two peers via GUI) with flaky guard
  - Acceptance: Automated send/receive between two app instances passes consistently

### Exit Criteria (M3)
- [ ] Users can install a desktop app and complete onboarding
- [ ] Contacts, inbox, search, and compose/send are fully functional via GUI
- [ ] Live receive, notifications, and queue monitoring work reliably
- [ ] Core security options (passphrase, unlock, rotation) are accessible and tested
- [ ] CI produces installers and runs GUI smoke tests on all platforms