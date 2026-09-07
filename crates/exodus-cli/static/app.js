// Project Exodus — Autonomous Code & Systems Workspace Client Logic

let operationalItems = [];
let selectedItemId = null;
let currentFilter = 'all';
let currentGraphTopology = null;
let selectedGraphNodeId = null;
let graphNodePositions = {};

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

let boardFilter = 'all';
let isAutoPipelineRunning = false;
let recentlyMovedId = null;

// Navigation Tabs
document.getElementById('tab-btn-board')?.addEventListener('click', () => {
  document.getElementById('tab-btn-board').classList.add('active');
  document.getElementById('tab-btn-queue').classList.remove('active');
  document.getElementById('tab-btn-graph').classList.remove('active');
  document.getElementById('view-board').classList.remove('hidden');
  document.getElementById('view-queue').classList.add('hidden');
  document.getElementById('view-graph').classList.add('hidden');
  renderKanbanBoard();
});

document.getElementById('tab-btn-queue')?.addEventListener('click', () => {
  document.getElementById('tab-btn-queue').classList.add('active');
  document.getElementById('tab-btn-board').classList.remove('active');
  document.getElementById('tab-btn-graph').classList.remove('active');
  document.getElementById('view-queue').classList.remove('hidden');
  document.getElementById('view-board').classList.add('hidden');
  document.getElementById('view-graph').classList.add('hidden');
});

document.getElementById('tab-btn-graph')?.addEventListener('click', () => {
  document.getElementById('tab-btn-graph').classList.add('active');
  document.getElementById('tab-btn-board').classList.remove('active');
  document.getElementById('tab-btn-queue').classList.remove('active');
  document.getElementById('view-graph').classList.remove('hidden');
  document.getElementById('view-board').classList.add('hidden');
  document.getElementById('view-queue').classList.add('hidden');
  renderEsgCanvas();
});

document.getElementById('btn-view-lineage')?.addEventListener('click', () => {
  document.getElementById('tab-btn-graph')?.click();
});

// Kanban Board Toolbar Filters
document.querySelectorAll('.board-filters .filter-chip').forEach(chip => {
  chip.addEventListener('click', (e) => {
    document.querySelectorAll('.board-filters .filter-chip').forEach(c => c.classList.remove('active'));
    e.target.classList.add('active');
    boardFilter = e.target.getAttribute('data-filter');
    renderKanbanBoard();
  });
});

// Auto-Run Pipeline
document.getElementById('btn-auto-pipeline')?.addEventListener('click', async () => {
  if (isAutoPipelineRunning) return;
  isAutoPipelineRunning = true;
  const btn = document.getElementById('btn-auto-pipeline');
  btn.classList.add('btn-primary-highlight');
  btn.innerHTML = `
    <span class="pulse-dot"></span>
    <span>Simulating Pipeline...</span>
  `;

  showToast('Starting automated pipeline execution...', 'info');

  try {
    // 1. Advance Captured -> In Sandbox -> Tests Passing
    for (const item of operationalItems) {
      const state = item.state.toLowerCase();
      if (state === 'captured') {
        recentlyMovedId = item.id;
        await fetch(`/api/operations/${item.id}/verify`, { method: 'POST' });
        await fetchQueue();
        await new Promise(r => setTimeout(r, 1200));
      }
    }

    // 2. Advance Tests Passing -> Approved -> Applied
    for (const item of operationalItems) {
      const state = item.state.toLowerCase();
      if (state === 'contract_verified' || state === 'contractverified' || state === 'sandboxed') {
        recentlyMovedId = item.id;
        await fetch(`/api/operations/${item.id}/approve`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ approver: 'exodus-autonomous-pipeline', notes: 'Auto-verified via pipeline runner' })
        });
        await fetchQueue();
        await new Promise(r => setTimeout(r, 1200));
      }
    }

    showToast('Pipeline simulation finished — all eligible tasks advanced!', 'success');
  } catch (err) {
    showToast('Auto-pipeline error: ' + err, 'error');
  } finally {
    isAutoPipelineRunning = false;
    btn.classList.remove('btn-primary-highlight');
    btn.innerHTML = `
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
      <span>Auto-Run Pipeline</span>
    `;
    recentlyMovedId = null;
  }
});

// Domain Filter Tabs
document.querySelectorAll('.domain-filters .filter-tab').forEach(tab => {
  tab.addEventListener('click', (e) => {
    document.querySelectorAll('.domain-filters .filter-tab').forEach(t => t.classList.remove('active'));
    e.target.classList.add('active');
    currentFilter = e.target.getAttribute('data-filter');
    renderQueue();
  });
});

async function fetchQueue() {
  try {
    const res = await fetch('/api/operations');
    if (res.ok) {
      operationalItems = await res.json();
      updateKpis();
      renderQueue();
      renderKanbanBoard();
      if (selectedItemId) {
        const item = operationalItems.find(i => i.id === selectedItemId);
        if (item) renderReview(item);
      }
    }
  } catch (err) {
    console.error('Failed to fetch operations queue:', err);
  }
}

// Linear / Notion Kanban Board Renderer
function renderKanbanBoard() {
  const filtered = boardFilter === 'all'
    ? operationalItems
    : operationalItems.filter(i => i.domain_tag === boardFilter);

  const buckets = {
    captured: [],
    sandboxed: [],
    verified: [],
    approved: [],
    promoted: []
  };

  filtered.forEach(item => {
    const s = item.state.toLowerCase();
    if (s === 'captured') buckets.captured.push(item);
    else if (s === 'sandboxed') buckets.sandboxed.push(item);
    else if (s === 'contract_verified' || s === 'contractverified' || s === 'degraded') buckets.verified.push(item);
    else if (s === 'human_approved' || s === 'humanapproved') buckets.approved.push(item);
    else if (s === 'promoted') buckets.promoted.push(item);
    else buckets.captured.push(item);
  });

  Object.keys(buckets).forEach(stage => {
    const countEl = document.getElementById(`count-${stage}`);
    if (countEl) countEl.textContent = buckets[stage].length;

    const colEl = document.getElementById(`cards-${stage}`);
    if (!colEl) return;

    if (buckets[stage].length === 0) {
      colEl.innerHTML = '<div class="empty-col-placeholder">No tasks in this stage</div>';
    } else {
      colEl.innerHTML = buckets[stage].map(item => renderKanbanCard(item, stage)).join('');
    }

    const colWrapper = document.getElementById(`col-${stage}`);
    if (colWrapper && !colWrapper.dataset.hasDropListener) {
      colWrapper.dataset.hasDropListener = 'true';
      colWrapper.addEventListener('dragover', (e) => {
        e.preventDefault();
        colWrapper.classList.add('drag-over');
      });
      colWrapper.addEventListener('dragleave', () => {
        colWrapper.classList.remove('drag-over');
      });
      colWrapper.addEventListener('drop', async (e) => {
        e.preventDefault();
        colWrapper.classList.remove('drag-over');
        const itemId = e.dataTransfer.getData('text/plain');
        if (itemId) {
          await handleDropOnStage(itemId, stage);
        }
      });
    }
  });

  document.querySelectorAll('.kanban-card').forEach(card => {
    card.addEventListener('dragstart', (e) => {
      e.dataTransfer.setData('text/plain', card.dataset.id);
      card.classList.add('dragging');
    });
    card.addEventListener('dragend', () => {
      card.classList.remove('dragging');
    });
  });
}

function renderKanbanCard(item, stage) {
  const isJustMoved = recentlyMovedId === item.id;
  const badgeClass = item.domain_tag === '#prod-bug' ? 'badge-emerald'
    : item.domain_tag === '#crm-request' ? 'badge-amber' : 'badge-cyan';
  
  const shortKey = item.id.length > 8 ? item.id.substring(item.id.length - 6).toUpperCase() : item.id.toUpperCase();

  let actionBtn = '';
  if (stage === 'captured') {
    actionBtn = `<button class="card-action-btn" onclick="window.advanceTask('${item.id}', 'verify', event)">⚡ Run Tests</button>`;
  } else if (stage === 'sandboxed') {
    actionBtn = `<button class="card-action-btn" onclick="window.advanceTask('${item.id}', 'verify', event)">▶ Run Tests</button>`;
  } else if (stage === 'verified') {
    actionBtn = `<button class="card-action-btn btn-promote" onclick="window.advanceTask('${item.id}', 'approve', event)">✓ Approve</button>`;
  } else if (stage === 'approved') {
    actionBtn = `<button class="card-action-btn btn-promote" onclick="window.advanceTask('${item.id}', 'promote', event)">🚀 Apply</button>`;
  } else if (stage === 'promoted') {
    actionBtn = `<span style="font-size: 10px; color: var(--accent-emerald); font-weight: 700;">✓ In master</span>`;
  }

  return `
    <div class="kanban-card ${isJustMoved ? 'just-moved' : ''}" 
         draggable="true" 
         data-id="${item.id}"
         onclick="window.openCardDetail('${item.id}')">
      <div class="card-top">
        <span class="card-key">${shortKey}</span>
        <span class="badge ${badgeClass}">${item.domain_tag}</span>
      </div>
      <div class="card-title">${escapeHtml(item.title)}</div>
      <div class="card-desc">${escapeHtml(item.description)}</div>
      <div class="card-footer">
        <div class="card-actor">
          <span>👤 ${escapeHtml(item.requester)}</span>
        </div>
        <div class="card-actions">
          ${actionBtn}
        </div>
      </div>
    </div>
  `;
}

window.openCardDetail = (id) => {
  selectItem(id);
  document.getElementById('tab-btn-queue')?.click();
};

window.advanceTask = async (id, action, event) => {
  if (event) event.stopPropagation();
  recentlyMovedId = id;

  try {
    if (action === 'verify') {
      const res = await fetch(`/api/operations/${id}/verify`, { method: 'POST' });
      if (res.ok) showToast('Tests executed and verified in sandbox', 'success');
    } else if (action === 'approve') {
      const res = await fetch(`/api/operations/${id}/approve`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ approver: 'human_operator', notes: 'Approved via Kanban' })
      });
      if (res.ok) showToast('Task signed off and approved', 'success');
    } else if (action === 'promote') {
      const res = await fetch(`/api/operations/${id}/approve`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ approver: 'human_operator', notes: 'Promoted to master' })
      });
      if (res.ok) showToast('Changes promoted and merged into master', 'success');
    }
    await fetchQueue();
  } catch (err) {
    showToast('Failed to advance task: ' + err, 'error');
  }
};

async function handleDropOnStage(itemId, targetStage) {
  recentlyMovedId = itemId;
  try {
    if (targetStage === 'verified' || targetStage === 'sandboxed') {
      await fetch(`/api/operations/${itemId}/verify`, { method: 'POST' });
      showToast(`Task moved to ${formatStateLabel(targetStage)}`, 'success');
    } else if (targetStage === 'approved' || targetStage === 'promoted') {
      await fetch(`/api/operations/${itemId}/approve`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ approver: 'human_operator', notes: `Moved via Kanban drag & drop` })
      });
      showToast(`Task moved to ${formatStateLabel(targetStage)}`, 'success');
    }
    await fetchQueue();
  } catch (err) {
    showToast('Cannot transition task: ' + err, 'error');
  }
}

window.openNewTaskModal = (domainTag) => {
  if (domainTag) {
    const sel = document.getElementById('new-item-domain');
    if (sel) sel.value = domainTag;
  }
  document.getElementById('modal-ingest')?.classList.remove('hidden');
};

function updateKpis() {
  const prodBug = operationalItems.filter(i => i.domain_tag === '#prod-bug').length;
  const crm = operationalItems.filter(i => i.domain_tag === '#crm-request').length;
  const survey = operationalItems.filter(i => i.domain_tag === '#survey-mapping').length;

  const prodBugEl = document.getElementById('kpi-prod-bug');
  const crmEl = document.getElementById('kpi-crm-request');
  const surveyEl = document.getElementById('kpi-survey-mapping');

  if (prodBugEl) prodBugEl.textContent = prodBug;
  if (crmEl) crmEl.textContent = crm;
  if (surveyEl) surveyEl.textContent = survey;
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
  const container = document.getElementById('items-list');
  const filtered = currentFilter === 'all' 
    ? operationalItems 
    : operationalItems.filter(i => i.domain_tag === currentFilter);

  const badgeEl = document.getElementById('feed-count-badge');
  if (badgeEl) badgeEl.textContent = `${filtered.length} items`;

  if (filtered.length === 0) {
    container.innerHTML = '<div class="empty-state" style="padding: 24px; text-align: center; color: var(--text-muted);">No tasks in this category.</div>';
    return;
  }

  container.innerHTML = filtered.map(item => {
    const isSelected = item.id === selectedItemId ? 'selected' : '';
    const badgeColor = item.domain_tag === '#prod-bug' ? 'badge-emerald' : item.domain_tag === '#crm-request' ? 'badge-amber' : 'badge-cyan';
    
    return `
      <div class="item-card ${isSelected}" onclick="selectItem('${item.id}')">
        <div class="item-card-header">
          <span class="badge ${badgeColor}">${item.domain_tag}</span>
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

window.selectItem = (id) => {
  selectedItemId = id;
  renderQueue();
  const item = operationalItems.find(i => i.id === id);
  if (item) {
    renderReview(item);
  }
};

function renderReview(item) {
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

  if (!item.last_verification) {
    summaryEl.textContent = 'Not yet verified. Click "Run Tests" to execute automated checks in sandbox.';
    summaryEl.className = 'verification-summary';
    rulesListEl.innerHTML = '';
    return;
  }

  const ver = item.last_verification;
  const isOk = ver.status === 'Verified' || ver.status === 'Compatible';

  summaryEl.innerHTML = `
    <strong>Status:</strong> <span style="color: ${isOk ? 'var(--accent-emerald)' : 'var(--accent-rose)'};">${ver.status}</span> &bull;
    <strong>Verifier:</strong> ${escapeHtml(ver.verifier_identity)} &bull;
    <strong>Duration:</strong> ${ver.execution_duration_ms}ms
  `;

  rulesListEl.innerHTML = ver.rules_checked.map(rule => {
    const passed = rule.passed;
    return `
      <div class="rule-result-item ${passed ? 'pass' : 'fail'}">
        <div>
          <strong>${escapeHtml(rule.rule_id)}</strong> &bull;
          <span style="color: var(--text-secondary);">${escapeHtml(rule.rule_name)}</span>
        </div>
        <div style="display: flex; gap: 8px; align-items: center;">
          <span style="color: var(--text-muted); font-size: 11px;">${escapeHtml(rule.message)}</span>
          <span class="badge ${passed ? 'badge-emerald' : 'badge-rose'}">${passed ? 'PASSED' : 'FAILED'}</span>
        </div>
      </div>
    `;
  }).join('');
}

function renderAudit(item) {
  const container = document.getElementById('audit-entries');
  if (!item.audit_trail || item.audit_trail.length === 0) {
    container.innerHTML = '<div style="color: var(--text-muted); font-size: 12px;">No activity logged yet.</div>';
    return;
  }

  container.innerHTML = item.audit_trail.map(a => `
    <div style="font-size: 12px; margin-bottom: 8px; display: flex; justify-content: space-between; border-bottom: 1px solid rgba(255,255,255,0.05); padding-bottom: 6px;">
      <div>
        <strong style="color: var(--accent-cyan);">${escapeHtml(a.actor)}</strong> &bull;
        <span>${escapeHtml(a.action)}</span>
        ${a.details ? `<span style="color: var(--text-muted); font-size: 11px;"> (${escapeHtml(a.details)})</span>` : ''}
      </div>
      <span style="color: var(--text-muted); font-family: var(--font-mono); font-size: 11px;">${formatTime(a.timestamp)}</span>
    </div>
  `).join('');
}

// Review Pane Actions
document.getElementById('btn-action-verify')?.addEventListener('click', async () => {
  if (!selectedItemId) return;
  try {
    const res = await fetch(`/api/operations/${selectedItemId}/verify`, { method: 'POST' });
    if (res.ok) {
      showToast('Automated tests passed in sandbox', 'success');
      await fetchQueue();
    } else {
      showToast('Validation failed: ' + (await res.text()), 'error');
    }
  } catch (err) {
    showToast('Failed to run verification: ' + err, 'error');
  }
});

document.getElementById('btn-action-approve')?.addEventListener('click', async () => {
  if (!selectedItemId) return;
  try {
    const res = await fetch(`/api/operations/${selectedItemId}/approve`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ approver: 'human_operator', notes: 'Approved via Web Dashboard' })
    });
    if (res.ok) {
      showToast('Task approved and applied successfully', 'success');
      await fetchQueue();
    } else {
      showToast('Approval error: ' + (await res.text()), 'error');
    }
  } catch (err) {
    showToast('Failed to approve task: ' + err, 'error');
  }
});

document.getElementById('btn-action-reject')?.addEventListener('click', async () => {
  if (!selectedItemId) return;
  try {
    const res = await fetch(`/api/operations/${selectedItemId}/reject`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ actor: 'human_operator', reason: 'Rejected via Web Dashboard' })
    });
    if (res.ok) {
      showToast('Task rejected', 'info');
      await fetchQueue();
    }
  } catch (err) {
    showToast('Failed to reject: ' + err, 'error');
  }
});

// SDLC Settings Modal
document.getElementById('btn-sdlc-settings')?.addEventListener('click', async () => {
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
        const sc = await scRes.json();
        document.getElementById('sdlc-scaffold-preview').textContent = JSON.stringify(sc, null, 2);
      }
    }
  } catch (err) {
    console.error('Failed to load SDLC settings:', err);
  }
});

document.getElementById('btn-close-sdlc')?.addEventListener('click', () => document.getElementById('modal-sdlc').classList.add('hidden'));
document.getElementById('btn-cancel-sdlc')?.addEventListener('click', () => document.getElementById('modal-sdlc').classList.add('hidden'));

// New Task Modal
document.getElementById('btn-new-item')?.addEventListener('click', () => {
  document.getElementById('modal-ingest').classList.remove('hidden');
});
document.getElementById('btn-close-ingest')?.addEventListener('click', () => document.getElementById('modal-ingest').classList.add('hidden'));
document.getElementById('btn-cancel-ingest')?.addEventListener('click', () => document.getElementById('modal-ingest').classList.add('hidden'));

document.getElementById('btn-submit-ingest')?.addEventListener('click', async () => {
  const domain = document.getElementById('new-item-domain').value;
  const title = document.getElementById('new-item-title').value;
  const desc = document.getElementById('new-item-desc').value;
  const requester = document.getElementById('new-item-requester').value;

  if (!title) {
    showToast('Please provide a task title', 'error');
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
      body: JSON.stringify({
        title,
        description: desc || 'Created via Web UI',
        requester,
        payload
      })
    });
    if (res.ok) {
      document.getElementById('modal-ingest').classList.add('hidden');
      showToast(`Task created: ${title}`, 'success');
      await fetchQueue();
    }
  } catch (err) {
    showToast('Failed to create task: ' + err, 'error');
  }
});

// Change Lineage Graph Canvas & Node Inspector
async function renderEsgCanvas() {
  const canvas = document.getElementById('esg-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');

  const rect = canvas.parentElement.getBoundingClientRect();
  canvas.width = rect.width;
  canvas.height = rect.height;

  let topology;
  try {
    const res = await fetch('/api/graph');
    if (res.ok) {
      topology = await res.json();
    }
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

      ctx.fillStyle = isHighlighted ? '#38bdf8' : 'rgba(59, 130, 246, 0.8)';
      ctx.beginPath();
      ctx.arc(to.x, to.y, 4, 0, Math.PI * 2);
      ctx.fill();
    });
  }

  Object.values(graphNodePositions).forEach(pos => {
    const n = pos.node;
    const isSelected = n.id === selectedGraphNodeId;
    const r = pos.radius;

    let ringColor = '#3b82f6';
    if (n.kind === 'trigger') ringColor = '#f43f5e';
    else if (n.kind === 'prompt') ringColor = '#f59e0b';
    else if (n.kind === 'action') ringColor = '#6366f1';
    else if (n.kind === 'file_change') ringColor = '#3b82f6';
    else if (n.kind === 'symbol') ringColor = '#8b5cf6';
    else if (n.kind === 'test') ringColor = '#10b981';
    else if (n.kind === 'target') ringColor = '#06b6d4';

    if (isSelected) {
      ctx.beginPath();
      ctx.arc(pos.x, pos.y, r + 9, 0, Math.PI * 2);
      ctx.strokeStyle = ringColor;
      ctx.lineWidth = 2;
      ctx.stroke();
    }

    ctx.beginPath();
    ctx.arc(pos.x, pos.y, r + 3, 0, Math.PI * 2);
    ctx.strokeStyle = isSelected ? ringColor : 'rgba(255, 255, 255, 0.15)';
    ctx.lineWidth = isSelected ? 3 : 1.5;
    ctx.stroke();

    ctx.beginPath();
    ctx.arc(pos.x, pos.y, r, 0, Math.PI * 2);
    ctx.fillStyle = '#111827';
    ctx.fill();

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

    ctx.fillStyle = isSelected ? '#ffffff' : '#e2e8f0';
    ctx.font = isSelected ? 'bold 12px "Plus Jakarta Sans"' : '600 11px "Plus Jakarta Sans"';
    ctx.textAlign = 'center';
    ctx.fillText(n.label, pos.x, pos.y + r + 18);

    if (n.subtitle) {
      ctx.fillStyle = '#94a3b8';
      ctx.font = '10px "Plus Jakarta Sans"';
      ctx.fillText(n.subtitle, pos.x, pos.y + r + 32);
    }
  });
}

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

// Boot & polling
fetchQueue();
setInterval(fetchQueue, 5000);
