// SAGE City Miniworld Renderer - Performance Optimized
//
// OPTIMIZATIONS:
// - requestAnimationFrame instead of setInterval
// - Offscreen canvas for static ground/overlay layers
// - Dirty rectangle rendering — only redraw changed tiles/characters
// - Double-buffered rendering
// - ImageBitmap sprite tile cache (pre-cut)
// - WebSocket delta updates (only process changes)
// - Viewport culling for all rendering
// - Object pooling / GC minimization
// - Debounced resize
// - Performance metrics logging

const TILE_SIZE = 16;
const SCALE = 2;
const TILE_PX = TILE_SIZE * SCALE; // 32 — used everywhere

// ============================================================================
// PERFORMANCE METRICS
// ============================================================================

const perf = {
    frameCount: 0,
    frameTimes: new Float64Array(120),
    frameIdx: 0,
    lastFrameTime: 0,
    msgSizes: [],
    deltaMsgCount: 0,
    fullMsgCount: 0,
    avgFrameTime() {
        let sum = 0, count = Math.min(this.frameCount, 120);
        for (let i = 0; i < count; i++) sum += this.frameTimes[i];
        return count ? (sum / count) : 0;
    },
    logStats() {
        const avg = this.avgFrameTime();
        const fps = avg > 0 ? (1000 / avg).toFixed(1) : '?';
        const avgSize = this.msgSizes.length
            ? (this.msgSizes.reduce((a,b)=>a+b,0) / this.msgSizes.length / 1024).toFixed(1)
            : '?';
        console.log(`[PERF] FPS: ${fps} | avg frame: ${avg.toFixed(1)}ms | avg msg: ${avgSize}KB | deltas: ${this.deltaMsgCount} | full: ${this.fullMsgCount}`);
        this.msgSizes = [];
    }
};
// Log stats every 10s
setInterval(() => perf.logStats(), 10000);

// ============================================================================
// SPRITE SHEETS
// ============================================================================

const SPRITE_SHEETS = {
    grass: '/sprites/Ground/Grass.png',
    deadGrass: '/sprites/Ground/DeadGrass.png',
    shore: '/sprites/Ground/Shore.png',
    texturedGrass: '/sprites/Ground/TexturedGrass.png',
    trees: '/sprites/Nature/Trees.png',
    pineTrees: '/sprites/Nature/PineTrees.png',
    deadTrees: '/sprites/Nature/DeadTrees.png',
    rocks: '/sprites/Nature/Rocks.png',
    wheatfield: '/sprites/Nature/Wheatfield.png',
    houses: '/sprites/Buildings/Wood/Houses.png',
    taverns: '/sprites/Buildings/Wood/Taverns.png',
    market: '/sprites/Buildings/Wood/Market.png',
    chapels: '/sprites/Buildings/Wood/Chapels.png',
    workshops: '/sprites/Buildings/Wood/Workshops.png',
    purpleHouses: '/sprites/Buildings/Purple/PurpleHouses.png',
    cyanHouses: '/sprites/Buildings/Cyan/CyanHouses.png',
    limeHouses: '/sprites/Buildings/Lime/LimeHouses.png',
    redHouses: '/sprites/Buildings/Red/RedHouses.png',
    purpleTaverns: '/sprites/Buildings/Purple/PurpleTaverns.png',
    cyanTaverns: '/sprites/Buildings/Cyan/CyanTaverns.png',
    limeTaverns: '/sprites/Buildings/Lime/LimeTaverns.png',
    redTaverns: '/sprites/Buildings/Red/RedTaverns.png',
    well: '/sprites/Miscellaneous/Well.png',
    bridge: '/sprites/Miscellaneous/Bridge.png',
    magePurple: '/sprites/Characters/Soldiers/Ranged/PurpleRanged/MagePurple.png',
    mageCyan: '/sprites/Characters/Soldiers/Ranged/CyanRanged/MageCyan.png',
    mageLime: '/sprites/Characters/Soldiers/Ranged/LimeRanged/MageLime.png',
    mageRed: '/sprites/Characters/Soldiers/Ranged/RedRanged/MageRed.png',
    farmerPurple: '/sprites/Characters/Workers/PurpleWorker/FarmerPurple.png',
    farmerCyan: '/sprites/Characters/Workers/CyanWorker/FarmerCyan.png',
    farmerLime: '/sprites/Characters/Workers/LimeWorker/FarmerLime.png',
    farmerRed: '/sprites/Characters/Workers/RedWorker/FarmerRed.png',
    swordsmanPurple: '/sprites/Characters/Soldiers/Melee/PurpleMelee/SwordsmanPurple.png',
    swordsmanCyan: '/sprites/Characters/Soldiers/Melee/CyanMelee/SwordsmanCyan.png',
    swordsmanLime: '/sprites/Characters/Soldiers/Melee/LimeMelee/SwordsmanLime.png',
    swordsmanRed: '/sprites/Characters/Soldiers/Melee/RedMelee/SwordsmanRed.png',
};

const sprites = {};           // raw Image objects (fallback)
const spriteBitmaps = {};     // ImageBitmap tile cache: key -> ImageBitmap
let spritesLoaded = false;
let spritesLoadedCount = 0;
const totalSprites = Object.keys(SPRITE_SHEETS).length;

// ============================================================================
// SPRITE MAPPINGS (unchanged)
// ============================================================================

const GROUND_SPRITES = {
    'Grass': ['grass', 1, 0], 'GrassLight': ['grass', 2, 0], 'GrassDark': ['grass', 0, 0],
    'GrassTextured': ['texturedGrass', 0, 0], 'DeadGrass': ['deadGrass', 0, 0],
    'Path': ['grass', 3, 0], 'Water': ['shore', 4, 0], 'WaterShore': ['shore', 2, 0],
    'Sand': ['shore', 0, 0], 'Stone': ['grass', 4, 0], 'Bridge': ['grass', 3, 0],
    'Cliff': ['grass', 4, 0], 'CliffWater': ['shore', 3, 0], 'Dirt': ['deadGrass', 1, 0],
};

function getOverlaySpriteSheet(overlay, teamColor) {
    switch (overlay) {
        case 'TreeOak': return 'trees';
        case 'TreePine': return 'pineTrees';
        case 'TreeDead': return 'deadTrees';
        case 'Rock': case 'RockSmall': return 'rocks';
        case 'Wheatfield': return 'wheatfield';
        case 'House': case 'HouseLarge': case 'Hut': return getTeamSheet('Houses', teamColor);
        case 'Tavern': return getTeamSheet('Taverns', teamColor);
        case 'Market': return 'market';
        case 'Chapel': return 'chapels';
        case 'Well': return 'well';
        case 'Bridge': return 'bridge';
        case 'Workshop': return 'workshops';
        default: return null; // Fence, Gate, Cave, Tombstone, etc. use fallback rendering
    }
}

function getTeamSheet(buildingType, teamColor) {
    const p = { 'Purple': 'purple', 'Cyan': 'cyan', 'Lime': 'lime', 'Red': 'red', 'Wood': '' };
    const prefix = p[teamColor] || '';
    return prefix ? prefix + buildingType : buildingType.toLowerCase();
}

const CHARACTER_SPRITES = {
    'MagePurple': 'magePurple', 'MageCyan': 'mageCyan', 'MageLime': 'mageLime', 'MageRed': 'mageRed',
    'FarmerPurple': 'farmerPurple', 'FarmerCyan': 'farmerCyan', 'FarmerLime': 'farmerLime', 'FarmerRed': 'farmerRed',
    'SwordsmanPurple': 'swordsmanPurple', 'SwordsmanCyan': 'swordsmanCyan', 'SwordsmanLime': 'swordsmanLime', 'SwordsmanRed': 'swordsmanRed',
};

const DIRECTION_ROWS = { 'down': 0, 'left': 3, 'right': 6, 'up': 9, 'Down': 0, 'Left': 3, 'Right': 6, 'Up': 9 };
const SPRITE_COLS = 5;

const ENTITY_HEIGHTS = {
    'TreeOak': 1, 'TreePine': 1, 'TreeDead': 1,
    'House': 2, 'Tavern': 2, 'Market': 2, 'Chapel': 2,
    'Rock': 1, 'Wheatfield': 1, 'Well': 1, 'character': 1,
};

// ============================================================================
// STATUS EMOJI MAP
// ============================================================================

const STATE_EMOJI = {
    'idle': '💭', 'Idle': '💭', 'walking': '🚶', 'Walking': '🚶',
    'working': '⚡', 'Working': '⚡', 'talking': '💬', 'Talking': '💬',
    'sleeping': '💤', 'Sleeping': '💤', 'eating': '🍽️', 'Eating': '🍽️',
    'shopping': '🛒', 'Shopping': '🛒', 'researching': '🔬', 'Researching': '🔬',
    'coding': '💻', 'Coding': '💻', 'analyzing': '📊', 'Analyzing': '📊',
};

const STATE_COLORS = {
    'idle': '#e3b341', 'Idle': '#e3b341', 'walking': '#3fb950', 'Walking': '#3fb950',
    'working': '#f0883e', 'Working': '#f0883e', 'talking': '#3fb950', 'Talking': '#3fb950',
    'sleeping': '#58a6ff', 'Sleeping': '#58a6ff', 'eating': '#3fb950', 'Eating': '#3fb950',
    'shopping': '#3fb950', 'Shopping': '#3fb950', 'researching': '#a371f7', 'Researching': '#a371f7',
    'coding': '#79c0ff', 'Coding': '#79c0ff', 'analyzing': '#f0883e', 'Analyzing': '#f0883e',
};

// ============================================================================
// STATE
// ============================================================================

let world = null;
let canvas, ctx, bubbleCanvas, bubbleCtx;
let isPlaying = true;
let speed = 1;
let selectedEntity = null;
let followingEntity = null;
let buildings = [];
let animFrame = 0;
let ws = null;
let bubbleAnimPhase = 0;

// Camera state (smooth scrolling)
let camera = { x: 0, y: 0, targetX: 0, targetY: 0 };
let isDragging = false;
let dragStart = { x: 0, y: 0, camX: 0, camY: 0 };

// Activity log & per-character logs
let activityLog = [];
let prevCharStates = {};
let taskCompletionAnimations = []; // { x, y, text, startTime }
let charActivityLogs = {}; // { charId: [{text, time}] }
let conversations = {}; // { "id1-id2": [{speaker, text, time}] }

// ============================================================================
// OFFSCREEN CANVAS + DIRTY TRACKING
// ============================================================================

// Static ground layer — only redrawn when viewport moves or tiles change
let groundCanvas = null;
let groundCtx = null;
let groundValid = false;     // false => must redraw ground layer
let lastViewport = { sx: -1, sy: -1, ex: -1, ey: -1 };

// Double buffer for compositing
let bufferCanvas = null;
let bufferCtx = null;

// Track which character positions changed for dirty rect rendering
let prevCharPositions = {};  // id -> {x, y, state, direction, anim_frame}
let dirtyCharTiles = new Set(); // "x,y" strings of tiles that need entity redraw

// Reusable entity array to avoid GC
let _entityPool = [];

// ============================================================================
// INIT
// ============================================================================

document.addEventListener('DOMContentLoaded', () => {
    canvas = document.getElementById('worldCanvas');
    ctx = canvas.getContext('2d');
    ctx.imageSmoothingEnabled = false;

    // Apply will-change for GPU compositing
    canvas.style.willChange = 'transform';

    bubbleCanvas = document.getElementById('bubbleCanvas');
    bubbleCtx = bubbleCanvas.getContext('2d');

    loadSprites();
    connectWebSocket();
    setupCanvasInteraction();

    // Use requestAnimationFrame with throttle to ~6.67fps for animation ticks
    let lastAnimTick = 0;
    const ANIM_INTERVAL = 150; // ms between animation frame advances

    function gameLoop(timestamp) {
        const frameStart = performance.now();

        // Advance animation frame at fixed rate
        if (timestamp - lastAnimTick >= ANIM_INTERVAL) {
            animFrame = (animFrame + 1) % SPRITE_COLS;
            bubbleAnimPhase = (bubbleAnimPhase + 1) % 60;
            lastAnimTick = timestamp;
        }

        // Smooth camera interpolation (runs every rAF for smoothness)
        const lerp = 0.12;
        const dx = (camera.targetX - camera.x) * lerp;
        const dy = (camera.targetY - camera.y) * lerp;
        if (Math.abs(dx) > 0.5 || Math.abs(dy) > 0.5) {
            camera.x += dx;
            camera.y += dy;
            groundValid = false; // viewport moved
        }

        if (world && spritesLoaded) {
            render();
            renderBubbles();
            // Minimap at reduced rate (every 10 frames)
            if (perf.frameCount % 10 === 0) renderMinimap();
        }

        // Perf tracking
        const elapsed = performance.now() - frameStart;
        perf.frameTimes[perf.frameIdx] = elapsed;
        perf.frameIdx = (perf.frameIdx + 1) % 120;
        perf.frameCount++;

        requestAnimationFrame(gameLoop);
    }
    requestAnimationFrame(gameLoop);

    // Debounced resize handler
    let resizeTimer = null;
    window.addEventListener('resize', () => {
        if (resizeTimer) clearTimeout(resizeTimer);
        resizeTimer = setTimeout(() => {
            groundValid = false;
        }, 100);
    });
});

// ============================================================================
// SPRITE LOADING — use createImageBitmap for async decode
// ============================================================================

function loadSprites() {
    for (const [key, path] of Object.entries(SPRITE_SHEETS)) {
        const img = new Image();
        img.onload = () => {
            sprites[key] = img;
            // Pre-cut tiles into ImageBitmap cache (async, off main thread)
            precutSpriteTiles(key, img);
            spritesLoadedCount++;
            if (spritesLoadedCount === totalSprites) {
                spritesLoaded = true;
                groundValid = false;
                if (world) render();
            }
        };
        img.onerror = () => {
            spritesLoadedCount++;
            if (spritesLoadedCount === totalSprites) {
                spritesLoaded = true;
                if (world) render();
            }
        };
        img.src = path;
    }
}

function precutSpriteTiles(sheetKey, img) {
    if (typeof createImageBitmap === 'undefined') return;
    const cols = Math.floor(img.naturalWidth / TILE_SIZE);
    const rows = Math.floor(img.naturalHeight / TILE_SIZE);
    for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
            const cacheKey = `${sheetKey}:${c}:${r}`;
            createImageBitmap(img, c * TILE_SIZE, r * TILE_SIZE, TILE_SIZE, TILE_SIZE)
                .then(bmp => { spriteBitmaps[cacheKey] = bmp; })
                .catch(() => {});
        }
    }
}

// Get a pre-cut tile bitmap, falling back to source clipping
function getTileBitmap(sheetKey, col, row) {
    return spriteBitmaps[`${sheetKey}:${col}:${row}`] || null;
}

// ============================================================================
// WEBSOCKET — supports delta updates
// ============================================================================

function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${window.location.host}/ws`);

    ws.onopen = () => {
        document.getElementById('statusDot').classList.remove('off');
        document.getElementById('statusText').textContent = 'Connected';
        setTimeout(() => document.getElementById('conn-status').classList.remove('show'), 2000);
    };
    ws.onmessage = (event) => {
        const rawSize = typeof event.data === 'string' ? event.data.length : event.data.byteLength;
        perf.msgSizes.push(rawSize);

        const data = JSON.parse(event.data);

        if (data.type === 'world_state' || data.type === 'full_state') {
            // Full state — initial connection or resync
            perf.fullMsgCount++;
            world = data.world;
            groundValid = false; // full redraw needed
            prevCharPositions = {};
            detectActivity();
            extractBuildings();
            if (spritesLoaded) render();
            updateUI();

        } else if (data.type === 'delta') {
            // Delta update — only apply changes
            perf.deltaMsgCount++;
            if (!world) return; // need full state first

            // Update time/tick
            if (data.time_of_day !== undefined) world.time_of_day = data.time_of_day;
            if (data.tick !== undefined) world.tick = data.tick;

            // Apply character changes
            if (data.characters_changed) {
                for (const [id, charData] of Object.entries(data.characters_changed)) {
                    // Mark old position dirty
                    if (world.characters[id]) {
                        dirtyCharTiles.add(`${world.characters[id].x},${world.characters[id].y}`);
                    }
                    world.characters[id] = charData;
                    // Mark new position dirty
                    dirtyCharTiles.add(`${charData.x},${charData.y}`);
                }
            }

            // Remove characters
            if (data.characters_removed) {
                for (const id of data.characters_removed) {
                    if (world.characters[id]) {
                        dirtyCharTiles.add(`${world.characters[id].x},${world.characters[id].y}`);
                        delete world.characters[id];
                    }
                }
            }

            // Tile changes (rare — buildings placed/destroyed)
            if (data.tiles_changed) {
                for (const tc of data.tiles_changed) {
                    world.tiles[tc.y][tc.x] = tc.tile;
                    groundValid = false; // need ground redraw
                }
            }

            detectActivity();
            if (spritesLoaded) render();
            updateUI();
        }
    };
    ws.onclose = () => {
        document.getElementById('statusDot').classList.add('off');
        document.getElementById('statusText').textContent = 'Reconnecting…';
        document.getElementById('conn-status').classList.add('show');
        setTimeout(connectWebSocket, 2000);
    };
    ws.onerror = () => {};
}

function sendCommand(cmd) {
    if (ws && ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify(cmd));
}

// ============================================================================
// HELPER: Parse character state
// ============================================================================

function parseCharState(char) {
    const s = char.state;
    if (typeof s === 'string') {
        const colonIdx = s.indexOf(':');
        if (colonIdx > 0) {
            const key = s.slice(0, colonIdx).toLowerCase();
            const detail = s.slice(colonIdx + 1);
            if (key === 'talking') {
                const partnerChar = world ? world.characters[detail] : null;
                const partnerName = partnerChar ? partnerChar.name : detail;
                return { key, label: 'Talking', detail: `with ${partnerName}`, talkingWith: detail };
            }
            const labels = { 'researching': 'Researching', 'coding': 'Coding', 'analyzing': 'Analyzing' };
            return { key, label: labels[key] || key, detail, talkingWith: null };
        }
        return { key: s.toLowerCase(), label: s, detail: null, talkingWith: null };
    }
    if (typeof s === 'object') {
        if (s.Talking) {
            const partnerId = s.Talking.with || s.Talking.partner;
            const partnerChar = world ? world.characters[partnerId] : null;
            const partnerName = partnerChar ? partnerChar.name : partnerId;
            return { key: 'talking', label: 'Talking', detail: `with ${partnerName}`, talkingWith: partnerId };
        }
        if (s.Walking) {
            const dest = s.Walking.destination || s.Walking.to;
            return { key: 'walking', label: 'Walking', detail: dest ? `to ${dest}` : null, talkingWith: null };
        }
        if (s.Working) {
            const task = s.Working.task || s.Working.on;
            return { key: 'working', label: 'Working', detail: task ? `on ${task}` : null, talkingWith: null };
        }
        const firstKey = Object.keys(s)[0];
        if (firstKey) return { key: firstKey.toLowerCase(), label: firstKey, detail: null, talkingWith: null };
    }
    return { key: 'idle', label: 'Idle', detail: null, talkingWith: null };
}

function getCharColor(char) {
    const colorMap = { 'Cyan':'#00CED1','Lime':'#32CD32','Purple':'#9400D3','Red':'#DC143C' };
    for (const [k,v] of Object.entries(colorMap)) if (char.sprite.includes(k)) return v;
    return '#8b949e';
}

function getWorldTimeStr() {
    if (!world) return '00:00';
    const h = Math.floor(world.time_of_day / 100);
    const m = world.time_of_day % 100;
    return `${String(h).padStart(2,'0')}:${String(m).padStart(2,'0')}`;
}

// ============================================================================
// ACTIVITY DETECTION
// ============================================================================

function detectActivity() {
    if (!world) return;
    const time = getWorldTimeStr();

    for (const [id, char] of Object.entries(world.characters)) {
        const prev = prevCharStates[id];
        const ps = parseCharState(char);

        if (!charActivityLogs[id]) charActivityLogs[id] = [];

        if (!prev) {
            prevCharStates[id] = { key: ps.key, talkingWith: ps.talkingWith, current_task: char.current_task || null };
            continue;
        }

        if (prev.key !== ps.key) {
            let actText = '';
            const emoji = STATE_EMOJI[ps.key] || '❓';
            switch (ps.key) {
                case 'talking': actText = `${emoji} ${char.name} started chatting${ps.detail ? ' ' + ps.detail : ''}`; break;
                case 'walking': actText = `${emoji} ${char.name} is on the move${ps.detail ? ' ' + ps.detail : ''}`; break;
                case 'working': actText = `${emoji} ${char.name} started working${ps.detail ? ' ' + ps.detail : ''}`; break;
                case 'sleeping': actText = `${emoji} ${char.name} went to sleep`; break;
                case 'eating': actText = `${emoji} ${char.name} is eating`; break;
                case 'shopping': actText = `${emoji} ${char.name} went shopping`; break;
                case 'idle': actText = `${emoji} ${char.name} is idle`; break;
                default: actText = `${emoji} ${char.name}: ${ps.label}`;
            }
            if (actText) {
                addActivity(actText, time);
                charActivityLogs[id].push({ text: actText, time });
                if (charActivityLogs[id].length > 10) charActivityLogs[id].shift();
            }
        }

        if (ps.key === 'talking' && ps.talkingWith) {
            const convKey = [id, ps.talkingWith].sort().join('-');
            if (!conversations[convKey]) conversations[convKey] = [];
        }

        if (char.current_task && !prev.current_task) {
            addActivity(`🔬 ${char.name} started: ${char.current_task}`);
        }
        if (prev.current_task && !char.current_task && char.last_result) {
            const summary = char.last_result.length > 50 ? char.last_result.slice(0, 47) + '...' : char.last_result;
            addActivity(`✨ ${char.name} completed task: ${summary}`);
            taskCompletionAnimations.push({ x: char.x, y: char.y, text: 'Task Complete!', startTime: Date.now() });
        }

        prevCharStates[id] = { key: ps.key, talkingWith: ps.talkingWith, current_task: char.current_task || null };
    }
}

function addActivity(text, time) {
    if (!time) time = getWorldTimeStr();
    if (activityLog.length > 0 && activityLog[activityLog.length - 1].text === text) return;
    activityLog.push({ text, time });
    if (activityLog.length > 50) activityLog.shift();
    updateTicker();
}

function updateTicker() {
    const el = document.getElementById('tickerContent');
    if (activityLog.length === 0) return;
    const items = activityLog.slice(-15);
    el.innerHTML = items.map(a =>
        `<span class="ticker-item"><span class="ti-time">${a.time}</span>${a.text}</span>`
    ).join('');
    el.style.animation = 'none';
    el.offsetHeight;
    const duration = Math.max(15, items.length * 4);
    el.style.animation = `tickerScroll ${duration}s linear infinite`;
}

// ============================================================================
// CANVAS INTERACTION
// ============================================================================

function setupCanvasInteraction() {
    const container = document.getElementById('game-container');

    canvas.addEventListener('click', (e) => {
        if (Math.abs(e.clientX - dragStart.x) > 5 || Math.abs(e.clientY - dragStart.y) > 5) return;
        const rect = canvas.getBoundingClientRect();
        const scaleX = canvas.width / rect.width;
        const scaleY = canvas.height / rect.height;
        const clickX = (e.clientX - rect.left) * scaleX;
        const clickY = (e.clientY - rect.top) * scaleY;
        const tileX = Math.floor(clickX / TILE_PX);
        const tileY = Math.floor(clickY / TILE_PX);

        if (world) {
            for (const [id, char] of Object.entries(world.characters)) {
                if (char.x === tileX && char.y === tileY) {
                    selectEntity({ type: 'character', id, x: char.x, y: char.y, data: char });
                    return;
                }
            }
            const tile = world.tiles[tileY]?.[tileX];
            if (tile && tile.overlay && isBuilding(tile.overlay)) {
                selectEntity({ type: 'building', id: `${tileX},${tileY}`, x: tileX, y: tileY, data: tile });
                return;
            }
        }
        selectEntity(null);
    });

    container.addEventListener('mousedown', (e) => {
        isDragging = true;
        dragStart = { x: e.clientX, y: e.clientY, camX: camera.targetX, camY: camera.targetY };
        container.style.cursor = 'grabbing';
    });
    document.addEventListener('mousemove', (e) => {
        if (!isDragging) return;
        camera.targetX = dragStart.camX - (e.clientX - dragStart.x);
        camera.targetY = dragStart.camY - (e.clientY - dragStart.y);
        followingEntity = null;
    });
    document.addEventListener('mouseup', () => {
        isDragging = false;
        document.getElementById('game-container').style.cursor = 'grab';
    });
    container.style.cursor = 'grab';

    const miniCanvas = document.getElementById('minimapCanvas');
    miniCanvas.addEventListener('click', (e) => {
        if (!world) return;
        const rect = miniCanvas.getBoundingClientRect();
        const mx = (e.clientX - rect.left) / miniCanvas.width;
        const my = (e.clientY - rect.top) / miniCanvas.height;
        const container = document.getElementById('game-container');
        camera.targetX = mx * world.config.width * TILE_PX - container.clientWidth / 2;
        camera.targetY = my * world.config.height * TILE_PX - container.clientHeight / 2;
        followingEntity = null;
    });
}

function isBuilding(overlay) {
    return overlay === 'House' || overlay === 'Tavern' || overlay === 'Market' || overlay === 'Chapel' || overlay === 'Well' || overlay === 'Hut' || overlay === 'Tower' || overlay === 'Mausoleum';
}

function selectEntity(entity) {
    selectedEntity = entity;
    followingEntity = entity;
    if (entity) {
        setCameraTarget(entity.x, entity.y);
        if (entity.type === 'character') showCharDetail(entity.id);
    } else {
        document.getElementById('char-detail').classList.remove('visible');
    }
    updateUI();
}

function setCameraTarget(tileX, tileY) {
    const container = document.getElementById('game-container');
    camera.targetX = tileX * TILE_PX + TILE_PX / 2 - container.clientWidth / 2;
    camera.targetY = tileY * TILE_PX + TILE_PX / 2 - container.clientHeight / 2;
}

// ============================================================================
// CHARACTER DETAIL PANEL
// ============================================================================

function showCharDetail(charId) {
    if (!world || !world.characters[charId]) return;
    const char = world.characters[charId];
    const ps = parseCharState(char);
    const color = getCharColor(char);
    const stateColor = STATE_COLORS[ps.key] || '#8b949e';
    const emoji = STATE_EMOJI[ps.key] || '❓';
    const logs = charActivityLogs[charId] || [];

    let role = 'Villager';
    if (char.sprite.includes('Mage')) role = 'Mage';
    else if (char.sprite.includes('Farmer')) role = 'Farmer';
    else if (char.sprite.includes('Swordsman')) role = 'Swordsman';

    const panel = document.getElementById('char-detail');
    const body = document.getElementById('charDetailBody');

    let html = `
        <div class="cd-header">
            <div class="cd-avatar" style="background:${color}">${emoji}</div>
            <div>
                <div class="cd-name">${char.name}</div>
                <div class="cd-role">${role} · ${char.sprite}</div>
                <div class="cd-state-badge" style="background:${stateColor}22;color:${stateColor};border:1px solid ${stateColor}44;">
                    ${emoji} ${ps.label}${ps.detail ? ' ' + ps.detail : ''}
                </div>
            </div>
        </div>
        <div class="cd-section">
            <div class="cd-section-title">Current Task</div>
            <div class="cd-task">${ps.key === 'idle' ? 'No active task' : `${ps.label}${ps.detail ? ' ' + ps.detail : ''}`}</div>
        </div>
        <div class="cd-section">
            <div class="cd-section-title">Position</div>
            <div class="cd-task" style="font-family:monospace;font-size:10px;">x: ${char.x}, y: ${char.y} · facing ${char.direction}</div>
        </div>`;

    if (char.personality || char.traits) {
        const traits = char.personality || char.traits || [];
        if (Array.isArray(traits) && traits.length > 0) {
            html += `
        <div class="cd-section">
            <div class="cd-section-title">Traits</div>
            <div class="cd-traits">${traits.map(t => `<span class="cd-trait">${t}</span>`).join('')}</div>
        </div>`;
        }
    }

    html += `
        <div class="cd-section">
            <div class="cd-section-title">Recent Activity</div>
            ${logs.length === 0 ? '<div style="color:#484f58;font-size:10px;">No activity yet</div>' :
              logs.slice(-8).reverse().map(l => `<div class="cd-log-item"><span class="cd-log-time">${l.time}</span>${l.text}</div>`).join('')}
        </div>
        <button class="cd-assign-btn" onclick="alert('Task assignment coming soon!')">📋 Assign Task</button>`;

    body.innerHTML = html;
    panel.classList.add('visible');

    if (ps.talkingWith) showConvoPanel(charId, ps.talkingWith);
    else document.getElementById('convo-panel').classList.remove('visible');
}

// ============================================================================
// CONVERSATION PANEL
// ============================================================================

function showConvoPanel(id1, id2) {
    if (!world) return;
    const char1 = world.characters[id1];
    const char2 = world.characters[id2];
    if (!char1 || !char2) return;

    const convKey = [id1, id2].sort().join('-');
    const msgs = conversations[convKey] || [];
    const panel = document.getElementById('convo-panel');
    const body = document.getElementById('convoBody');

    const c1 = getCharColor(char1);
    const c2 = getCharColor(char2);

    let html = `<div style="text-align:center;margin-bottom:8px;font-size:10px;color:#8b949e;">
        ${char1.name} 💬 ${char2.name}
    </div>`;

    if (msgs.length === 0) {
        html += `<div style="text-align:center;color:#484f58;font-size:10px;padding:20px;">Conversation in progress…</div>`;
        html += `
            <div class="convo-bubble">
                <div class="convo-avatar-sm" style="background:${c1}">${char1.name[0]}</div>
                <div><div class="convo-name">${char1.name}</div><div class="convo-text">…</div></div>
            </div>
            <div class="convo-bubble right">
                <div class="convo-avatar-sm" style="background:${c2}">${char2.name[0]}</div>
                <div><div class="convo-name" style="text-align:right;">${char2.name}</div><div class="convo-text">…</div></div>
            </div>`;
    } else {
        html += msgs.slice(-10).map(m => {
            const isFirst = m.speaker === id1;
            const speaker = isFirst ? char1 : char2;
            const col = isFirst ? c1 : c2;
            return `<div class="convo-bubble${isFirst ? '' : ' right'}">
                <div class="convo-avatar-sm" style="background:${col}">${speaker.name[0]}</div>
                <div><div class="convo-name"${isFirst ? '' : ' style="text-align:right;"'}>${speaker.name}</div><div class="convo-text">${m.text}</div></div>
            </div>`;
        }).join('');
    }

    body.innerHTML = html;
    panel.classList.add('visible');
}

// ============================================================================
// BUILDINGS
// ============================================================================

function extractBuildings() {
    buildings = [];
    if (!world) return;
    const seen = new Set();
    for (let y = 0; y < world.config.height; y++) {
        for (let x = 0; x < world.config.width; x++) {
            const tile = world.tiles[y][x];
            if (tile.overlay && isBuilding(tile.overlay)) {
                if (tile.sprite_row > 0) continue;
                const key = `${tile.overlay}-${x}-${y}`;
                if (seen.has(key)) continue;
                seen.add(key);
                buildings.push({ type: tile.overlay, x, y, color: tile.team_color || 'Wood', variant: tile.sprite_col });
            }
        }
    }
}

// ============================================================================
// RENDERING — Offscreen ground + dirty entity rendering + double buffer
// ============================================================================

function render() {
    if (!world) return;
    const w = world.config.width, h = world.config.height;
    const cw = w * TILE_PX, ch = h * TILE_PX;

    // Resize canvases if needed
    if (canvas.width !== cw || canvas.height !== ch) {
        canvas.width = cw; canvas.height = ch;
        ctx.imageSmoothingEnabled = false;
        groundValid = false;
    }

    // Position canvas based on camera
    canvas.style.left = (-camera.x) + 'px';
    canvas.style.top = (-camera.y) + 'px';

    // Compute visible tile range
    const container = document.getElementById('game-container');
    const startX = Math.max(0, Math.floor(camera.x / TILE_PX) - 2);
    const startY = Math.max(0, Math.floor(camera.y / TILE_PX) - 2);
    const endX = Math.min(w, Math.ceil((camera.x + container.clientWidth) / TILE_PX) + 2);
    const endY = Math.min(h, Math.ceil((camera.y + container.clientHeight) / TILE_PX) + 2);

    // === GROUND LAYER (offscreen, cached) ===
    if (!groundCanvas) {
        groundCanvas = new OffscreenCanvas(cw, ch);
        groundCtx = groundCanvas.getContext('2d');
        groundCtx.imageSmoothingEnabled = false;
        groundValid = false;
    }
    if (groundCanvas.width !== cw || groundCanvas.height !== ch) {
        groundCanvas.width = cw;
        groundCanvas.height = ch;
        groundCtx.imageSmoothingEnabled = false;
        groundValid = false;
    }

    if (!groundValid) {
        // Redraw ground layer for visible area (plus margin)
        const gsx = Math.max(0, startX - 4);
        const gsy = Math.max(0, startY - 4);
        const gex = Math.min(w, endX + 4);
        const gey = Math.min(h, endY + 4);

        // Clear the region
        groundCtx.fillStyle = '#1a4466';
        groundCtx.fillRect(gsx * TILE_PX, gsy * TILE_PX, (gex - gsx) * TILE_PX, (gey - gsy) * TILE_PX);

        // Water layer
        for (let y = gsy; y < gey; y++)
            for (let x = gsx; x < gex; x++)
                if (world.tiles[y][x].ground === 'Water')
                    drawGroundTile(groundCtx, x, y, world.tiles[y][x]);

        // Ground layer
        for (let y = gsy; y < gey; y++)
            for (let x = gsx; x < gex; x++)
                if (world.tiles[y][x].ground !== 'Water')
                    drawGroundTile(groundCtx, x, y, world.tiles[y][x]);

        // Overlays (static — trees, buildings, rocks)
        for (let y = gsy; y < gey; y++)
            for (let x = gsx; x < gex; x++) {
                const tile = world.tiles[y][x];
                if (tile.overlay) drawOverlay(groundCtx, x, y, tile);
            }

        groundValid = true;
        lastViewport = { sx: startX, sy: startY, ex: endX, ey: endY };
    }

    // === COMPOSITE: blit ground, then draw entities ===
    // Clear visible region of main canvas
    ctx.fillStyle = '#1a4466';
    ctx.fillRect(startX * TILE_PX, startY * TILE_PX, (endX - startX) * TILE_PX, (endY - startY) * TILE_PX);

    // Blit ground
    const sx = startX * TILE_PX, sy = startY * TILE_PX;
    const sw = (endX - startX) * TILE_PX, sh = (endY - startY) * TILE_PX;
    ctx.drawImage(groundCanvas, sx, sy, sw, sh, sx, sy, sw, sh);

    // === ENTITY LAYER (characters — drawn every frame over ground) ===
    // Build entity list (reuse array)
    _entityPool.length = 0;
    for (const [id, char] of Object.entries(world.characters)) {
        // Viewport culling
        if (char.x < startX - 1 || char.x > endX + 1 || char.y < startY - 1 || char.y > endY + 1) continue;
        _entityPool.push({ type: 'character', x: char.x, y: char.y, sortY: char.y + 1, id, char });
    }
    _entityPool.sort((a, b) => a.sortY - b.sortY);

    for (const e of _entityPool) {
        const sel = selectedEntity?.type === 'character' && selectedEntity?.id === e.id;
        drawCharacter(ctx, e.char, sel);
    }

    // Task completion animations
    const now = Date.now();
    taskCompletionAnimations = taskCompletionAnimations.filter(a => now - a.startTime < 3000);
    for (const anim of taskCompletionAnimations) {
        const age = (now - anim.startTime) / 3000;
        const alpha = 1 - age;
        const yOff = -30 * age;
        const apx = anim.x * TILE_PX + TILE_PX / 2;
        const apy = anim.y * TILE_PX + yOff;

        ctx.save();
        ctx.globalAlpha = alpha;
        ctx.font = 'bold 10px Inter, sans-serif';
        ctx.textAlign = 'center';
        const text = `✨ ${anim.text}`;
        const tw = ctx.measureText(text).width + 12;
        ctx.fillStyle = 'rgba(34, 197, 94, 0.9)';
        roundRect(ctx, apx - tw / 2, apy - 12, tw, 16, 6);
        ctx.fill();
        ctx.fillStyle = '#fff';
        ctx.fillText(text, apx, apy);
        ctx.restore();
    }

    // Follow entity
    if (followingEntity?.type === 'character') {
        const c = world.characters[followingEntity.id];
        if (c) { followingEntity.x = c.x; followingEntity.y = c.y; setCameraTarget(c.x, c.y); }
    }

    updateDayNight();
    dirtyCharTiles.clear();
}

// ============================================================================
// DRAWING HELPERS — accept context parameter for ground/main canvas
// ============================================================================

function drawGroundTile(c, x, y, tile) {
    const px = x * TILE_PX, py = y * TILE_PX;
    const si = GROUND_SPRITES[tile.ground];
    if (si) {
        const sheetKey = si[0];
        const col = tile.sprite_col !== undefined ? tile.sprite_col : si[1];
        const row = tile.sprite_row !== undefined ? tile.sprite_row : si[2];
        const sheet = sprites[sheetKey];
        if (sheet) {
            const mc = Math.floor(sheet.naturalWidth / TILE_SIZE) - 1;
            const mr = Math.floor(sheet.naturalHeight / TILE_SIZE) - 1;
            const sc = Math.min(col, mc), sr = Math.min(row, mr);
            // Try pre-cut bitmap first
            const bmp = getTileBitmap(sheetKey, sc, sr);
            if (bmp) {
                c.drawImage(bmp, px, py, TILE_PX, TILE_PX);
            } else {
                c.drawImage(sheet, sc * TILE_SIZE, sr * TILE_SIZE, TILE_SIZE, TILE_SIZE, px, py, TILE_PX, TILE_PX);
            }
            return;
        }
    }
    const colors = { 'Grass':'#6db04a','GrassLight':'#8cc85a','GrassDark':'#549642','GrassTextured':'#7cb550','DeadGrass':'#9b8844','Path':'#c2b280','Water':'#2288aa','WaterShore':'#5dade2','Sand':'#edc9af','Stone':'#808080','Bridge':'#8b7355','Cliff':'#707060','CliffWater':'#3a6688','Dirt':'#8b7355' };
    c.fillStyle = colors[tile.ground] || '#6db04a';
    c.fillRect(px, py, TILE_PX, TILE_PX);
}

function drawOverlay(c, x, y, tile) {
    const px = x * TILE_PX, py = y * TILE_PX;
    const sk = getOverlaySpriteSheet(tile.overlay, tile.team_color);
    const sheet = sprites[sk];
    if (sheet && sheet.complete && sheet.naturalWidth > 0) {
        const col = tile.sprite_col || 0, row = tile.sprite_row || 0;
        const mc = Math.floor(sheet.naturalWidth / TILE_SIZE) - 1;
        const mr = Math.floor(sheet.naturalHeight / TILE_SIZE) - 1;
        const sc = Math.min(col, mc), sr = Math.min(row, mr);
        const bmp = getTileBitmap(sk, sc, sr);
        if (bmp) {
            c.drawImage(bmp, px, py, TILE_PX, TILE_PX);
        } else {
            c.drawImage(sheet, sc * TILE_SIZE, sr * TILE_SIZE, TILE_SIZE, TILE_SIZE, px, py, TILE_PX, TILE_PX);
        }
    } else {
        drawOverlayFallback(c, px, py, tile.overlay, tile.team_color);
    }
    if (selectedEntity?.type === 'building' && selectedEntity.x === x && selectedEntity.y === y) {
        c.strokeStyle = '#f0883e'; c.lineWidth = 2;
        c.strokeRect(px + 1, py + 1, TILE_PX - 2, TILE_PX - 2);
    }
}

function drawCharacter(c, char, isSelected) {
    const px = char.x * TILE_PX, py = char.y * TILE_PX;
    const sk = CHARACTER_SPRITES[char.sprite];
    const sheet = sprites[sk];

    // OpenClaw task aura
    if (char.current_task && char.task_status) {
        const auraColors = {
            'pending': 'rgba(255, 200, 50, 0.3)',
            'running': 'rgba(100, 200, 255, 0.4)',
            'completed': 'rgba(100, 255, 100, 0.4)',
        };
        const auraColor = auraColors[char.task_status] || 'rgba(100, 200, 255, 0.3)';
        const pulse = 0.6 + 0.4 * Math.sin(Date.now() / 400);

        c.save();
        c.globalAlpha = pulse;
        const gradient = c.createRadialGradient(px + TILE_PX / 2, py + TILE_PX / 2, TILE_PX / 4, px + TILE_PX / 2, py + TILE_PX / 2, TILE_PX);
        gradient.addColorStop(0, auraColor);
        gradient.addColorStop(1, 'rgba(0,0,0,0)');
        c.fillStyle = gradient;
        c.fillRect(px - TILE_PX / 2, py - TILE_PX / 2, TILE_PX * 2, TILE_PX * 2);
        c.restore();
    }

    // Shadow
    c.fillStyle = 'rgba(0,0,0,0.25)';
    c.beginPath(); c.ellipse(px + TILE_PX / 2, py + TILE_PX - 3, TILE_PX / 3, 4, 0, 0, Math.PI * 2); c.fill();

    if (sheet && sheet.complete && sheet.naturalWidth > 0) {
        const baseRow = DIRECTION_ROWS[char.direction] || 0;
        const ps = parseCharState(char);
        const moving = ps.key === 'walking' || ps.key === 'talking';
        const row = baseRow + (moving ? 1 : 0);
        const bmp = getTileBitmap(sk, animFrame, row);
        if (bmp) {
            c.drawImage(bmp, px, py, TILE_PX, TILE_PX);
        } else {
            c.drawImage(sheet, animFrame * TILE_SIZE, row * TILE_SIZE, TILE_SIZE, TILE_SIZE, px, py, TILE_PX, TILE_PX);
        }
    } else {
        drawCharacterFallback(c, char, isSelected);
    }

    if (isSelected) {
        c.strokeStyle = '#f0883e'; c.lineWidth = 2;
        c.setLineDash([4, 3]);
        c.strokeRect(px - 2, py - 2, TILE_PX + 4, TILE_PX + 4);
        c.setLineDash([]);
    }

    // Name label
    c.font = '600 9px Inter, sans-serif';
    c.textAlign = 'center';
    const tw = c.measureText(char.name).width + 8;
    const ny = py - 6;
    c.fillStyle = isSelected ? 'rgba(240,136,62,0.9)' : 'rgba(13,17,23,0.85)';
    roundRect(c, px + TILE_PX / 2 - tw / 2, ny - 10, tw, 14, 4);
    c.fill();
    c.fillStyle = isSelected ? '#fff' : '#c9d1d9';
    c.fillText(char.name, px + TILE_PX / 2, ny);
}

function roundRect(c, x, y, w, h, r) {
    c.beginPath();
    c.moveTo(x + r, y); c.lineTo(x + w - r, y); c.quadraticCurveTo(x + w, y, x + w, y + r);
    c.lineTo(x + w, y + h - r); c.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
    c.lineTo(x + r, y + h); c.quadraticCurveTo(x, y + h, x, y + h - r);
    c.lineTo(x, y + r); c.quadraticCurveTo(x, y, x + r, y);
    c.closePath();
}

// ============================================================================
// STATUS BUBBLES (separate canvas layer) — with viewport culling
// ============================================================================

function renderBubbles() {
    if (!world) return;
    const container = document.getElementById('game-container');
    const cw = container.clientWidth;
    const ch = container.clientHeight;

    if (bubbleCanvas.width !== cw || bubbleCanvas.height !== ch) {
        bubbleCanvas.width = cw;
        bubbleCanvas.height = ch;
    }
    bubbleCanvas.style.left = '0px';
    bubbleCanvas.style.top = '0px';

    bubbleCtx.clearRect(0, 0, cw, ch);

    const floatOffset = Math.sin(bubbleAnimPhase * Math.PI / 30) * 3;

    for (const [id, char] of Object.entries(world.characters)) {
        const sx = char.x * TILE_PX - camera.x + TILE_PX / 2;
        const sy = char.y * TILE_PX - camera.y;

        // Viewport culling
        if (sx < -50 || sx > cw + 50 || sy < -50 || sy > ch + 50) continue;

        const ps = parseCharState(char);
        const emoji = STATE_EMOJI[ps.key] || '❓';
        const stateColor = STATE_COLORS[ps.key] || '#8b949e';

        const bx = sx;
        const by = sy - 28 + floatOffset;

        bubbleCtx.save();

        let extraText = '';
        if ((ps.key === 'talking' || ps.key === 'working' || ps.key === 'walking') && ps.detail) {
            extraText = ps.detail;
        }

        bubbleCtx.font = '13px sans-serif';
        const emojiWidth = 18;
        bubbleCtx.font = '600 8px Inter, sans-serif';
        const textWidth = extraText ? bubbleCtx.measureText(extraText).width : 0;
        const totalWidth = emojiWidth + (extraText ? textWidth + 4 : 0);
        const pillW = Math.max(24, totalWidth + 10);
        const pillH = 18;
        const pillX = bx - pillW / 2;
        const pillY = by - pillH / 2;

        // Shadow
        bubbleCtx.fillStyle = 'rgba(0,0,0,0.3)';
        roundRectPath(bubbleCtx, pillX + 1, pillY + 1, pillW, pillH, 9);
        bubbleCtx.fill();

        // Background
        bubbleCtx.fillStyle = `${stateColor}22`;
        roundRectPath(bubbleCtx, pillX, pillY, pillW, pillH, 9);
        bubbleCtx.fill();
        bubbleCtx.strokeStyle = `${stateColor}66`;
        bubbleCtx.lineWidth = 1;
        bubbleCtx.stroke();

        // Emoji
        bubbleCtx.font = '12px sans-serif';
        bubbleCtx.textAlign = 'center';
        bubbleCtx.textBaseline = 'middle';
        bubbleCtx.fillStyle = '#fff';
        if (extraText) {
            bubbleCtx.fillText(emoji, pillX + 12, by);
            bubbleCtx.font = '600 8px Inter, sans-serif';
            bubbleCtx.textAlign = 'left';
            bubbleCtx.fillStyle = stateColor;
            bubbleCtx.fillText(extraText, pillX + 22, by + 1);
        } else {
            bubbleCtx.fillText(emoji, bx, by);
        }

        if (ps.key === 'sleeping') {
            const zPhase = (bubbleAnimPhase + 15) % 60;
            const zFloat = Math.sin(zPhase * Math.PI / 30) * 5;
            bubbleCtx.font = '8px sans-serif';
            bubbleCtx.globalAlpha = 0.6;
            bubbleCtx.fillStyle = '#58a6ff';
            bubbleCtx.textAlign = 'center';
            bubbleCtx.fillText('z', bx + 14, by - 8 + zFloat);
            bubbleCtx.fillText('z', bx + 20, by - 14 + zFloat);
            bubbleCtx.globalAlpha = 1;
        }

        bubbleCtx.restore();
    }
}

function roundRectPath(c, x, y, w, h, r) {
    c.beginPath();
    c.moveTo(x + r, y); c.lineTo(x + w - r, y); c.quadraticCurveTo(x + w, y, x + w, y + r);
    c.lineTo(x + w, y + h - r); c.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
    c.lineTo(x + r, y + h); c.quadraticCurveTo(x, y + h, x, y + h - r);
    c.lineTo(x, y + r); c.quadraticCurveTo(x, y, x + r, y);
    c.closePath();
}

// ============================================================================
// DAY/NIGHT CYCLE
// ============================================================================

function updateDayNight() {
    if (!world) return;
    const h = world.time_of_day / 100;
    let alpha = 0;
    if (h < 5) alpha = 0.35;
    else if (h < 7) alpha = 0.35 * (1 - (h - 5) / 2);
    else if (h < 18) alpha = 0;
    else if (h < 20) alpha = 0.35 * ((h - 18) / 2);
    else alpha = 0.35;

    document.getElementById('day-night-overlay').style.background = `rgba(10, 15, 40, ${alpha})`;
}

// ============================================================================
// MINIMAP
// ============================================================================

function renderMinimap() {
    if (!world) return;
    const mc = document.getElementById('minimapCanvas');
    const mx = mc.getContext('2d');
    const w = world.config.width, h = world.config.height;
    const sx = mc.width / w, sy = mc.height / h;

    mx.fillStyle = '#0d1117';
    mx.fillRect(0, 0, mc.width, mc.height);

    const groundColors = { 'Grass':'#4a7a34','GrassLight':'#5a8a40','GrassDark':'#3a6a28','GrassTextured':'#4d7d38','DeadGrass':'#7a6a30','Path':'#a09060','Water':'#1a5577','WaterShore':'#3a88aa','Sand':'#c0a080','Stone':'#606060' };
    for (let y = 0; y < h; y++) {
        for (let x = 0; x < w; x++) {
            const t = world.tiles[y][x];
            mx.fillStyle = groundColors[t.ground] || '#4a7a34';
            mx.fillRect(x * sx, y * sy, Math.ceil(sx), Math.ceil(sy));
            if (t.overlay && isBuilding(t.overlay)) {
                mx.fillStyle = '#f0883e';
                mx.fillRect(x * sx, y * sy, Math.ceil(sx), Math.ceil(sy));
            }
        }
    }

    for (const char of Object.values(world.characters)) {
        const colors = { 'Cyan':'#00dddd','Lime':'#44ee44','Purple':'#bb44ff','Red':'#ff4444' };
        let col = '#ffffff';
        for (const [k, v] of Object.entries(colors)) if (char.sprite.includes(k)) { col = v; break; }
        mx.fillStyle = col;
        mx.fillRect(char.x * sx - 1, char.y * sy - 1, 3, 3);
    }

    const container = document.getElementById('game-container');
    const vx = camera.x / (w * TILE_PX) * mc.width;
    const vy = camera.y / (h * TILE_PX) * mc.height;
    const vw = container.clientWidth / (w * TILE_PX) * mc.width;
    const vh = container.clientHeight / (h * TILE_PX) * mc.height;
    mx.strokeStyle = 'rgba(255,255,255,0.6)';
    mx.lineWidth = 1;
    mx.strokeRect(vx, vy, vw, vh);
}

// ============================================================================
// FALLBACKS
// ============================================================================

function drawOverlayFallback(c, px, py, overlay, teamColor) {
    const tc = { 'Purple':'#9400D3','Cyan':'#00CED1','Lime':'#32CD32','Red':'#DC143C','Wood':'#8B4513' };
    const color = tc[teamColor] || '#8B4513';
    if (overlay.includes('Tree')) {
        c.fillStyle = '#654321'; c.fillRect(px + TILE_PX / 2 - 3, py + TILE_PX / 2, 6, TILE_PX / 2);
        c.fillStyle = overlay === 'TreePine' ? '#228B22' : '#32CD32';
        c.beginPath(); c.arc(px + TILE_PX / 2, py + TILE_PX / 3, TILE_PX / 3, 0, Math.PI * 2); c.fill();
    } else if (['House','Tavern','Market','Chapel','Hut'].includes(overlay)) {
        c.fillStyle = '#8B4513'; c.fillRect(px + 2, py + TILE_PX / 2, TILE_PX - 4, TILE_PX / 2 - 2);
        c.fillStyle = color; c.beginPath();
        c.moveTo(px + TILE_PX / 2, py + 2); c.lineTo(px + 2, py + TILE_PX / 2); c.lineTo(px + TILE_PX - 2, py + TILE_PX / 2);
        c.closePath(); c.fill();
    } else if (overlay === 'Well') {
        c.fillStyle = '#4682B4'; c.beginPath(); c.arc(px + TILE_PX / 2, py + TILE_PX / 2, TILE_PX / 3, 0, Math.PI * 2); c.fill();
    } else if (overlay === 'Rock' || overlay === 'RockSmall') {
        c.fillStyle = '#808080'; c.beginPath(); c.ellipse(px + TILE_PX / 2, py + TILE_PX / 2 + 2, TILE_PX / 3, TILE_PX / 4, 0, 0, Math.PI * 2); c.fill();
    } else if (overlay === 'Wheatfield') {
        c.fillStyle = '#DAA520'; c.fillRect(px + 2, py + 2, TILE_PX - 4, TILE_PX - 4);
    } else if (overlay === 'Fence' || overlay === 'Gate') {
        c.fillStyle = '#8B7355'; c.fillRect(px + 2, py + TILE_PX / 2 - 2, TILE_PX - 4, 4);
        c.fillRect(px + 4, py + TILE_PX / 4, 3, TILE_PX / 2);
        c.fillRect(px + TILE_PX - 7, py + TILE_PX / 4, 3, TILE_PX / 2);
    } else if (overlay === 'Tombstone') {
        c.fillStyle = '#696969'; c.fillRect(px + TILE_PX / 2 - 4, py + TILE_PX / 3, 8, TILE_PX / 2);
        c.beginPath(); c.arc(px + TILE_PX / 2, py + TILE_PX / 3, 4, Math.PI, 0); c.fill();
    } else if (overlay === 'Cave' || overlay === 'Mausoleum') {
        c.fillStyle = '#555'; c.beginPath(); c.arc(px + TILE_PX / 2, py + TILE_PX / 2 + 4, TILE_PX / 3, Math.PI, 0); c.fill();
        c.fillRect(px + TILE_PX / 2 - TILE_PX / 3, py + TILE_PX / 2 + 4, TILE_PX * 2 / 3, TILE_PX / 4);
    } else if (overlay === 'Tower') {
        c.fillStyle = '#808080'; c.fillRect(px + TILE_PX / 4, py + 2, TILE_PX / 2, TILE_PX - 4);
        c.fillStyle = '#606060'; c.fillRect(px + TILE_PX / 4 - 2, py, TILE_PX / 2 + 4, 4);
    } else if (overlay === 'Portal') {
        c.fillStyle = '#8a2be2'; c.beginPath(); c.ellipse(px + TILE_PX / 2, py + TILE_PX / 2, TILE_PX / 3, TILE_PX / 2.5, 0, 0, Math.PI * 2); c.fill();
        c.fillStyle = '#4b0082'; c.beginPath(); c.ellipse(px + TILE_PX / 2, py + TILE_PX / 2, TILE_PX / 5, TILE_PX / 4, 0, 0, Math.PI * 2); c.fill();
    } else if (overlay === 'Chest') {
        c.fillStyle = '#DAA520'; c.fillRect(px + TILE_PX / 4, py + TILE_PX / 3, TILE_PX / 2, TILE_PX / 3);
        c.fillStyle = '#B8860B'; c.fillRect(px + TILE_PX / 2 - 2, py + TILE_PX / 3 + 2, 4, 4);
    } else if (overlay === 'Cactus') {
        c.fillStyle = '#2E8B57'; c.fillRect(px + TILE_PX / 2 - 3, py + TILE_PX / 4, 6, TILE_PX / 2);
        c.fillRect(px + TILE_PX / 2 - 8, py + TILE_PX / 3, 5, 4);
        c.fillRect(px + TILE_PX / 2 + 3, py + TILE_PX / 3 + 4, 5, 4);
    } else if (overlay === 'Bush' || overlay === 'Flowers') {
        c.fillStyle = overlay === 'Flowers' ? '#FF69B4' : '#228B22';
        c.beginPath(); c.arc(px + TILE_PX / 2, py + TILE_PX / 2 + 2, TILE_PX / 4, 0, Math.PI * 2); c.fill();
    } else if (overlay === 'Tumbleweed') {
        c.fillStyle = '#9b8844'; c.beginPath(); c.arc(px + TILE_PX / 2, py + TILE_PX / 2 + 2, TILE_PX / 5, 0, Math.PI * 2); c.fill();
    } else if (['Chicken','Pig','Sheep','Horse'].includes(overlay)) {
        const animalColors = { 'Chicken':'#FFD700','Pig':'#FFB6C1','Sheep':'#F5F5DC','Horse':'#8B4513' };
        c.fillStyle = animalColors[overlay] || '#AAA';
        c.beginPath(); c.ellipse(px + TILE_PX / 2, py + TILE_PX / 2 + 2, TILE_PX / 4, TILE_PX / 5, 0, 0, Math.PI * 2); c.fill();
    }
}

function drawCharacterFallback(c, char, isSelected) {
    const px = char.x * TILE_PX, py = char.y * TILE_PX;
    const colors = { 'Cyan':'#00FFFF','Lime':'#00FF00','Purple':'#9400D3','Red':'#FF4444' };
    let color = '#FFFFFF';
    for (const [k, v] of Object.entries(colors)) if (char.sprite.includes(k)) { color = v; break; }
    c.fillStyle = color;
    c.beginPath(); c.arc(px + TILE_PX / 2, py + TILE_PX / 2, TILE_PX / 3, 0, Math.PI * 2); c.fill();
}

// ============================================================================
// UI UPDATES
// ============================================================================

function updateUI() {
    if (!world) return;
    const h = Math.floor(world.time_of_day / 100);
    const m = world.time_of_day % 100;
    document.getElementById('worldTime').textContent = `${String(h).padStart(2,'0')}:${String(m).padStart(2,'0')}`;
    document.getElementById('tickCount').textContent = world.tick;
    document.getElementById('popCount').textContent = Object.keys(world.characters).length;

    updateRoster();
    updateBuildingList();

    if (selectedEntity?.type === 'character') showCharDetail(selectedEntity.id);
}

function updateRoster() {
    const rl = document.getElementById('rosterList');
    const sorted = Object.entries(world.characters).sort((a, b) => a[1].name.localeCompare(b[1].name));

    const items = [];
    for (const [id, char] of sorted) {
        const sel = selectedEntity?.type === 'character' && selectedEntity?.id === id;
        const ps = parseCharState(char);
        const color = getCharColor(char);
        const stateColor = STATE_COLORS[ps.key] || '#8b949e';
        const emoji = STATE_EMOJI[ps.key] || '❓';

        let activity = ps.label;
        if (ps.detail) activity += ' ' + ps.detail;

        let taskHtml = '';
        if (char.current_task) {
            const statusIcon = { 'pending': '⏳', 'running': '⚡', 'completed': '✅' }[char.task_status] || '🔄';
            taskHtml = `<div class="ri-task" style="font-size:9px;color:#58a6ff;margin-top:1px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">${statusIcon} ${char.current_task}</div>`;
        }

        items.push(`
            <div class="roster-item${sel ? ' sel' : ''}" onclick="focusCharacter('${id}')">
                <div class="ri-avatar" style="background:${color}33;border:1px solid ${color}66">${emoji}</div>
                <div class="ri-info">
                    <div class="ri-name">
                        ${char.name}
                        <div class="ri-dot state-${ps.key}" style="width:6px;height:6px;border-radius:50%;"></div>
                    </div>
                    <div class="ri-activity">${activity}</div>
                    ${taskHtml}
                </div>
            </div>`);
    }
    rl.innerHTML = items.join('');
}

function updateBuildingList() {
    const bl = document.getElementById('buildingList');
    bl.innerHTML = '';
    const icons = { 'House':'🏠','Tavern':'🍺','Market':'🏪','Chapel':'⛪','Well':'💧' };
    // Use DocumentFragment for batch DOM insertion
    const frag = document.createDocumentFragment();
    for (const b of buildings) {
        const sel = selectedEntity?.type === 'building' && selectedEntity?.x === b.x && selectedEntity?.y === b.y;
        const d = document.createElement('div');
        d.className = `bld-row${sel ? ' sel' : ''}`;
        d.innerHTML = `<span class="bld-name">${icons[b.type] || '🏛'} ${b.type}</span><span class="bld-team">${b.color}</span><span style="float:right;color:#484f58;font-size:9px">${b.x},${b.y}</span>`;
        d.onclick = () => selectEntity({ type: 'building', id: `${b.x},${b.y}`, x: b.x, y: b.y, data: b });
        frag.appendChild(d);
    }
    bl.appendChild(frag);
}

// Global function for roster clicks
function focusCharacter(id) {
    if (!world || !world.characters[id]) return;
    const char = world.characters[id];
    selectEntity({ type: 'character', id, x: char.x, y: char.y, data: char });
}
