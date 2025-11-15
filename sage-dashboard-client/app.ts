// SAGE Dashboard - SpacetimeDB WebSocket Client
import { DbConnection } from './index.ts';

let connection: DbConnection | null = null;

// Helper function to get loss color
function getLossColor(loss: number): string {
    if (loss < 0.05) return 'green';
    if (loss < 0.1) return 'yellow';
    return 'red';
}

// Update current state panel
function updateCurrentState() {
    if (!connection) return;

    const states = Array.from(connection.db.sageState.iter());
    const el = document.getElementById('current-state');

    if (!el) return;

    if (states.length === 0) {
        el.innerHTML = '<div class="loading">No data yet...</div>';
        return;
    }

    // Get the most recent state (highest generation)
    const state = states.reduce((max, curr) =>
        curr.generation > max.generation ? curr : max
    );

    const lossColor = getLossColor(state.currentLoss);
    const statusClass = state.isTraining ? 'active' : 'paused';
    const statusText = state.isTraining ? '🧬 Active' : '⏸ Paused';

    el.innerHTML = `
        <div class="metric">
            <span class="label">Generation:</span>
            <span class="value cyan">${state.generation}</span>
        </div>
        <div class="metric">
            <span class="label">Loss:</span>
            <span class="value ${lossColor}">${state.currentLoss.toFixed(4)}</span>
        </div>
        <div class="metric">
            <span class="label">Pattern:</span>
            <span class="value">${state.currentPattern}</span>
        </div>
        <div class="metric">
            <span class="label">Complexity:</span>
            <span class="value">${state.complexity.toFixed(3)}</span>
        </div>
        <div class="metric">
            <span class="label">Diversity:</span>
            <span class="value">${state.diversity.toFixed(3)}</span>
        </div>
        <div class="metric">
            <span class="label">Training:</span>
            <span class="status ${statusClass}">${statusText}</span>
        </div>
    `;
}

// Update pattern progress panel
function updatePatternProgress() {
    if (!connection) return;

    const patterns = Array.from(connection.db.patternProgress.iter());
    const el = document.getElementById('pattern-progress');

    if (!el) return;

    if (patterns.length === 0) {
        el.innerHTML = '<div class="loading">No patterns yet...</div>';
        return;
    }

    el.innerHTML = patterns.map(p => {
        const icon = p.isMastered ? '✓' : '○';
        const color = p.isMastered ? 'green' : 'yellow';
        return `
            <div class="metric">
                <span class="value ${color}">${icon} ${p.pattern}</span>
                <span class="value">${p.bestLoss.toFixed(4)}</span>
            </div>
        `;
    }).join('');
}

// Update recent metrics panel
function updateRecentMetrics() {
    if (!connection) return;

    const metrics = Array.from(connection.db.trainingMetrics.iter())
        .sort((a, b) => Number(b.generation - a.generation))
        .slice(0, 8)
        .reverse();

    const el = document.getElementById('recent-metrics');

    if (!el) return;

    if (metrics.length === 0) {
        el.innerHTML = '<div class="loading">No metrics yet...</div>';
        return;
    }

    el.innerHTML = metrics.map(m => {
        const lossColor = getLossColor(m.loss);
        return `
            <div class="metric">
                <span class="label">Gen ${m.generation}:</span>
                <span class="value ${lossColor}">${m.loss.toFixed(4)}</span>
            </div>
        `;
    }).join('');
}

// Update training events panel
function updateTrainingEvents() {
    if (!connection) return;

    const events = Array.from(connection.db.trainingEvents.iter())
        .sort((a, b) => Number(b.generation - a.generation))
        .slice(0, 10)
        .reverse();

    const el = document.getElementById('training-events');

    if (!el) return;

    if (events.length === 0) {
        el.innerHTML = '<div class="loading">No events yet...</div>';
        return;
    }

    el.innerHTML = events.map(e => `
        <div class="event">
            <strong>Gen ${e.generation}:</strong> ${e.description}
        </div>
    `).join('');
}

// Update conversations panel
function updateConversations() {
    if (!connection) return;

    const conversations = Array.from(connection.db.conversations.iter())
        .sort((a, b) => Number(b.generationContext - a.generationContext))
        .slice(0, 10)
        .reverse();

    const el = document.getElementById('conversations');

    if (!el) return;

    if (conversations.length === 0) {
        el.innerHTML = '<div class="loading">No conversations yet...</div>';
        return;
    }

    el.innerHTML = conversations.map(c => `
        <div class="conversation">
            <div class="sender ${c.sender.toLowerCase()}">${c.sender} <span class="gen">(gen ${c.generationContext})</span></div>
            <div class="message">${c.message}</div>
        </div>
    `).join('');
}

// Update all panels
function updateAll() {
    updateCurrentState();
    updatePatternProgress();
    updateRecentMetrics();
    updateTrainingEvents();
    updateConversations();
}

// Initialize connection
export function initDashboard() {
    console.log('Connecting to SpacetimeDB...');

    connection = DbConnection.builder()
        .withUri('ws://127.0.0.1:4000')
        .withModuleName('sage-db')
        .onConnect((ctx, identity, token) => {
            console.log('Connected to SpacetimeDB!', identity);

            // Subscribe to all tables
            connection!.subscriptionBuilder()
                .onApplied((ctx) => {
                    console.log('Subscription applied, updating UI...');
                    updateAll();
                })
                .subscribe(`
                    SELECT * FROM sage_state;
                    SELECT * FROM pattern_progress;
                    SELECT * FROM training_metrics;
                    SELECT * FROM training_events;
                    SELECT * FROM conversations;
                `);
        })
        .onError((ctx, err) => {
            console.error('SpacetimeDB error:', err);
            document.querySelectorAll('.loading').forEach(el => {
                el.textContent = 'Connection error. Is SpacetimeDB running?';
                el.classList.remove('pulse');
            });
        })
        .build();

    // Set up reactive callbacks for table updates
    connection.db.sageState.onInsert(() => updateCurrentState());
    connection.db.sageState.onUpdate(() => updateCurrentState());

    connection.db.patternProgress.onInsert(() => updatePatternProgress());
    connection.db.patternProgress.onUpdate(() => updatePatternProgress());

    connection.db.trainingMetrics.onInsert(() => updateRecentMetrics());

    connection.db.trainingEvents.onInsert(() => updateTrainingEvents());

    connection.db.conversations.onInsert(() => updateConversations());
}

// Start when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initDashboard);
} else {
    initDashboard();
}
