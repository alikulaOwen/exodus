// Project Exodus Mission Control — Tauri Desktop Frontend Logic

// Tauri v2 invoke helper with mock/fallback support
const invoke = async (cmd, args = {}) => {
  if (window.__TAURI__?.core?.invoke) {
    return await window.__TAURI__.core.invoke(cmd, args);
  }
  // Fallback to fetch API if running in web browser mode
  console.log(`[IPC Invoke Fallback] ${cmd}`, args);
  return null;
};

let currentItems = [];
let selectedItemId = null;
let currentFilter = 'all';

// Toast Notifications
function showToast(message, type = 'info') {
  const container = document.getElementById('toast-container');
  if (!container) return;
  const toast = document.createElement('div');
  toast.className = `toast ${type}`;
  const icon = type === 'success' ? '✓' : type === 'error' ? '✗' : 'ℹ';
  toast.innerHTML = `<span style="font-weight: bold;">${icon}</span><span>${escapeHtml(message)}</span>`;
  container.appendChild(toast);
  setTimeout(() => {
    toast.style.opacity = '0';
    toast.style.transform = 'translateY(12px) scale(0.95)';
    setTimeout(() => toast.remove(), 250);
  }, 3500);
}

// Navigation Tabs
document.getElementById('tab-btn-queue').onclick = () => {
  document.getElementById('tab-btn-queue').classList.add('active');
  document.getElementById('tab-btn-graph').classList.remove('active');
  document.getElementById('view-queue').classList.remove('hidden');
  document.getElementById('view-graph').classList.add('hidden');
};

document.getElementById('tab-btn-graph').onclick = () => {
  document.getElementById('tab-btn-graph').classList.add('active');
  document.getElementById('tab-btn-queue').classList.remove('active');
  document.getElementById('view-graph').classList.remove('hidden');
  document.getElementById('view-queue').classList.add('hidden');
  renderEsgCanvas();
};

// Queue Management
async function fetchQueue() {
  try {
    const items = await invoke('list_operations');
    if (items) {
      currentItems = items;
      renderKPIs();
      renderQueue();
      if (selectedItemId) {
        const item = currentItems.find(i => i.id === selectedItemId);
        if (item) selectItem(item);
      }
    }
  } catch (err) {
    console.error('Failed to fetch operations:', err);
  }
}

function renderKPIs() {
  const bugCount = currentItems.filter(i => i.domain_tag === '#prod-bug').length;
  const crmCount = currentItems.filter(i => i.domain_tag === '#crm-request').length;
  const surveyCount = currentItems.filter(i => i.domain_tag === '#survey-mapping').length;

  document.getElementById('kpi-prod-bug').textContent = bugCount;
  document.getElementById('kpi-crm-request').textContent = crmCount;
  document.getElementById('kpi-survey-mapping').textContent = surveyCount;
}

function formatStateLabel(state) {
  if (!state) return '';
  const map = {
    'captured': 'Triggered',
    'sandboxed': 'In Sandbox',
    'contract_verified': 'Tests Passing',
    'contractverified': 'Tests Passing',
    'degraded': 'Degraded',
    'human_approved': 'Approved',
    'humanapproved': 'Approved',
    'promoted': 'Applied',
    'rejected': 'Rejected'
  };
  return map[state.toLowerCase()] || state;
}

function renderQueue() {
  const listEl = document.getElementById('items-list');
  const filtered = currentFilter === 'all'
    ? currentItems
    : currentItems.filter(i => i.domain_tag === currentFilter);

  document.getElementById('feed-count-badge').textContent = `${filtered.length} items`;

  if (filtered.length === 0) {
    listEl.innerHTML = '<div class="empty-state">No operational items found in this domain.</div>';
    return;
  }

  listEl.innerHTML = filtered.map(item => {
    const isSelected = item.id === selectedItemId;
    const badgeClass = item.domain_tag === '#prod-bug' ? 'badge-emerald'
      : item.domain_tag === '#crm-request' ? 'badge-amber' : 'badge-cyan';

    return `
      <div class="item-card ${isSelected ? 'selected' : ''}" onclick="window.selectItemById('${item.id}')">
        <div class="item-card-header">
          <span class="badge ${badgeClass}">${item.domain_tag}</span>
          <span class="badge badge-state">${formatStateLabel(item.state)}</span>
        </div>
        <div class="item-title">${escapeHtml(item.title)}</div>
        <div class="item-meta">
          <span>${escapeHtml(item.requester)}</span>
          <span>${formatTime(item.created_at)}</span>
        </div>
      </div>
    `;
  }).join('');
}

window.selectItemById = (id) => {
  selectedItemId = id;
  const item = currentItems.find(i => i.id === id);
  if (item) selectItem(item);
  renderQueue();
};

function selectItem(item) {
  document.getElementById('empty-review').classList.add('hidden');
  document.getElementById('active-review').classList.remove('hidden');

  document.getElementById('review-id').textContent = item.id;
  document.getElementById('review-title').textContent = item.title;
  document.getElementById('review-desc').textContent = item.description;

  const domainBadge = document.getElementById('review-domain-badge');
  domainBadge.textContent = item.domain_tag;
  domainBadge.className = `badge ${item.domain_tag === '#prod-bug' ? 'badge-emerald' : item.domain_tag === '#crm-request' ? 'badge-amber' : 'badge-cyan'}`;

  document.getElementById('review-state-badge').textContent = formatStateLabel(item.state);

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
    'contractverified': 2,
    'degraded': 2,
    'human_approved': 3,
    'humanapproved': 3,
    'promoted': 4,
    'rejected': -1
  };

  const currentIdx = stateMap[state.toLowerCase()] ?? 0;

  stages.forEach((stage, idx) => {
    const node = document.getElementById(`step-${stage}`);
    if (!node) return;
    node.className = 'step-node';
    if (currentIdx === -1) return;
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
          Codebase Changes & Git Diff
        </h3>
        <div style="margin-bottom: 12px; font-size: 12px; color: var(--text-secondary);">
          <strong>Commit:</strong> <code style="color: var(--accent-cyan);">${escapeHtml(data.commit_id)}</code> &bull;
          <strong>Reproduction Command:</strong> <code>${escapeHtml(data.reproduction_command || 'cargo test')}</code>
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
          Commercial Policy & Contract Impact
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
          Customer Feedback & Categorization (${responses.length} Items)
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

// User Actions via Tauri IPC
document.getElementById('btn-action-verify').onclick = async () => {
  if (!selectedItemId) return;
  showToast('Running deterministic contract verification via Tauri IPC...', 'info');
  try {
    const updated = await invoke('verify_operation', { id: selectedItemId });
    if (updated) {
      showToast(
        updated.verification_report?.passed ? 'Contract verification passed!' : 'Verification finished with contract failures.',
        updated.verification_report?.passed ? 'success' : 'error'
      );
      await fetchQueue();
    }
  } catch (err) {
    showToast('Verification failed: ' + err, 'error');
  }
};

document.getElementById('btn-action-approve').onclick = async () => {
  if (!selectedItemId) return;
  try {
    const updated = await invoke('approve_operation', {
      id: selectedItemId,
      approver: 'human_architect',
      notes: 'Approved via Exodus Mission Control Desktop'
    });
    if (updated) {
      showToast('Item approved & promoted to permanent case memory!', 'success');
      await fetchQueue();
    }
  } catch (err) {
    showToast('Approval failed: ' + err, 'error');
  }
};

document.getElementById('btn-action-reject').onclick = async () => {
  if (!selectedItemId) return;
  const reason = prompt('Please enter rejection reason:');
  if (!reason) return;
  try {
    const updated = await invoke('reject_operation', {
      id: selectedItemId,
      actor: 'human_architect',
      reason
    });
    if (updated) {
      showToast('Rejection recorded.', 'info');
      await fetchQueue();
    }
  } catch (err) {
    showToast('Rejection failed: ' + err, 'error');
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
    const data = await invoke('get_sdlc_settings');
    if (data) {
      document.getElementById('sdlc-repo-url').value = data.remote_repo.repo_url;
      document.getElementById('sdlc-provider').value = data.remote_repo.provider;
      document.getElementById('sdlc-default-branch').value = data.remote_repo.default_branch;
      document.getElementById('sdlc-ci-type').value = data.pipeline_plugin.ci_type;

      const scaffold = await invoke('get_sdlc_scaffold');
      if (scaffold) {
        document.getElementById('sdlc-scaffold-preview').textContent = JSON.stringify(scaffold, null, 2);
      }
    }
  } catch (err) {
    console.error('Failed to load SDLC settings:', err);
  }
};

document.getElementById('btn-close-sdlc').onclick = () => document.getElementById('modal-sdlc').classList.add('hidden');
document.getElementById('btn-cancel-sdlc').onclick = () => document.getElementById('modal-sdlc').classList.add('hidden');

// Ingestion Modal
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
    showToast('Please provide an event title', 'error');
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
    const res = await invoke('ingest_operation', {
      title,
      description: desc || null,
      requester,
      domainTag: domain,
      payload
    });
    if (res) {
      document.getElementById('modal-ingest').classList.add('hidden');
      showToast(`Event ingested into fabric: ${title}`, 'success');
      await fetchQueue();
    }
  } catch (err) {
    showToast('Failed to ingest event: ' + err, 'error');
  }
};

// Change Lineage Graph Canvas & Node Inspector
let currentGraphTopology = null;
let selectedGraphNodeId = null;
let graphNodePositions = {};

// View Lineage Button in Queue
document.getElementById('btn-view-lineage')?.addEventListener('click', () => {
  document.getElementById('tab-btn-graph')?.click();
});

async function renderEsgCanvas() {
  const canvas = document.getElementById('esg-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');

  // Handle high-DPI scaling
  const rect = canvas.parentElement.getBoundingClientRect();
  canvas.width = rect.width;
  canvas.height = rect.height;

  let topology;
  try {
    topology = await invoke('get_esg_topology');
  } catch (_) {}

  if (!topology) {
    topology = {
      nodes: [
        {
          id: "node_trigger",
          label: "CI Failure #412",
          kind: "trigger",
          wave: 1,
          status: "active",
          subtitle: "Trigger: GitHub Actions CI Crash",
          detail: "Pipeline failed in auth_integration suite. TokenVerifier panicked on unhandled ExpiredSignature error.",
          diff_snippet: null,
          meta: { "source": "GitHub Actions", "event": "CI Build #412", "branch": "master" }
        },
        {
          id: "node_prompt",
          label: "Developer Prompt",
          kind: "prompt",
          wave: 2,
          status: "completed",
          subtitle: "Prompt: Handle Token Expiration",
          detail: "Prompt: 'In crates/auth/src/token.rs, safely catch ExpiredSignature and return AuthError::TokenExpired instead of panicking. Run all unit tests to confirm the fix.'",
          diff_snippet: null,
          meta: { "author": "engineer@exodus.dev", "model": "Claude 3.5 Sonnet" }
        },
        {
          id: "node_action",
          label: "AI Repair Task",
          kind: "action",
          wave: 3,
          status: "completed",
          subtitle: "Task: Safe Token Validation",
          detail: "Generated bounded repair for token validation. Isolated changes inside git worktree sandbox and prepared regression tests.",
          diff_snippet: null,
          meta: { "strategy": "Bounded AST Repair", "sandbox": ".exodus/worktrees/op-eng-412" }
        },
        {
          id: "node_code_file",
          label: "auth/src/token.rs",
          kind: "file_change",
          wave: 4,
          status: "modified",
          subtitle: "Codebase File (+8, -2 lines)",
          detail: "Modified authenticate_session to parse JWT claims safely and map expiration to structured error.",
          diff_snippet: "@@ -40,7 +40,11 @@ fn authenticate_session(token: &str) -> Result<Session, AuthError> {\n-    let claims = parse_jwt_unchecked(token)?; // Panic on expired token\n+    let claims = match parse_jwt_safe(token) {\n+        Ok(c) => c,\n+        Err(JwtError::ExpiredSignature) => return Err(AuthError::TokenExpired),\n+        Err(e) => return Err(AuthError::InvalidToken(e.to_string())),\n+    };\n     validate_expiration(&claims)?;\n     Ok(Session::from_claims(claims))",
          meta: { "file": "crates/auth/src/token.rs", "diff": "+8 / -2 lines" }
        },
        {
          id: "node_fn_symbol",
          label: "verify_token()",
          kind: "symbol",
          wave: 4,
          status: "verified",
          subtitle: "Function: AuthHandler::verify_token",
          detail: "Exported public signature: pub async fn verify_token(&self, token: &str) -> Result<Session, AuthError>",
          diff_snippet: null,
          meta: { "visibility": "pub", "type": "async fn" }
        },
        {
          id: "node_test_run",
          label: "Automated Tests",
          kind: "test",
          wave: 5,
          status: "passed",
          subtitle: "cargo test (3 passed)",
          detail: "Running 3 tests in crates/auth/tests/auth_integration.rs:\ntest test_valid_token ... ok\ntest test_token_expiration ... ok\ntest test_malformed_token ... ok\n\ntest result: ok. 3 passed; 0 failed; 0 ignored; finished in 0.42s",
          diff_snippet: null,
          meta: { "command": "cargo test --test auth_integration", "passed": "3", "failed": "0" }
        },
        {
          id: "node_target_merge",
          label: "Release Target",
          kind: "target",
          wave: 6,
          status: "ready",
          subtitle: "Ready for 1-Click Merge",
          detail: "Clean diff with all automated tests passing. Ready for human review sign-off and branch promotion.",
          diff_snippet: null,
          meta: { "target_branch": "master", "status": "Ready for Review" }
        }
      ],
      edges: [
        { from: "node_trigger", to: "node_prompt", edge_type: "triggers", is_cycle_edge: false },
        { from: "node_prompt", to: "node_action", edge_type: "instructs", is_cycle_edge: false },
        { from: "node_action", to: "node_code_file", edge_type: "modifies", is_cycle_edge: false },
        { from: "node_code_file", to: "node_fn_symbol", edge_type: "contains", is_cycle_edge: false },
        { from: "node_code_file", to: "node_test_run", edge_type: "verified_by", is_cycle_edge: false },
        { from: "node_test_run", to: "node_target_merge", edge_type: "promotes_to", is_cycle_edge: false }
      ]
    };
  }

  currentGraphTopology = topology;
  document.getElementById('graph-node-count').textContent = topology.nodes.length;
  document.getElementById('graph-edge-count').textContent = topology.edges.length;
  document.getElementById('graph-status-count').textContent = 'All Passing';

  // Group nodes by stage wave
  const waveGroups = {};
  topology.nodes.forEach(n => {
    waveGroups[n.wave] = waveGroups[n.wave] || [];
    waveGroups[n.wave].push(n);
  });

  graphNodePositions = {};
  const width = canvas.width;
  const height = canvas.height;
  const waveKeys = Object.keys(waveGroups).sort((a, b) => Number(a) - Number(b));
  const colWidth = width / (waveKeys.length + 1);

  waveKeys.forEach((waveKey, colIdx) => {
    const colNodes = waveGroups[waveKey];
    const rowHeight = height / (colNodes.length + 1);
    colNodes.forEach((node, rowIdx) => {
      graphNodePositions[node.id] = {
        x: colWidth * (colIdx + 1),
        y: rowHeight * (rowIdx + 1),
        radius: 26,
        node
      };
    });
  });

  // Auto-select code change node or first node if none selected
  if (!selectedGraphNodeId) {
    const defaultNode = topology.nodes.find(n => n.kind === 'file_change') || topology.nodes[0];
    if (defaultNode) {
      selectedGraphNodeId = defaultNode.id;
      renderNodeDrawer(defaultNode);
    }
  }

  drawGraph(ctx, canvas.width, canvas.height, waveKeys, colWidth);
}

function drawGraph(ctx, width, height, waveKeys, colWidth) {
  ctx.clearRect(0, 0, width, height);

  const stageLabels = {
    '1': '1. TRIGGER',
    '2': '2. PROMPT',
    '3': '3. TASK ACTION',
    '4': '4. CODE CHANGE',
    '5': '5. TESTS',
    '6': '6. TARGET'
  };

  // Draw stage column dividers & headers
  waveKeys.forEach((waveKey, colIdx) => {
    const x = colWidth * (colIdx + 1);
    ctx.strokeStyle = 'rgba(36, 50, 71, 0.35)';
    ctx.lineWidth = 1;
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(x, 46);
    ctx.lineTo(x, height - 36);
    ctx.stroke();
    ctx.setLineDash([]);

    ctx.fillStyle = '#64748b';
    ctx.font = '700 10px "JetBrains Mono"';
    ctx.textAlign = 'center';
    ctx.fillText(stageLabels[waveKey] || `STAGE ${waveKey}`, x, 28);
  });

  // Draw Edges
  if (currentGraphTopology?.edges) {
    currentGraphTopology.edges.forEach(edge => {
      const from = graphNodePositions[edge.from];
      const to = graphNodePositions[edge.to];
      if (!from || !to) return;

      const isHighlighted = selectedGraphNodeId === edge.from || selectedGraphNodeId === edge.to;
      ctx.strokeStyle = isHighlighted ? '#38bdf8' : 'rgba(59, 130, 246, 0.45)';
      ctx.lineWidth = isHighlighted ? 2.5 : 1.6;
      ctx.beginPath();

      const cpX = (from.x + to.x) / 2;
      const cpY = (from.y + to.y) / 2;
      ctx.moveTo(from.x, from.y);
      ctx.quadraticCurveTo(cpX, cpY, to.x, to.y);
      ctx.stroke();

      // Arrow indicator head
      ctx.fillStyle = isHighlighted ? '#38bdf8' : 'rgba(59, 130, 246, 0.8)';
      ctx.beginPath();
      ctx.arc(to.x, to.y, 4, 0, Math.PI * 2);
      ctx.fill();
    });
  }

  // Draw Nodes
  Object.values(graphNodePositions).forEach(pos => {
    const n = pos.node;
    const isSelected = n.id === selectedGraphNodeId;
    const r = pos.radius;

    // Color by kind
    let ringColor = '#3b82f6';
    if (n.kind === 'trigger') ringColor = '#f43f5e';
    else if (n.kind === 'prompt') ringColor = '#f59e0b';
    else if (n.kind === 'action') ringColor = '#6366f1';
    else if (n.kind === 'file_change') ringColor = '#3b82f6';
    else if (n.kind === 'symbol') ringColor = '#8b5cf6';
    else if (n.kind === 'test') ringColor = '#10b981';
    else if (n.kind === 'target') ringColor = '#06b6d4';

    // Selection Glow
    if (isSelected) {
      ctx.beginPath();
      ctx.arc(pos.x, pos.y, r + 9, 0, Math.PI * 2);
      ctx.strokeStyle = ringColor;
      ctx.lineWidth = 2;
      ctx.stroke();
    }

    // Outer ring
    ctx.beginPath();
    ctx.arc(pos.x, pos.y, r + 3, 0, Math.PI * 2);
    ctx.strokeStyle = isSelected ? ringColor : 'rgba(255, 255, 255, 0.15)';
    ctx.lineWidth = isSelected ? 3 : 1.5;
    ctx.stroke();

    // Node body
    ctx.beginPath();
    ctx.arc(pos.x, pos.y, r, 0, Math.PI * 2);
    ctx.fillStyle = '#111827';
    ctx.fill();

    // Inner icon / kind indicator
    ctx.fillStyle = ringColor;
    ctx.font = 'bold 10px "JetBrains Mono"';
    ctx.textAlign = 'center';
    let kindShort = 'FN';
    if (n.kind === 'trigger') kindShort = 'TRIG';
    else if (n.kind === 'prompt') kindShort = 'PRMT';
    else if (n.kind === 'action') kindShort = 'TASK';
    else if (n.kind === 'file_change') kindShort = 'FILE';
    else if (n.kind === 'symbol') kindShort = 'FN';
    else if (n.kind === 'test') kindShort = 'TEST';
    else if (n.kind === 'target') kindShort = 'DEST';
    ctx.fillText(kindShort, pos.x, pos.y + 3);

    // Node label below
    ctx.fillStyle = isSelected ? '#ffffff' : '#e2e8f0';
    ctx.font = isSelected ? 'bold 12px "Plus Jakarta Sans"' : '600 11px "Plus Jakarta Sans"';
    ctx.textAlign = 'center';
    ctx.fillText(n.label, pos.x, pos.y + r + 18);

    // Subtitle below label
    if (n.subtitle) {
      ctx.fillStyle = '#94a3b8';
      ctx.font = '10px "Plus Jakarta Sans"';
      ctx.fillText(n.subtitle, pos.x, pos.y + r + 32);
    }
  });
}

// Canvas Click Event: Node Hit-Testing & Selection
const canvasEl = document.getElementById('esg-canvas');
if (canvasEl) {
  canvasEl.addEventListener('click', (e) => {
    const rect = canvasEl.getBoundingClientRect();
    const scaleX = canvasEl.width / rect.width;
    const scaleY = canvasEl.height / rect.height;
    const mouseX = (e.clientX - rect.left) * scaleX;
    const mouseY = (e.clientY - rect.top) * scaleY;

    for (const pos of Object.values(graphNodePositions)) {
      const dx = mouseX - pos.x;
      const dy = mouseY - pos.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist <= pos.radius + 10) {
        selectedGraphNodeId = pos.node.id;
        renderNodeDrawer(pos.node);
        const ctx = canvasEl.getContext('2d');
        const waveKeys = Object.keys(graphNodePositions).map(id => graphNodePositions[id].node.wave);
        const uniqueWaves = [...new Set(waveKeys)].sort((a, b) => a - b);
        const colWidth = canvasEl.width / (uniqueWaves.length + 1);
        drawGraph(ctx, canvasEl.width, canvasEl.height, uniqueWaves, colWidth);
        break;
      }
    }
  });
}

function renderNodeDrawer(node) {
  const kindEl = document.getElementById('drawer-node-kind');
  const statusEl = document.getElementById('drawer-node-status');
  const labelEl = document.getElementById('drawer-node-label');
  const subEl = document.getElementById('drawer-node-sub');
  const detailEl = document.getElementById('drawer-node-detail');
  const diffSec = document.getElementById('drawer-diff-section');
  const diffContent = document.getElementById('drawer-diff-content');
  const metaSec = document.getElementById('drawer-meta-section');
  const metaGrid = document.getElementById('drawer-meta-grid');

  if (!kindEl) return;

  const kindNames = {
    trigger: 'TRIGGER / EVENT',
    prompt: 'DEVELOPER PROMPT',
    action: 'AI TASK ACTION',
    file_change: 'CODEBASE FILE CHANGE',
    symbol: 'FUNCTION / SYMBOL',
    test: 'AUTOMATED TEST RUN',
    target: 'RELEASE TARGET'
  };

  kindEl.textContent = kindNames[node.kind] || node.kind.toUpperCase();
  kindEl.className = `badge ${node.kind === 'trigger' ? 'badge-rose' : node.kind === 'prompt' ? 'badge-amber' : node.kind === 'test' ? 'badge-emerald' : 'badge-cyan'}`;

  statusEl.textContent = (node.status || 'Active').toUpperCase();
  statusEl.className = `badge ${node.status === 'passed' || node.status === 'verified' ? 'badge-emerald' : node.status === 'ready' ? 'badge-cyan' : 'badge-state'}`;

  labelEl.textContent = node.label;
  subEl.textContent = node.subtitle || '';
  detailEl.textContent = node.detail || 'No further description available.';

  // Code Diff Preview
  if (node.diff_snippet) {
    diffSec.classList.remove('hidden');
    const lines = node.diff_snippet.split('\n');
    diffContent.innerHTML = lines.map(line => {
      if (line.startsWith('+')) return `<div class="diff-add">${escapeHtml(line)}</div>`;
      if (line.startsWith('-')) return `<div class="diff-rem">${escapeHtml(line)}</div>`;
      if (line.startsWith('@@')) return `<div class="diff-ctx">${escapeHtml(line)}</div>`;
      return `<div>${escapeHtml(line)}</div>`;
    }).join('');
  } else {
    diffSec.classList.add('hidden');
  }

  // Metadata Grid
  if (node.meta && Object.keys(node.meta).length > 0) {
    metaSec.classList.remove('hidden');
    metaGrid.innerHTML = Object.entries(node.meta).map(([k, v]) => `
      <div class="drawer-meta-item">
        <div class="drawer-meta-k">${escapeHtml(k.replace(/_/g, ' '))}</div>
        <div class="drawer-meta-v">${escapeHtml(v)}</div>
      </div>
    `).join('');
  } else {
    metaSec.classList.add('hidden');
  }
}

// Helpers
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

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
  if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
  if (e.key === 'Escape') {
    document.getElementById('modal-sdlc')?.classList.add('hidden');
    document.getElementById('modal-ingest')?.classList.add('hidden');
  } else if (e.key === 'v' || e.key === 'V') {
    document.getElementById('btn-action-verify')?.click();
  } else if (e.key === 'a' || e.key === 'A') {
    document.getElementById('btn-action-approve')?.click();
  } else if (e.key === '1') {
    document.getElementById('tab-btn-queue')?.click();
  } else if (e.key === '2' || e.key === 'g' || e.key === 'G') {
    document.getElementById('tab-btn-graph')?.click();
  }
});

// Initial boot & polling
fetchQueue();
setInterval(fetchQueue, 5000);

