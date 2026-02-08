// SAGE Neural Command Dashboard
(() => {
  'use strict';

  // State
  let ws = null;
  let reconnectTimer = null;
  let reconnectDelay = 1000;
  let instances = [];
  let pendingApprovals = [];
  let activityLog = [];
  let selectedInstance = null;
  const MAX_FEED = 200;

  // DOM
  const $ = id => document.getElementById(id);
  const connDot = $('connDot');
  const connLabel = $('connLabel');
  const instanceGrid = $('instanceGrid');
  const instanceCount = $('instanceCount');
  const approvalList = $('approvalList');
  const approvalCount = $('approvalCount');
  const activityFeed = $('activityFeed');
  const feedCount = $('feedCount');
  const modal = $('modal');
  const modalTitle = $('modalTitle');
  const modalContent = $('modalContent');

  // ── WebSocket ──
  function connectWS() {
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${proto}//${location.host}/api/ws`);

    ws.onopen = () => {
      setConnected(true);
      reconnectDelay = 1000;
      addActivity('system', 'WebSocket connected');
    };

    ws.onmessage = (e) => {
      try {
        const data = JSON.parse(e.data);
        handleUpdate(data);
      } catch {}
    };

    ws.onclose = () => {
      setConnected(false);
      scheduleReconnect();
    };

    ws.onerror = () => {
      ws.close();
    };
  }

  function scheduleReconnect() {
    clearTimeout(reconnectTimer);
    reconnectTimer = setTimeout(() => {
      addActivity('system', `Reconnecting (${(reconnectDelay/1000).toFixed(0)}s)...`);
      connectWS();
      reconnectDelay = Math.min(reconnectDelay * 1.5, 30000);
    }, reconnectDelay);
  }

  function setConnected(ok) {
    connDot.classList.toggle('connected', ok);
    connLabel.textContent = ok ? 'Connected' : 'Disconnected';
  }

  // ── Data Handling ──
  function handleUpdate(data) {
    if (data.instances) {
      const oldMap = new Map(instances.map(i => [i.instance_id, i]));
      instances = data.instances;

      // Detect changes for activity feed
      for (const inst of instances) {
        const old = oldMap.get(inst.instance_id);
        if (!old) {
          addActivity('instance', `${inst.name} came online`, 'green');
        } else if (old.status !== inst.status) {
          addActivity('status', `${inst.name}: ${old.status} → ${inst.status}`, inst.status === 'online' ? 'green' : 'red');
        } else if (old.total_tasks !== inst.total_tasks) {
          addActivity('task', `${inst.name} completed task #${inst.total_tasks}`, 'cyan');
        }
      }
      renderInstances();
    }
    if (data.pending_count !== undefined) {
      approvalCount.textContent = data.pending_count;
      if (data.pending_count > 0) fetchPending();
    }
  }

  // ── API ──
  async function api(path, opts = {}) {
    try {
      const r = await fetch(`/api/${path}`, {
        headers: { 'Content-Type': 'application/json' },
        ...opts
      });
      return await r.json();
    } catch (e) {
      addActivity('error', `API error: ${e.message}`, 'red');
      return null;
    }
  }

  async function fetchInstances() {
    const res = await api('instances');
    if (res?.success) {
      instances = res.data;
      renderInstances();
    }
  }

  async function fetchPending() {
    const res = await api('pending');
    if (res?.success) {
      pendingApprovals = res.data;
      renderApprovals();
    }
  }

  async function approveAction(id) {
    const res = await api(`approve/${id}`, { method: 'POST' });
    if (res?.success) {
      addActivity('approve', `Action #${id} approved`, 'green');
      fetchPending();
    }
  }

  async function rejectAction(id) {
    const reason = prompt('Rejection reason (optional):') || '';
    const res = await api(`reject/${id}?reason=${encodeURIComponent(reason)}`, { method: 'POST' });
    if (res?.success) {
      addActivity('reject', `Action #${id} rejected`, 'red');
      fetchPending();
    }
  }

  // ── Rendering ──
  function renderInstances() {
    instanceCount.textContent = instances.length;
    instanceGrid.innerHTML = instances.map(i => `
      <div class="instance-card${selectedInstance === i.instance_id ? ' selected' : ''}" data-id="${esc(i.instance_id)}">
        <div class="card-top">
          <span class="card-name">${esc(i.name)}</span>
          <span class="badge badge-${i.status}">${esc(i.status)}</span>
        </div>
        <div class="card-role">${esc(i.role)}</div>
        <div class="card-stats">
          <div class="stat"><div class="stat-val">${i.total_tasks}</div><div class="stat-label">Tasks</div></div>
          <div class="stat"><div class="stat-val">${i.success_rate.toFixed(1)}%</div><div class="stat-label">Success</div></div>
          <div class="stat"><div class="stat-val">${i.pending_approvals}</div><div class="stat-label">Pending</div></div>
        </div>
        <span class="expertise-badge">${esc(i.expertise_level)}</span>
      </div>
    `).join('');

    if (!instances.length) {
      instanceGrid.innerHTML = '<div class="empty-state">No instances registered.<br>Start a SAGE instance to see it here.</div>';
    }

    instanceGrid.querySelectorAll('.instance-card').forEach(card => {
      card.addEventListener('click', () => openDetail(card.dataset.id));
    });
  }

  function renderApprovals() {
    approvalCount.textContent = pendingApprovals.length;
    if (!pendingApprovals.length) {
      approvalList.innerHTML = '<div class="empty-state">✓ No pending approvals</div>';
      return;
    }
    approvalList.innerHTML = pendingApprovals.map(a => `
      <div class="approval-item">
        <div class="approval-top">
          <span class="approval-type">${esc(a.action_type)}</span>
          <span class="risk risk-${a.risk_level}">${esc(a.risk_level)}</span>
        </div>
        <div class="approval-desc">${esc(a.description)}</div>
        <div class="approval-instance">${esc(a.instance_id)}</div>
        <div class="approval-actions">
          <button class="btn btn-approve" data-approve="${a.id}">✓ Approve</button>
          <button class="btn btn-reject" data-reject="${a.id}">✗ Reject</button>
        </div>
      </div>
    `).join('');

    approvalList.querySelectorAll('[data-approve]').forEach(btn =>
      btn.addEventListener('click', () => approveAction(btn.dataset.approve))
    );
    approvalList.querySelectorAll('[data-reject]').forEach(btn =>
      btn.addEventListener('click', () => rejectAction(btn.dataset.reject))
    );
  }

  // ── Activity Feed ──
  function addActivity(type, msg, color) {
    const now = new Date();
    const time = now.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
    activityLog.unshift({ time, type, msg, color });
    if (activityLog.length > MAX_FEED) activityLog.length = MAX_FEED;
    feedCount.textContent = activityLog.length;
    renderFeed();
  }

  function renderFeed() {
    const colorMap = { green: 'var(--green)', red: 'var(--red)', cyan: 'var(--cyan)', purple: 'var(--purple)' };
    activityFeed.innerHTML = activityLog.slice(0, 50).map(e => `
      <div class="feed-item">
        <span class="feed-time">${e.time}</span>
        <span class="feed-type" style="color:${colorMap[e.color] || 'var(--dim)'}">${esc(e.type)}</span>
        <span class="feed-msg">${esc(e.msg)}</span>
      </div>
    `).join('');
  }

  // ── Detail Modal ──
  async function openDetail(id) {
    selectedInstance = id;
    renderInstances();
    modalTitle.textContent = id;
    modalContent.innerHTML = '<div class="empty-state">Loading...</div>';
    modal.classList.add('active');

    const [expertise, tasks] = await Promise.all([
      api(`expertise/${encodeURIComponent(id)}`),
      api(`tasks/${encodeURIComponent(id)}`)
    ]);

    let html = '';

    // Expertise
    if (expertise?.success && expertise.data) {
      const e = expertise.data;
      html += `<div style="margin-bottom:16px">
        <div style="font-size:12px;color:var(--dim);margin-bottom:4px">ROLE</div>
        <div style="font-size:14px;color:#fff;margin-bottom:12px">${esc(e.role)} — <span style="color:var(--purple)">${esc(e.level)}</span></div>
        <div style="font-size:12px;color:var(--dim);margin-bottom:4px">OVERALL SCORE</div>
        <div style="font-size:24px;font-weight:700;color:var(--cyan);margin-bottom:16px">${e.overall_score.toFixed(1)}%</div>
      </div>`;

      if (e.skills?.length) {
        html += '<div style="font-size:12px;color:var(--dim);margin-bottom:8px">SKILLS</div>';
        for (const s of e.skills) {
          const pct = Math.min(s.score * 100, 100);
          html += `<div class="skill-bar-wrap">
            <div class="skill-label"><span>${esc(s.name)}</span><span>${pct.toFixed(0)}%</span></div>
            <div class="skill-track"><div class="skill-fill" style="width:${pct}%"></div></div>
          </div>`;
        }
      }

      if (e.milestones?.length) {
        html += '<div style="font-size:12px;color:var(--dim);margin:16px 0 8px">MILESTONES</div>';
        html += e.milestones.map(m => `<span class="expertise-badge" style="margin:2px">${esc(m.name)}</span>`).join('');
      }
    }

    // Tasks
    if (tasks?.success && tasks.data?.length) {
      html += `<div style="font-size:12px;color:var(--dim);margin:20px 0 8px">RECENT TASKS</div>
        <table class="task-table">
          <thead><tr><th>#</th><th>Type</th><th>Result</th><th>Time</th><th>Approved</th></tr></thead>
          <tbody>`;
      for (const t of tasks.data.slice(0, 20)) {
        html += `<tr>
          <td>${t.id}</td>
          <td>${esc(t.task_type)}</td>
          <td class="${t.success ? 'task-success' : 'task-fail'}">${t.success ? '✓' : '✗'}</td>
          <td>${t.execution_time_ms}ms</td>
          <td>${t.human_approved ? '✓' : '—'}</td>
        </tr>`;
      }
      html += '</tbody></table>';
    }

    if (!html) html = '<div class="empty-state">No data available for this instance.</div>';
    modalContent.innerHTML = html;
  }

  // ── Utilities ──
  function esc(s) {
    if (s == null) return '';
    const d = document.createElement('div');
    d.textContent = String(s);
    return d.innerHTML;
  }

  // ── Events ──
  $('modalClose').addEventListener('click', () => {
    modal.classList.remove('active');
    selectedInstance = null;
    renderInstances();
  });
  modal.addEventListener('click', (e) => {
    if (e.target === modal) {
      modal.classList.remove('active');
      selectedInstance = null;
      renderInstances();
    }
  });

  // ── Init ──
  addActivity('system', 'SAGE Neural Command initialized', 'purple');
  fetchInstances();
  fetchPending();
  connectWS();

  // Periodic fallback poll
  setInterval(() => {
    fetchInstances();
    fetchPending();
  }, 30000);
})();
