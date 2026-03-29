const express = require('express');
const cors = require('cors');

const app = express();
const PORT = process.env.PORT || 3001;

// In-memory store for node stats
// Key: node_id, Value: { stats, lastSeen }
const nodeStats = new Map();

// Eviction threshold: 5 minutes
const EVICTION_MS = 5 * 60 * 1000;

app.use(cors());
app.use(express.json());

// POST /api/stats - receive stats from a node
app.post('/api/stats', (req, res) => {
  const stats = req.body;

  if (!stats || !stats.node_id) {
    return res.status(400).json({ error: 'Missing node_id' });
  }

  nodeStats.set(stats.node_id, {
    stats,
    lastSeen: Date.now()
  });

  res.json({ ok: true });
});

// GET /api/stats - return all active nodes
app.get('/api/stats', (req, res) => {
  const now = Date.now();
  const active = [];

  // Evict stale nodes and collect active ones
  for (const [nodeId, entry] of nodeStats.entries()) {
    if (now - entry.lastSeen > EVICTION_MS) {
      nodeStats.delete(nodeId);
    } else {
      active.push({
        ...entry.stats,
        last_seen: entry.lastSeen
      });
    }
  }

  res.json({
    nodes: active,
    count: active.length,
    timestamp: now
  });
});

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'ok', nodes: nodeStats.size });
});

app.listen(PORT, () => {
  console.log(`SAGE Stats API listening on port ${PORT}`);
});
