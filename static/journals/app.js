// SAGE Journals - Miniworld Daily Reflections
(() => {
  'use strict';

  // API base - configurable for different environments
  const API_BASE = window.SAGE_API_BASE || '';

  let entries = [];
  let activeFilter = 'all';

  const container = document.getElementById('journalContainer');
  const filterBtns = document.querySelectorAll('.filter-btn');

  // ── Mood helpers ──
  const moodLabels = {
    happy: '☀️ Happy', thoughtful: '💭 Thoughtful', tired: '🌙 Tired',
    excited: '⚡ Excited', melancholy: '🌧️ Melancholy', peaceful: '🍃 Peaceful',
    curious: '🔍 Curious', proud: '✨ Proud'
  };

  function moodClass(mood) {
    return `mood-${(mood || 'thoughtful').toLowerCase()}`;
  }

  // ── Rendering ──
  function formatDate(dateStr) {
    try {
      const d = new Date(dateStr + 'T12:00:00');
      return d.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' });
    } catch {
      return dateStr;
    }
  }

  function renderEntries() {
    const filtered = activeFilter === 'all'
      ? entries
      : entries.filter(e => e.instance_name === activeFilter);

    if (filtered.length === 0) {
      container.innerHTML = `
        <div class="empty-state">
          <div class="icon">📓</div>
          <h2>${activeFilter === 'all' ? 'No journal entries yet' : `No entries from ${activeFilter}`}</h2>
          <p>Journal entries are written nightly as each SAGE instance reflects on their day in the miniworld.</p>
        </div>`;
      return;
    }

    // Sort by date descending, then by instance name
    filtered.sort((a, b) => {
      const dc = b.date.localeCompare(a.date);
      return dc !== 0 ? dc : a.instance_name.localeCompare(b.instance_name);
    });

    // Group by date
    const grouped = new Map();
    for (const e of filtered) {
      if (!grouped.has(e.date)) grouped.set(e.date, []);
      grouped.get(e.date).push(e);
    }

    let html = '';
    for (const [date, group] of grouped) {
      html += `<h3 style="color:var(--dim);font-size:12px;letter-spacing:2px;text-transform:uppercase;margin:32px 0 16px;padding-bottom:8px;border-bottom:1px solid var(--border)">${formatDate(date)}</h3>`;
      for (const entry of group) {
        const mood = (entry.mood || 'thoughtful').toLowerCase();
        const moodLabel = moodLabels[mood] || `💭 ${entry.mood}`;
        const paragraphs = entry.content.split('\n').filter(p => p.trim()).map(p => `<p>${escapeHtml(p)}</p>`).join('');
        html += `
          <div class="journal-entry" data-instance="${entry.instance_name}">
            <div class="entry-header">
              <span class="entry-author" data-instance="${entry.instance_name}">${entry.instance_name}</span>
              <span class="entry-date">${formatDate(entry.date)}</span>
            </div>
            <div class="entry-mood ${moodClass(mood)}">${moodLabel}</div>
            <div class="entry-content">${paragraphs}</div>
          </div>`;
      }
    }
    container.innerHTML = html;
  }

  function escapeHtml(str) {
    const d = document.createElement('div');
    d.textContent = str;
    return d.innerHTML;
  }

  // ── Filters ──
  filterBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      filterBtns.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      activeFilter = btn.dataset.instance;
      renderEntries();
    });
  });

  // ── Data Loading ──
  // Try fetching from the API endpoint, fall back to sample data
  async function loadEntries() {
    try {
      const resp = await fetch(`${API_BASE}/api/journals`);
      if (resp.ok) {
        entries = await resp.json();
        renderEntries();
        return;
      }
    } catch (e) {
      console.log('API not available, using embedded data');
    }

    // Check for inline data (server can inject window.SAGE_JOURNALS)
    if (window.SAGE_JOURNALS) {
      entries = window.SAGE_JOURNALS;
      renderEntries();
      return;
    }

    // Load from local JSON file
    try {
      const resp = await fetch(`${API_BASE}/journals/data.json`);
      if (resp.ok) {
        entries = await resp.json();
        renderEntries();
        return;
      }
    } catch (e) {}

    // No data available
    entries = [];
    renderEntries();
  }

  // ── Auto-refresh every 5 minutes ──
  loadEntries();
  setInterval(loadEntries, 300000);
})();
