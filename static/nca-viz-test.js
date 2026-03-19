#!/usr/bin/env node
// Tests for the NCA visualizer logic
// Run: node nca-viz-test.js

const G = 48;

function makeGrid() {
    return Array.from({length: G}, () => new Array(G).fill(0));
}

function inject(activation, cx, cy, strength, radius) {
    for (let y = 0; y < G; y++) {
        for (let x = 0; x < G; x++) {
            const dist = Math.sqrt((x-cx)**2 + (y-cy)**2);
            if (dist < radius) {
                activation[y][x] = Math.min(1, activation[y][x] + strength * (1 - dist/radius));
            }
        }
    }
}

function onEncode(activation, text) {
    const n = Math.max(3, Math.min(8, Math.floor(text.length / 4)));
    for (let i = 0; i < n; i++) {
        let hash = 0;
        for (let j = 0; j < text.length; j++) hash = (hash * 31 + text.charCodeAt(j) + i * 137) & 0xffff;
        const cx = (hash % G);
        const cy = (Math.floor(hash / G) % G);
        inject(activation, cx, cy, 0.9, 4 + (i % 3));
    }
    for (let i = 0; i < 6; i++) {
        inject(activation, Math.floor(Math.random() * G), Math.floor(Math.random() * G), 0.6, 2);
    }
}

function stepNCA(activation, memory, isAnimating) {
    const next = makeGrid();
    // Total energy budget: decay * self + spread * neighbors, must sum < 1 to guarantee decay
    // decay + 4*spread < 1 ensures system is energy-dissipative
    const decay = isAnimating ? 0.80 : 0.72;
    const spread = isAnimating ? 0.045 : 0.03; // 4*0.045=0.18, 0.80+0.18=0.98 < 1 ✓
    for (let y = 0; y < G; y++) {
        for (let x = 0; x < G; x++) {
            const n4 = (
                activation[(y-1+G)%G][x] +
                activation[(y+1)%G][x] +
                activation[y][(x-1+G)%G] +
                activation[y][(x+1)%G]
            ) * spread;
            const noise = (Math.random() - 0.5) * (isAnimating ? 0.04 : 0.008);
            // Memory only adds a tiny glow floor — capped so it can't drive saturation
            const mem = Math.min(0.008, memory[y][x] * 0.015);
            next[y][x] = Math.min(1, Math.max(0, activation[y][x] * decay + n4 + noise + mem));
        }
    }
    return next;
}

function stats(grid, label) {
    const flat = grid.flat();
    const mean = flat.reduce((a,b)=>a+b,0) / flat.length;
    const max = Math.max(...flat);
    const active = flat.filter(v => v > 0.01).length;
    const saturated = flat.filter(v => v > 0.9).length;
    console.log(`  ${label}: mean=${mean.toFixed(4)} max=${max.toFixed(3)} active=${active}/${flat.length} saturated=${saturated}`);
    return { mean, max, active, saturated };
}

let passed = 0;
let failed = 0;
function assert(condition, msg) {
    if (condition) { console.log(`  ✅ ${msg}`); passed++; }
    else { console.log(`  ❌ FAIL: ${msg}`); failed++; }
}

// ── Test 1: Idle decay — grid should go quiet without input
console.log('\n── Test 1: Idle decay (no input, 200 frames)');
{
    let activation = makeGrid();
    let memory = makeGrid();
    // Start with some activation
    inject(activation, 24, 24, 0.8, 6);
    stats(activation, 'after inject');
    for (let i = 0; i < 200; i++) activation = stepNCA(activation, memory, false);
    const s = stats(activation, 'after 200 idle frames');
    assert(s.mean < 0.05, 'Grid decays to near-zero at idle');
    assert(s.saturated === 0, 'No saturated cells at idle');
}

// ── Test 2: Single message encode — should light up but not saturate
console.log('\n── Test 2: Single message encode');
{
    let activation = makeGrid();
    let memory = makeGrid();
    onEncode(activation, 'what is sage');
    const s = stats(activation, 'immediately after encode');
    assert(s.active > 20, `Has active cells (got ${s.active})`);
    assert(s.mean < 0.3, `Mean stays reasonable (got ${s.mean.toFixed(3)})`);
    assert(s.saturated < 100, `Not too many saturated cells (got ${s.saturated})`);
}

// ── Test 3: Encode then decay — activation should spread then die
console.log('\n── Test 3: Encode → 30 frames processing → 100 frames idle');
{
    let activation = makeGrid();
    let memory = makeGrid();
    onEncode(activation, 'how does the nca brain work');
    stats(activation, 'after encode');
    // 30 frames "processing" (isAnimating=true)
    for (let i = 0; i < 30; i++) activation = stepNCA(activation, memory, true);
    const s30 = stats(activation, 'after 30 processing frames');
    // 100 frames idle
    for (let i = 0; i < 100; i++) activation = stepNCA(activation, memory, false);
    const s130 = stats(activation, 'after 100 idle frames');
    assert(s30.mean > 0.01, 'Still active during processing');
    assert(s130.mean < s30.mean, 'Decays after processing ends');
    assert(s130.saturated < 20, `Few saturated cells at rest (got ${s130.saturated})`);
}

// ── Test 4: Multiple messages — does it accumulate and saturate?
console.log('\n── Test 4: 5 messages in a row (worst case)');
{
    let activation = makeGrid();
    let memory = makeGrid();
    const msgs = ['hello', 'what are you', 'how do you learn', 'tell me about nodes', 'whats the nca grid'];
    for (const msg of msgs) {
        onEncode(activation, msg);
        for (let i = 0; i < 30; i++) activation = stepNCA(activation, memory, true);
        // commit to memory
        for (let y = 0; y < G; y++)
            for (let x = 0; x < G; x++)
                memory[y][x] = Math.min(1, memory[y][x] * 0.85 + activation[y][x] * 0.3);
        for (let i = 0; i < 20; i++) activation = stepNCA(activation, memory, false);
    }
    const s = stats(activation, 'after 5 messages');
    const sm = stats(memory, 'memory after 5 messages');
    assert(s.saturated < 200, `Activation doesn't fully saturate (got ${s.saturated} saturated)`);
    assert(sm.mean < 0.4, `Memory mean stays reasonable (got ${sm.mean.toFixed(3)})`);
}

// ── Test 5: Idle sparks — one spark every 40 frames, check steady state
console.log('\n── Test 5: Idle spark rate over 2000 frames');
{
    let activation = makeGrid();
    let memory = makeGrid();
    let idleTick = 0;
    for (let frame = 0; frame < 2000; frame++) {
        idleTick++;
        if (idleTick % 40 === 0) {
            inject(activation, Math.floor(Math.random() * G), Math.floor(Math.random() * G), 0.4, 2);
        }
        activation = stepNCA(activation, memory, false);
    }
    const s = stats(activation, 'after 2000 idle frames with sparks');
    assert(s.mean > 0.002, 'Grid stays alive (not dead)');
    assert(s.mean < 0.1, `Grid stays subtle at idle (got ${s.mean.toFixed(4)})`);
    assert(s.saturated < 10, `No saturation at idle (got ${s.saturated})`);
}

// ── Test 6: What parameters keep it alive without saturating?
console.log('\n── Test 6: Parameter sensitivity — find safe decay/spread values');
{
    const configs = [
        { decay: 0.72, spread: 0.03, label: 'new idle' },
        { decay: 0.80, spread: 0.045, label: 'new active' },
        { decay: 0.65, spread: 0.03, label: 'more aggressive decay' },
        { decay: 0.78, spread: 0.05, label: 'slightly hotter active' },
    ];
    for (const cfg of configs) {
        let activation = makeGrid();
        const memory = makeGrid();
        onEncode(activation, 'test message here for sage');
        for (let i = 0; i < 60; i++) {
            const next = makeGrid();
            for (let y = 0; y < G; y++) {
                for (let x = 0; x < G; x++) {
                    const n4 = (activation[(y-1+G)%G][x]+activation[(y+1)%G][x]+activation[y][(x-1+G)%G]+activation[y][(x+1)%G]) * cfg.spread;
                    next[y][x] = Math.min(1, Math.max(0, activation[y][x] * cfg.decay + n4));
                }
            }
            activation = next;
        }
        const s = stats(activation, cfg.label);
        assert(s.mean < 0.15 && s.saturated < 50, `${cfg.label}: mean=${s.mean.toFixed(3)} sat=${s.saturated} — ${s.mean < 0.15 && s.saturated < 50 ? 'OK' : 'TOO HOT'}`);
    }
}

console.log(`\n── Results: ${passed} passed, ${failed} failed\n`);
if (failed > 0) process.exit(1);
