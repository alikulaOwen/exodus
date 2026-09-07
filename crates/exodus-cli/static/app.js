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
    if (currentIdx === -1) {
      // Handled as rejected state
      return;
    }
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

// User Actions
document.getElementById('btn-action-verify').onclick = async () => {
  if (!selectedItemId) return;
  showToast('Running deterministic contract verification...', 'info');
  try {
    const res = await fetch(`/api/operations/${selectedItemId}/verify`, { method: 'POST' });
    if (res.ok) {
      const data = await res.json();
      const passed = data.verification_report?.passed;
      showToast(
        passed ? 'Contract verification passed!' : 'Verification finished with contract failures.',
        passed ? 'success' : 'error'
      );
      await fetchQueue();
    } else {
      showToast('Verification failed: ' + await res.text(), 'error');
    }
  } catch (err) {
    showToast('Error running verification: ' + err, 'error');
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
      showToast('Item approved & promoted to permanent case memory!', 'success');
      await fetchQueue();
    } else {
      showToast('Approval failed: ' + await res.text(), 'error');
    }
  } catch (err) {
    showToast('Error approving item: ' + err, 'error');
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
      showToast('Item rejection recorded.', 'info');
      await fetchQueue();
    } else {
      showToast('Rejection failed: ' + await res.text(), 'error');
    }
  } catch (err) {
    showToast('Error rejecting item: ' + err, 'error');
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
      showToast(`Event ingested into fabric: ${title}`, 'success');
      await fetchQueue();
    } else {
      showToast('Ingestion failed: ' + await res.text(), 'error');
    }
  } catch (err) {
    showToast('Failed to ingest event: ' + err, 'error');
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

// Global Keyboard Shortcuts
document.addEventListener('keydown', (e) => {
  if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
  if (e.key === 'Escape') {
    document.getElementById('modal-sdlc')?.classList.add('hidden');
    document.getElementById('modal-ingest')?.classList.add('hidden');
  } else if (e.key === 'v' || e.key === 'V') {
    document.getElementById('btn-action-verify')?.click();
  } else if (e.key === 'a' || e.key === 'A') {
    document.getElementById('btn-action-approve')?.click();
  }
});

// Initial fetch & polling
fetchQueue();
setInterval(fetchQueue, 5000);
