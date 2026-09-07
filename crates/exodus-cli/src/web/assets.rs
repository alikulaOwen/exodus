//! Embedded Web Assets for the Exodus Universal HITL Gate Web UI.
//!
//! Embedded directly into the executable using string constants for single-binary zero-dependency distribution.

pub const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Project Exodus — Enterprise Agentic Execution Fabric</title>
  <link rel="stylesheet" href="/static/style.css">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;700&family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap" rel="stylesheet">
</head>
<body>
  <div class="app-shell">
    <!-- Navigation Header -->
    <header class="top-nav">
      <div class="brand">
        <div class="brand-icon">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
          </svg>
        </div>
        <div class="brand-text">
          <span class="brand-title">EXODUS</span>
          <span class="brand-subtitle">Enterprise Agentic Execution Fabric</span>
        </div>
      </div>

      <div class="header-status">
        <div class="status-pill status-online">
          <span class="pulse-dot"></span>
          <span>SurrealDB Living Memory Active</span>
        </div>
        <div class="status-pill status-neutral">
          <span>Zero-External Runtime (Embedded Axum)</span>
        </div>
      </div>

      <div class="header-actions">
        <button id="btn-sdlc-settings" class="btn btn-secondary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
            <circle cx="12" cy="12" r="3"/>
          </svg>
          SDLC Pipeline
        </button>
        <button id="btn-new-item" class="btn btn-primary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          Ingest Event
        </button>
      </div>
    </header>

    <!-- KPI Summary Bar -->
    <section class="kpi-grid">
      <div class="kpi-card" data-domain="all">
        <div class="kpi-header">
          <span class="kpi-title">TOTAL PIPELINE ITEMS</span>
          <span class="badge badge-indigo">All Domains</span>
        </div>
        <div class="kpi-value" id="kpi-total">0</div>
        <div class="kpi-sub">Across 6 closed-loop stages</div>
      </div>

      <div class="kpi-card" data-domain="#prod-bug">
        <div class="kpi-header">
          <span class="kpi-title">ENGINEERING (#prod-bug)</span>
          <span class="badge badge-emerald">Worktrees</span>
        </div>
        <div class="kpi-value" id="kpi-prod-bug">0</div>
        <div class="kpi-sub">Compiler & Test Verification</div>
      </div>

      <div class="kpi-card" data-domain="#crm-request">
        <div class="kpi-header">
          <span class="kpi-title">COMMERCIAL (#crm-request)</span>
          <span class="badge badge-amber">Policy Rules</span>
        </div>
        <div class="kpi-value" id="kpi-crm-request">0</div>
        <div class="kpi-sub">ARR & Discount Governance</div>
      </div>

      <div class="kpi-card" data-domain="#survey-mapping">
        <div class="kpi-header">
          <span class="kpi-title">PRODUCT (#survey-mapping)</span>
          <span class="badge badge-cyan">Taxonomy</span>
        </div>
        <div class="kpi-value" id="kpi-survey-mapping">0</div>
        <div class="kpi-sub">Feedback & FK Completeness</div>
      </div>
    </section>

    <!-- Main Workspace Area -->
    <main class="workspace-layout">
      <!-- Left Column: Filterable Event Feed -->
      <aside class="feed-pane">
        <div class="pane-header">
          <h2>Operational Queue</h2>
          <span class="feed-count" id="feed-count-badge">0 items</span>
        </div>

        <!-- Filter Tags -->
        <div class="domain-filters">
          <button class="filter-tab active" data-filter="all">All</button>
          <button class="filter-tab" data-filter="#prod-bug">#prod-bug</button>
          <button class="filter-tab" data-filter="#crm-request">#crm-request</button>
          <button class="filter-tab" data-filter="#survey-mapping">#survey-mapping</button>
        </div>

        <div class="items-list" id="items-list">
          <div class="empty-state">Loading operational queue...</div>
        </div>
      </aside>

      <!-- Right Column: Universal Governed HITL Review Gate -->
      <section class="review-pane" id="review-pane">
        <div class="empty-review" id="empty-review">
          <div class="empty-illustration">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="10"></circle>
              <polyline points="12 6 12 12 14 14"></polyline>
            </svg>
          </div>
          <h3>Select an Operational Item to Review</h3>
          <p>Choose an item from the left queue to inspect verification telemetry, diffs, policy checks, or perform human authorization.</p>
        </div>

        <div class="active-review hidden" id="active-review">
          <!-- Item Header & State Stepper -->
          <div class="review-header">
            <div class="review-meta">
              <div class="review-tags">
                <span class="badge" id="review-domain-badge">#domain</span>
                <span class="badge badge-state" id="review-state-badge">State</span>
                <span class="review-id" id="review-id">op-xxxx</span>
              </div>
              <h1 class="review-title" id="review-title">Item Title</h1>
              <p class="review-desc" id="review-desc">Description</p>
            </div>

            <!-- HITL Primary Action Buttons -->
            <div class="review-actions">
              <button id="btn-action-verify" class="btn btn-secondary">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                  <polyline points="22 4 12 14.01 9 11.01"></polyline>
                </svg>
                Verify Contract
              </button>
              <button id="btn-action-reject" class="btn btn-danger">
                Reject
              </button>
              <button id="btn-action-approve" class="btn btn-primary-highlight">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
                Approve & Promote (HITL)
              </button>
            </div>
          </div>

          <!-- 6-Stage Universal Closed-Loop Visual Stepper -->
          <div class="stepper-track">
            <div class="step-node" id="step-captured">
              <div class="step-circle">1</div>
              <span class="step-label">Captured</span>
            </div>
            <div class="step-line" id="line-1"></div>
            <div class="step-node" id="step-sandboxed">
              <div class="step-circle">2</div>
              <span class="step-label">Sandboxed</span>
            </div>
            <div class="step-line" id="line-2"></div>
            <div class="step-node" id="step-verified">
              <div class="step-circle">3</div>
              <span class="step-label">Verified</span>
            </div>
            <div class="step-line" id="line-3"></div>
            <div class="step-node" id="step-approved">
              <div class="step-circle">4</div>
              <span class="step-label">Human Approved</span>
            </div>
            <div class="step-line" id="line-4"></div>
            <div class="step-node" id="step-promoted">
              <div class="step-circle">5</div>
              <span class="step-label">Promoted (Memory)</span>
            </div>
          </div>

          <!-- Domain Dynamic Inspection Content -->
          <div class="inspection-container" id="inspection-container">
            <!-- Populated dynamically based on domain tag -->
          </div>

          <!-- Verification Telemetry & Rule Matrix -->
          <div class="verification-box" id="verification-box">
            <h3>Deterministic Contract Verification Results</h3>
            <div id="verification-summary" class="verification-summary">Not yet verified.</div>
            <div id="rule-results-list" class="rule-results-list"></div>
          </div>

          <!-- Audit Trail -->
          <div class="audit-box">
            <h3>Audit Trail & Lineage</h3>
            <div class="audit-entries" id="audit-entries"></div>
          </div>
        </div>
      </section>
    </main>

    <!-- SDLC Settings Modal -->
    <div class="modal-backdrop hidden" id="modal-sdlc">
      <div class="modal-dialog">
        <div class="modal-header">
          <h3>SDLC Pipeline & Remote Repository Integration</h3>
          <button class="modal-close" id="btn-close-sdlc">&times;</button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>Remote Git Repository URL</label>
            <input type="text" id="sdlc-repo-url" class="form-input" value="https://github.com/exodus-migration/exodus.git">
          </div>
          <div class="form-row">
            <div class="form-group">
              <label>Provider</label>
              <select id="sdlc-provider" class="form-select">
                <option value="github">GitHub</option>
                <option value="gitlab">GitLab</option>
                <option value="bitbucket">Bitbucket</option>
                <option value="generic-git">Generic Git</option>
              </select>
            </div>
            <div class="form-group">
              <label>Default Branch</label>
              <input type="text" id="sdlc-default-branch" class="form-input" value="master">
            </div>
          </div>
          <div class="form-row">
            <div class="form-group">
              <label>CI Automation Engine</label>
              <select id="sdlc-ci-type" class="form-select">
                <option value="github-actions">GitHub Actions (.github/workflows/exodus-verify.yml)</option>
                <option value="gitlab-ci">GitLab CI (.gitlab-ci.yml)</option>
                <option value="local-pre-commit">Local Git Pre-Push Hook (.git/hooks/pre-push)</option>
              </select>
            </div>
            <div class="form-group">
              <label>Auto-trigger on CI Failure</label>
              <select id="sdlc-auto-trigger" class="form-select">
                <option value="true">Enabled (Capture #prod-bug)</option>
                <option value="false">Disabled</option>
              </select>
            </div>
          </div>
          <div class="scaffold-preview">
            <label>Generated Workflow Configuration Preview</label>
            <pre class="code-preview" id="sdlc-scaffold-preview">Loading scaffold preview...</pre>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" id="btn-cancel-sdlc">Cancel</button>
          <button class="btn btn-primary" id="btn-save-sdlc">Save Settings & Update Scaffold</button>
        </div>
      </div>
    </div>

    <!-- New Event Ingestion Modal -->
    <div class="modal-backdrop hidden" id="modal-ingest">
      <div class="modal-dialog">
        <div class="modal-header">
          <h3>Ingest Operational Event</h3>
          <button class="modal-close" id="btn-close-ingest">&times;</button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>Operational Domain Tag</label>
            <select id="new-item-domain" class="form-select">
              <option value="#prod-bug">#prod-bug (Software Engineering Bug / CI Failure)</option>
              <option value="#crm-request">#crm-request (Commercial Discount / Tier Mutation)</option>
              <option value="#survey-mapping">#survey-mapping (Product CSAT / Taxonomy Alignment)</option>
            </select>
          </div>
          <div class="form-group">
            <label>Title</label>
            <input type="text" id="new-item-title" class="form-input" placeholder="e.g. Broken authentication timeout or Enterprise Tier Request">
          </div>
          <div class="form-group">
            <label>Description</label>
            <textarea id="new-item-desc" class="form-textarea" rows="3" placeholder="Context, stack trace snippet, or customer memo"></textarea>
          </div>
          <div class="form-group">
            <label>Requester Actor / Role</label>
            <input type="text" id="new-item-requester" class="form-input" value="human_architect">
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" id="btn-cancel-ingest">Cancel</button>
          <button class="btn btn-primary" id="btn-submit-ingest">Ingest into Fabric</button>
        </div>
      </div>
    </div>

  </div>
  <script src="/static/app.js"></script>
</body>
</html>
"#;

pub const STYLE_CSS: &str = r#"
:root {
  --bg-primary: #0b0f19;
  --bg-surface: #111827;
  --bg-card: #162032;
  --bg-card-hover: #1c2a42;
  --border: #243247;
  --border-focus: #3b82f6;
  
  --text-primary: #f3f4f6;
  --text-secondary: #9ca3af;
  --text-muted: #6b7280;

  --accent-cyan: #06b6d4;
  --accent-blue: #3b82f6;
  --accent-indigo: #6366f1;
  --accent-emerald: #10b981;
  --accent-amber: #f59e0b;
  --accent-rose: #f43f5e;

  --font-sans: 'Plus Jakarta Sans', system-ui, -apple-system, sans-serif;
  --font-mono: 'JetBrains Mono', monospace;
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-family: var(--font-sans);
  line-height: 1.5;
  font-size: 14px;
  overflow-x: hidden;
}

.app-shell {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

/* Header */
.top-nav {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  background-color: var(--bg-surface);
  border-bottom: 1px solid var(--border);
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
}

.brand-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  background: linear-gradient(135deg, var(--accent-indigo), var(--accent-cyan));
  color: white;
  border-radius: 10px;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.35);
}

.brand-title {
  display: block;
  font-weight: 800;
  font-size: 16px;
  letter-spacing: 1.5px;
  color: #fff;
}

.brand-subtitle {
  display: block;
  font-size: 11px;
  color: var(--text-secondary);
}

.header-status {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-pill {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 12px;
  border-radius: 9999px;
  font-size: 12px;
  font-weight: 500;
}

.status-online {
  background-color: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: var(--accent-emerald);
}

.pulse-dot {
  width: 7px;
  height: 7px;
  background-color: var(--accent-emerald);
  border-radius: 50%;
  box-shadow: 0 0 8px var(--accent-emerald);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { transform: scale(0.95); opacity: 0.8; }
  50% { transform: scale(1.3); opacity: 1; }
  100% { transform: scale(0.95); opacity: 0.8; }
}

.status-neutral {
  background-color: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border);
  color: var(--text-secondary);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

/* Buttons */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  border: 1px solid transparent;
}

.btn-primary {
  background: linear-gradient(135deg, var(--accent-blue), var(--accent-indigo));
  color: #fff;
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
}

.btn-primary:hover {
  filter: brightness(1.1);
  box-shadow: 0 4px 14px rgba(59, 130, 246, 0.5);
}

.btn-primary-highlight {
  background: linear-gradient(135deg, var(--accent-emerald), #059669);
  color: #fff;
  box-shadow: 0 2px 8px rgba(16, 185, 129, 0.3);
}

.btn-primary-highlight:hover {
  filter: brightness(1.1);
}

.btn-secondary {
  background-color: rgba(255, 255, 255, 0.06);
  border-color: var(--border);
  color: var(--text-primary);
}

.btn-secondary:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.btn-danger {
  background-color: rgba(244, 63, 94, 0.15);
  border-color: rgba(244, 63, 94, 0.3);
  color: var(--accent-rose);
}

.btn-danger:hover {
  background-color: rgba(244, 63, 94, 0.25);
}

/* KPI Summary */
.kpi-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  padding: 20px 24px;
  background-color: rgba(17, 24, 39, 0.4);
  border-bottom: 1px solid var(--border);
}

.kpi-card {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 16px;
  transition: transform 0.15s ease, border-color 0.15s ease;
}

.kpi-card:hover {
  border-color: var(--accent-blue);
  transform: translateY(-2px);
}

.kpi-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.kpi-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: var(--text-muted);
}

.kpi-value {
  font-size: 28px;
  font-weight: 800;
  color: #fff;
  line-height: 1.1;
  margin-bottom: 4px;
}

.kpi-sub {
  font-size: 12px;
  color: var(--text-secondary);
}

/* Workspace Layout */
.workspace-layout {
  display: grid;
  grid-template-columns: 360px 1fr;
  flex: 1;
  overflow: hidden;
}

/* Feed Pane */
.feed-pane {
  background-color: var(--bg-surface);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  height: calc(100vh - 180px);
}

.pane-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid var(--border);
}

.pane-header h2 {
  font-size: 15px;
  font-weight: 700;
}

.feed-count {
  font-size: 12px;
  color: var(--text-secondary);
  background: rgba(255, 255, 255, 0.08);
  padding: 2px 8px;
  border-radius: 999px;
}

.domain-filters {
  display: flex;
  gap: 6px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border);
  overflow-x: auto;
}

.filter-tab {
  background: none;
  border: 1px solid transparent;
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 6px;
  cursor: pointer;
  white-space: nowrap;
}

.filter-tab.active {
  background-color: rgba(59, 130, 246, 0.15);
  border-color: rgba(59, 130, 246, 0.3);
  color: #60a5fa;
}

.items-list {
  flex: 1;
  overflow-y: auto;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.item-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.item-card:hover {
  background-color: var(--bg-card-hover);
  border-color: rgba(99, 102, 241, 0.4);
}

.item-card.selected {
  border-color: var(--accent-blue);
  box-shadow: 0 0 0 1px var(--accent-blue), 0 4px 12px rgba(59, 130, 246, 0.2);
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.card-title {
  font-size: 13px;
  font-weight: 700;
  color: #fff;
  margin-bottom: 4px;
}

.card-meta {
  font-size: 11px;
  color: var(--text-muted);
  display: flex;
  justify-content: space-between;
}

/* Badges */
.badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 7px;
  border-radius: 6px;
}

.badge-emerald { background: rgba(16, 185, 129, 0.15); color: #34d399; }
.badge-amber { background: rgba(245, 158, 11, 0.15); color: #fbbf24; }
.badge-cyan { background: rgba(6, 182, 212, 0.15); color: #38bdf8; }
.badge-indigo { background: rgba(99, 102, 241, 0.15); color: #818cf8; }
.badge-rose { background: rgba(244, 63, 94, 0.15); color: #fb7185; }

/* Review Pane */
.review-pane {
  padding: 24px;
  overflow-y: auto;
  height: calc(100vh - 180px);
}

.empty-review {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-secondary);
  text-align: center;
  padding: 40px;
}

.empty-illustration {
  margin-bottom: 16px;
  color: var(--border);
}

.empty-review h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.review-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border);
}

.review-tags {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.review-id {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
}

.review-title {
  font-size: 22px;
  font-weight: 800;
  color: #fff;
  margin-bottom: 6px;
}

.review-desc {
  font-size: 13px;
  color: var(--text-secondary);
  max-width: 650px;
}

.review-actions {
  display: flex;
  gap: 10px;
}

/* 6-Stage Stepper */
.stepper-track {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 16px 24px;
  margin-bottom: 24px;
}

.step-node {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.step-circle {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background-color: var(--bg-card);
  border: 2px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 12px;
  color: var(--text-secondary);
}

.step-node.active .step-circle {
  background-color: rgba(59, 130, 246, 0.2);
  border-color: var(--accent-blue);
  color: #fff;
  box-shadow: 0 0 12px rgba(59, 130, 246, 0.4);
}

.step-node.completed .step-circle {
  background-color: var(--accent-emerald);
  border-color: var(--accent-emerald);
  color: #fff;
}

.step-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
}

.step-node.active .step-label,
.step-node.completed .step-label {
  color: var(--text-primary);
}

.step-line {
  flex: 1;
  height: 2px;
  background-color: var(--border);
  margin: 0 8px;
  margin-bottom: 20px;
}

.step-line.active {
  background-color: var(--accent-emerald);
}

/* Inspection Boxes */
.inspection-container {
  margin-bottom: 24px;
}

.panel-card {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
}

.panel-card h3 {
  font-size: 14px;
  font-weight: 700;
  margin-bottom: 14px;
  color: #fff;
  display: flex;
  align-items: center;
  gap: 8px;
}

.diff-box {
  background-color: #06090e;
  border: 1px solid var(--border);
  border-radius: 8px;
  font-family: var(--font-mono);
  font-size: 12px;
  padding: 14px;
  overflow-x: auto;
  line-height: 1.6;
}

.diff-add { color: #34d399; background: rgba(16, 185, 129, 0.08); display: block; }
.diff-rem { color: #f43f5e; background: rgba(244, 63, 94, 0.08); display: block; }
.diff-ctx { color: #9ca3af; display: block; }

.policy-meter-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 16px;
}

.meter-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 12px;
}

.meter-title { font-size: 11px; color: var(--text-muted); font-weight: 600; margin-bottom: 4px; }
.meter-value { font-size: 20px; font-weight: 700; color: #fff; }

.taxonomy-map-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mapping-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 10px 14px;
}

.mapping-feedback { font-size: 13px; color: #fff; font-style: italic; }

.verification-box, .audit-box {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
}

.verification-box h3, .audit-box h3 {
  font-size: 14px;
  font-weight: 700;
  margin-bottom: 12px;
  color: #fff;
}

.rule-result-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.03);
  margin-bottom: 6px;
  font-size: 12px;
}

.rule-result-item.pass { border-left: 3px solid var(--accent-emerald); }
.rule-result-item.fail { border-left: 3px solid var(--accent-rose); }

/* Modals */
.modal-backdrop {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
  backdrop-filter: blur(4px);
}

.modal-dialog {
  background-color: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 14px;
  width: 580px;
  max-width: 90vw;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.modal-header h3 { font-size: 16px; font-weight: 700; }
.modal-close { background: none; border: none; font-size: 20px; color: var(--text-muted); cursor: pointer; }

.modal-body {
  padding: 20px;
}

.form-group { margin-bottom: 14px; }
.form-group label { display: block; font-size: 12px; font-weight: 600; color: var(--text-secondary); margin-bottom: 6px; }
.form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }

.form-input, .form-select, .form-textarea {
  width: 100%;
  background-color: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 12px;
  color: #fff;
  font-size: 13px;
  font-family: inherit;
}

.form-input:focus, .form-select:focus, .form-textarea:focus {
  outline: none;
  border-color: var(--border-focus);
}

.code-preview {
  background-color: #06090e;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 10px;
  max-height: 140px;
  overflow-y: auto;
  color: var(--accent-cyan);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 14px 20px;
  border-top: 1px solid var(--border);
}

.hidden { display: none !important; }
"#;

pub const APP_JS: &str = r#"
// Project Exodus Universal HITL Gate Client Logic

let operationalItems = [];
let selectedItemId = null;
let currentFilter = 'all';

async function fetchQueue() {
  try {
    const res = await fetch('/api/operations');
    if (res.ok) {
      operationalItems = await res.json();
      updateKpis();
      renderQueue();
      if (selectedItemId) {
        const item = operationalItems.find(i => i.id === selectedItemId);
        if (item) renderReview(item);
      }
    }
  } catch (err) {
    console.error('Failed to fetch operations queue:', err);
  }
}

function updateKpis() {
  const total = operationalItems.length;
  const prodBug = operationalItems.filter(i => i.domain_tag === '#prod-bug').length;
  const crm = operationalItems.filter(i => i.domain_tag === '#crm-request').length;
  const survey = operationalItems.filter(i => i.domain_tag === '#survey-mapping').length;

  document.getElementById('kpi-total').textContent = total;
  document.getElementById('kpi-prod-bug').textContent = prodBug;
  document.getElementById('kpi-crm-request').textContent = crm;
  document.getElementById('kpi-survey-mapping').textContent = survey;
}

function renderQueue() {
  const container = document.getElementById('items-list');
  const filtered = currentFilter === 'all' 
    ? operationalItems 
    : operationalItems.filter(i => i.domain_tag === currentFilter);

  document.getElementById('feed-count-badge').textContent = `${filtered.length} items`;

  if (filtered.length === 0) {
    container.innerHTML = '<div class="empty-state" style="padding: 24px; text-align: center; color: var(--text-muted);">No operational items in this queue.</div>';
    return;
  }

  container.innerHTML = filtered.map(item => {
    const isSelected = item.id === selectedItemId ? 'selected' : '';
    const badgeColor = item.domain_tag === '#prod-bug' ? 'badge-emerald' : item.domain_tag === '#crm-request' ? 'badge-amber' : 'badge-cyan';
    
    return `
      <div class="item-card ${isSelected}" onclick="selectItem('${item.id}')">
        <div class="card-top">
          <span class="badge ${badgeColor}">${item.domain_tag}</span>
          <span class="badge badge-indigo">${item.state}</span>
        </div>
        <div class="card-title">${escapeHtml(item.title)}</div>
        <div class="card-meta">
          <span>${item.requester}</span>
          <span>${formatTime(item.created_at)}</span>
        </div>
      </div>
    `;
  }).join('');
}

function selectItem(id) {
  selectedItemId = id;
  renderQueue();
  const item = operationalItems.find(i => i.id === id);
  if (item) {
    renderReview(item);
  }
}

function renderReview(item) {
  document.getElementById('empty-review').classList.add('hidden');
  document.getElementById('active-review').classList.remove('hidden');

  document.getElementById('review-id').textContent = item.id;
  document.getElementById('review-title').textContent = item.title;
  document.getElementById('review-desc').textContent = item.description;
  
  const domainBadge = document.getElementById('review-domain-badge');
  domainBadge.textContent = item.domain_tag;
  domainBadge.className = `badge ${item.domain_tag === '#prod-bug' ? 'badge-emerald' : item.domain_tag === '#crm-request' ? 'badge-amber' : 'badge-cyan'}`;

  document.getElementById('review-state-badge').textContent = item.state;

  updateStepper(item.state);
  renderInspection(item);
  renderVerification(item);
  renderAudit(item);
}

function updateStepper(state) {
  const stages = ['captured', 'sandboxed', 'verified', 'approved', 'promoted'];
  const stateMap = {
    'captured': 0,
    'sandboxed': 1,
    'contract_verified': 2,
    'degraded': 2,
    'human_approved': 3,
    'promoted': 4
  };

  const currentIdx = stateMap[state.toLowerCase()] ?? 0;

  stages.forEach((stage, idx) => {
    const node = document.getElementById(`step-${stage}`);
    if (!node) return;
    node.className = 'step-node';
    if (idx < currentIdx) node.classList.add('completed');
    if (idx === currentIdx) node.classList.add('active');

    if (idx > 0) {
      const line = document.getElementById(`line-${idx}`);
      if (line) {
        line.className = idx <= currentIdx ? 'step-line active' : 'step-line';
      }
    }
  });
}

function renderInspection(item) {
  const container = document.getElementById('inspection-container');
  const payload = item.payload;

  if (payload.domain_type === 'prod_bug') {
    const data = payload.data;
    container.innerHTML = `
      <div class="panel-card">
        <h3>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline></svg>
          Engineering Sandbox Diff & Compiler Telemetry
        </h3>
        <div style="margin-bottom: 12px; font-size: 12px; color: var(--text-secondary);">
          <strong>Commit:</strong> <code style="color: var(--accent-cyan);">${escapeHtml(data.commit_id)}</code> &bull;
          <strong>Reproduction:</strong> <code>${escapeHtml(data.reproduction_command || 'cargo test')}</code>
        </div>
        <div class="diff-box">
          <span class="diff-ctx">@@ -40,7 +40,7 @@ fn authenticate_session(token: &str) -> Result&lt;Session&gt; {</span>
          <span class="diff-rem">-    let claims = parse_jwt_unchecked(token)?; // Panic on malformed token</span>
          <span class="diff-add">+    let claims = parse_jwt_safe(token).map_err(|e| AuthError::InvalidToken(e))?;</span>
          <span class="diff-ctx">     validate_expiration(&claims)?;</span>
          <span class="diff-ctx">     Ok(Session::from_claims(claims))</span>
        </div>
      </div>
    `;
  } else if (payload.domain_type === 'crm_request') {
    const data = payload.data;
    container.innerHTML = `
      <div class="panel-card">
        <h3>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="5" width="20" height="14" rx="2"></rect><line x1="2" y1="10" x2="22" y2="10"></line></svg>
          Commercial Policy Governance & Impact Matrix
        </h3>
        <div class="policy-meter-grid">
          <div class="meter-card">
            <div class="meter-title">ACCOUNT ARR</div>
            <div class="meter-value">$${Number(data.current_arr || 0).toLocaleString()}</div>
          </div>
          <div class="meter-card">
            <div class="meter-title">REQUESTED DISCOUNT</div>
            <div class="meter-value" style="color: ${data.requested_discount_pct > 25 ? 'var(--accent-amber)' : 'var(--accent-emerald)'};">${data.requested_discount_pct}%</div>
          </div>
          <div class="meter-card">
            <div class="meter-title">REQUESTED TIER</div>
            <div class="meter-value">${escapeHtml(data.requested_tier)}</div>
          </div>
        </div>
        <div style="font-size: 13px; color: var(--text-secondary);">
          <strong>Requester:</strong> ${escapeHtml(data.requester_role)} &bull; <strong>Term:</strong> ${data.contract_term_months || 12} Months
        </div>
      </div>
    `;
  } else if (payload.domain_type === 'survey_mapping') {
    const data = payload.data;
    const responses = data.responses || [];
    container.innerHTML = `
      <div class="panel-card">
        <h3>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
          Product CSAT Feedback & Taxonomy Ontology Mapping (${responses.length} Items)
        </h3>
        <div class="taxonomy-map-list">
          ${responses.map(r => `
            <div class="mapping-row">
              <span class="mapping-feedback">"${escapeHtml(r.feedback_text)}"</span>
              <span class="badge ${r.candidate_taxonomy_node ? 'badge-cyan' : 'badge-rose'}">
                ${r.candidate_taxonomy_node || 'UNMAPPED'}
              </span>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }
}

function renderVerification(item) {
  const summaryEl = document.getElementById('verification-summary');
  const rulesListEl = document.getElementById('rule-results-list');

  if (!item.verification_report) {
    summaryEl.innerHTML = '<span style="color: var(--text-muted);">Contract not yet verified. Click "Verify Contract" to run deterministic gates.</span>';
    rulesListEl.innerHTML = '';
    return;
  }

  const rep = item.verification_report;
  summaryEl.innerHTML = `
    <div style="font-weight: 700; color: ${rep.passed ? 'var(--accent-emerald)' : 'var(--accent-rose)'}; margin-bottom: 8px;">
      ${rep.passed ? '✓ PASSED CONTRACT VERIFICATION' : '✗ VERIFICATION FAILED'}
    </div>
    <div style="font-size: 12px; color: var(--text-secondary);">${escapeHtml(rep.summary)}</div>
  `;

  rulesListEl.innerHTML = (rep.rule_results || []).map(r => `
    <div class="rule-result-item ${r.passed ? 'pass' : 'fail'}">
      <div><strong>${escapeHtml(r.rule_name)}:</strong> ${escapeHtml(r.details)}</div>
      <span class="badge ${r.passed ? 'badge-emerald' : 'badge-rose'}">${r.passed ? 'PASS' : 'FAIL'}</span>
    </div>
  `).join('');
}

function renderAudit(item) {
  const container = document.getElementById('audit-entries');
  container.innerHTML = (item.audit_trail || []).map(a => `
    <div style="font-size: 11px; margin-bottom: 6px; color: var(--text-secondary); display: flex; justify-content: space-between;">
      <span><strong>[${escapeHtml(a.action)}]</strong> ${escapeHtml(a.details)} (${escapeHtml(a.actor)})</span>
      <span style="font-family: var(--font-mono); color: var(--text-muted);">${formatTime(a.timestamp)}</span>
    </div>
  `).join('');
}

// User Actions
document.getElementById('btn-action-verify').onclick = async () => {
  if (!selectedItemId) return;
  try {
    const res = await fetch(`/api/operations/${selectedItemId}/verify`, { method: 'POST' });
    if (res.ok) {
      await fetchQueue();
    } else {
      alert('Verification failed: ' + await res.text());
    }
  } catch (err) {
    alert('Error running verification: ' + err);
  }
};

document.getElementById('btn-action-approve').onclick = async () => {
  if (!selectedItemId) return;
  try {
    const res = await fetch(`/api/operations/${selectedItemId}/approve`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ approver: 'human_architect', notes: 'Verified and signed off via Exodus HITL Web UI' })
    });
    if (res.ok) {
      await fetchQueue();
    } else {
      alert('Approval failed: ' + await res.text());
    }
  } catch (err) {
    alert('Error approving item: ' + err);
  }
};

document.getElementById('btn-action-reject').onclick = async () => {
  if (!selectedItemId) return;
  const reason = prompt('Please enter rejection reason:');
  if (!reason) return;
  try {
    const res = await fetch(`/api/operations/${selectedItemId}/reject`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ actor: 'human_architect', reason })
    });
    if (res.ok) {
      await fetchQueue();
    }
  } catch (err) {
    alert('Error rejecting item: ' + err);
  }
};

// Filter Tab handlers
document.querySelectorAll('.filter-tab').forEach(tab => {
  tab.onclick = () => {
    document.querySelectorAll('.filter-tab').forEach(t => t.classList.remove('active'));
    tab.classList.add('active');
    currentFilter = tab.getAttribute('data-filter');
    renderQueue();
  };
});

// SDLC Modal
document.getElementById('btn-sdlc-settings').onclick = async () => {
  document.getElementById('modal-sdlc').classList.remove('hidden');
  try {
    const res = await fetch('/api/sdlc/settings');
    if (res.ok) {
      const data = await res.json();
      document.getElementById('sdlc-repo-url').value = data.remote_repo.repo_url;
      document.getElementById('sdlc-provider').value = data.remote_repo.provider;
      document.getElementById('sdlc-default-branch').value = data.remote_repo.default_branch;
      document.getElementById('sdlc-ci-type').value = data.pipeline_plugin.ci_type;
      
      const scRes = await fetch('/api/sdlc/scaffold');
      if (scRes.ok) {
        const scData = await scRes.json();
        document.getElementById('sdlc-scaffold-preview').textContent = JSON.stringify(scData, null, 2);
      }
    }
  } catch (err) {
    console.error('Failed to load SDLC settings:', err);
  }
};

document.getElementById('btn-close-sdlc').onclick = () => document.getElementById('modal-sdlc').classList.add('hidden');
document.getElementById('btn-cancel-sdlc').onclick = () => document.getElementById('modal-sdlc').classList.add('hidden');

// New Event Ingestion Modal
document.getElementById('btn-new-item').onclick = () => {
  document.getElementById('modal-ingest').classList.remove('hidden');
};
document.getElementById('btn-close-ingest').onclick = () => document.getElementById('modal-ingest').classList.add('hidden');
document.getElementById('btn-cancel-ingest').onclick = () => document.getElementById('modal-ingest').classList.add('hidden');

document.getElementById('btn-submit-ingest').onclick = async () => {
  const domain = document.getElementById('new-item-domain').value;
  const title = document.getElementById('new-item-title').value;
  const desc = document.getElementById('new-item-desc').value;
  const requester = document.getElementById('new-item-requester').value;

  if (!title) {
    alert('Please provide a title');
    return;
  }

  let payload;
  if (domain === '#prod-bug') {
    payload = {
      domain_type: 'prod_bug',
      data: { commit_id: 'git-rev-latest', error_message: title, reproduction_command: 'cargo check' }
    };
  } else if (domain === '#crm-request') {
    payload = {
      domain_type: 'crm_request',
      data: { account_id: 'acc-new', account_name: 'Acme Enterprises', current_arr: 120000, requested_discount_pct: 15, requested_tier: 'Enterprise', requester_role: 'AccountExec' }
    };
  } else {
    payload = {
      domain_type: 'survey_mapping',
      data: { batch_id: 'batch-manual', source_platform: 'UI-Manual', target_taxonomy_id: 'root', responses: [{ id: 'resp-1', feedback_text: title, candidate_taxonomy_node: 'tax-perf-latency' }] }
    };
  }

  try {
    const res = await fetch('/api/operations/ingest', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title, description: desc, requester, payload })
    });
    if (res.ok) {
      document.getElementById('modal-ingest').classList.add('hidden');
      await fetchQueue();
    } else {
      alert('Ingestion failed: ' + await res.text());
    }
  } catch (err) {
    alert('Failed to ingest event: ' + err);
  }
};

function escapeHtml(str) {
  if (!str) return '';
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function formatTime(isoStr) {
  if (!isoStr) return '';
  try {
    const d = new Date(isoStr);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } catch (_) {
    return isoStr;
  }
}

// Initial fetch & polling
fetchQueue();
setInterval(fetchQueue, 5000);
"#;
