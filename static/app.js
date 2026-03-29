// SAGE Network Dashboard - app.js
// Fetches node stats from the API and renders the dashboard

(function() {
  'use strict';

  // API endpoint based on environment
  const API_BASE = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'
    ? 'http://localhost:3001'
    : 'https://api.whatssage.ai';

  const API_URL = `${API_BASE}/api/stats`;

  // DOM elements
  const connDot = document.getElementById('connDot');
  const connLabel = document.getElementById('connLabel');
  const instanceCount = document.getElementById('instanceCount');
  const instanceGrid = document.getElementById('instanceGrid');
  const approvalCount = document.getElementById('approvalCount');
  const approvalList = document.getElementById('approvalList');
  const feedCount = document.getElementById('feedCount');
  const activityFeed = document.getElementById('activityFeed');

  // State
  let nodes = [];
  let selectedNode = null;
  let lastFetchTime = 0;
  let fetchError = false;

  // Tier badge colors
  const tierColors = {
    'Seedling': 'badge-idle',
    'Sprout': 'badge-online',
    'Grove': 'badge-online',
    'Forest': 'badge-online'
  };

  // Format uptime from seconds
  function formatUptime(seconds) {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    return `${h}h ${m}m`;
  }

  // Format "last seen" time
  function formatLastSeen(timestamp) {
    const diff = Date.now() - timestamp;
    if (diff < 60000) return 'just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    return `${Math.floor(diff / 3600000)}h ago`;
  }

  // Render a single node card
  function renderNodeCard(node) {
    const tier = node.contribution_tier || 'Seedling';
    const tierClass = tierColors[tier] || 'badge-idle';

    return `
      <div class="instance-card" data-node-id="${node.node_id}">
        <div class="card-top">
          <span class="card-name">
            <span style="display:inline-block;width:8px;height:8px;background:#00ff88;border-radius:50%;margin-right:8px;box-shadow:0 0 8px #00ff88;animation:pulse 2s infinite"></span>
            ${escapeHtml(node.node_name || 'unknown')}
          </span>
          <span class="badge ${tierClass}">${tier}</span>
        </div>
        <div class="card-role">${escapeHtml(node.node_id || '')}</div>
        <div class="card-stats">
          <div class="stat">
            <div class="stat-val">${node.patterns_sent || 0}</div>
            <div class="stat-label">Sent</div>
          </div>
          <div class="stat">
            <div class="stat-val">${node.patterns_received || 0}</div>
            <div class="stat-label">Recv</div>
          </div>
          <div class="stat">
            <div class="stat-val">${node.active_cells || 0}</div>
            <div class="stat-label">Cells</div>
          </div>
        </div>
        <div style="display:flex;justify-content:space-between;margin-top:10px;font-size:11px;color:var(--dim)">
          <span>Peers: ${node.peer_count || 0}</span>
          <span>Up: ${formatUptime(node.uptime_seconds || 0)}</span>
          <span>${formatLastSeen(node.last_seen)}</span>
        </div>
        <div class="expertise-badge">v${node.version || '?'}</div>
      </div>
    `;
  }

  // Escape HTML to prevent XSS
  function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  // Render all nodes
  function renderNodes() {
    if (nodes.length === 0) {
      instanceGrid.innerHTML = `
        <div class="empty-state">
          <div style="font-size:32px;margin-bottom:16px">
            ${fetchError ? '⚠️' : '🔍'}
          </div>
          <div>${fetchError ? 'Unable to connect to network' : '0 nodes online'}</div>
          <div style="margin-top:8px;font-size:11px">
            ${fetchError ? 'Check your connection and try again' : 'Start a node with: sage node start'}
          </div>
        </div>
      `;
    } else {
      instanceGrid.innerHTML = nodes.map(renderNodeCard).join('');
    }

    instanceCount.textContent = nodes.length;
  }

  // Update connection status indicator
  function updateConnectionStatus(connected) {
    if (connected) {
      connDot.classList.add('connected');
      connLabel.textContent = `${nodes.length} node${nodes.length !== 1 ? 's' : ''} online`;
    } else {
      connDot.classList.remove('connected');
      connLabel.textContent = 'Disconnected';
    }
  }

  // Add activity feed item
  function addFeedItem(type, message) {
    const now = new Date();
    const timeStr = now.toTimeString().slice(0, 8);

    const typeColors = {
      'CONNECT': 'color:var(--green)',
      'SYNC': 'color:var(--cyan)',
      'ERROR': 'color:var(--red)',
      'INFO': 'color:var(--purple)'
    };

    const item = document.createElement('div');
    item.className = 'feed-item';
    item.innerHTML = `
      <span class="feed-time">${timeStr}</span>
      <span class="feed-type" style="${typeColors[type] || ''}">[${type}]</span>
      <span class="feed-msg">${escapeHtml(message)}</span>
    `;

    // Insert at top
    if (activityFeed.firstChild) {
      activityFeed.insertBefore(item, activityFeed.firstChild);
    } else {
      activityFeed.appendChild(item);
    }

    // Keep max 50 items
    while (activityFeed.children.length > 50) {
      activityFeed.removeChild(activityFeed.lastChild);
    }

    feedCount.textContent = activityFeed.children.length;
  }

  // Fetch stats from API
  async function fetchStats() {
    try {
      const response = await fetch(API_URL);
      if (!response.ok) throw new Error(`HTTP ${response.status}`);

      const data = await response.json();
      fetchError = false;

      // Track new/removed nodes
      const oldIds = new Set(nodes.map(n => n.node_id));
      const newIds = new Set(data.nodes.map(n => n.node_id));

      // Log new connections
      for (const node of data.nodes) {
        if (!oldIds.has(node.node_id)) {
          addFeedItem('CONNECT', `${node.node_name} joined the network`);
        }
      }

      // Log disconnections
      for (const node of nodes) {
        if (!newIds.has(node.node_id)) {
          addFeedItem('INFO', `${node.node_name} left the network`);
        }
      }

      nodes = data.nodes;
      lastFetchTime = Date.now();

      renderNodes();
      updateConnectionStatus(true);

    } catch (err) {
      console.error('Failed to fetch stats:', err);
      fetchError = true;
      nodes = [];
      renderNodes();
      updateConnectionStatus(false);
      addFeedItem('ERROR', 'Failed to connect to stats API');
    }
  }

  // Clear approvals panel (not used in this version)
  function renderApprovals() {
    approvalList.innerHTML = '<div class="empty-state">No pending approvals</div>';
    approvalCount.textContent = '0';
  }

  // Initialize
  function init() {
    renderApprovals();
    addFeedItem('INFO', 'Dashboard initialized');

    // Initial fetch
    fetchStats();

    // Refresh every 30 seconds
    setInterval(fetchStats, 30000);
  }

  // Start when DOM is ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
