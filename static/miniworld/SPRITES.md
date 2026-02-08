# MiniWorld Sprites Reference

All sprites are 16×16 pixels per tile.

## Ground Tiles

### Grass.png (80×16 = 5 cols × 1 row)
| Col | Description |
|-----|-------------|
| 0 | Standard grass (green) |
| 1 | Light grass |
| 2 | Dark grass |
| 3 | Path/dirt (tan) |
| 4 | Stone (grey) |

### Shore.png (80×16 = 5 cols × 1 row)
| Col | Description |
|-----|-------------|
| 0 | Sand/beach |
| 1 | Shore transition light |
| 2 | Shore transition (WaterShore) |
| 3 | Shallow water |
| 4 | Deep water (Water) |

### TexturedGrass.png (varies)
- Multiple grass tiles with flowers/details
- Use for GrassTextured ground type

### DeadGrass.png
- Brown/dead grass tiles

---

## Nature Overlays

### Trees.png (64×16 = 4 cols × 1 row)
| Col | Description |
|-----|-------------|
| 0 | Green tree variant 1 |
| 1 | Green tree variant 2 |
| 2 | Bush/small tree |
| 3 | Flowers |

### PineTrees.png (64×16 = 4 cols × 1 row)
| Col | Description |
|-----|-------------|
| 0-3 | Pine tree variants |

### DeadTrees.png (64×16 = 4 cols × 1 row)
| Col | Description |
|-----|-------------|
| 0-3 | Dead/bare tree variants |

### Wheatfield.png (64×16 = 4 cols × 1 row)
| Col | Description |
|-----|-------------|
| 0 | Young wheat |
| 1 | Growing wheat |
| 2 | Mature wheat |
| 3 | Ready to harvest |

### Rocks.png
- Various rock formations

---

## Buildings (Multi-tile: 1 wide × 2 tall)

### Houses.png, Taverns.png, Market.png, Chapels.png, Workshops.png
- **Columns**: Different building variants (0, 1, 2, etc.)
- **Row 0**: Top part of building (roof)
- **Row 1**: Bottom part of building (entrance)

### Colored Buildings (Buildings/Purple/, Buildings/Cyan/, etc.)
- Same layout but team-colored

---

## Characters (Animation Grid)

### Layout: 5 cols × 12 rows (16×16 per frame, 80×192 total)

**Animation Columns (0-4):**
Each column is a frame in the animation cycle.

**Direction Rows (3 rows per direction):**
| Rows | Direction | Row 0 | Row 1 | Row 2 |
|------|-----------|-------|-------|-------|
| 0-2 | Down | Idle | Walk | Action |
| 3-5 | Left | Idle | Walk | Action |
| 6-8 | Right | Idle | Walk | Action |
| 9-11 | Up | Idle | Walk | Action |

### Character Types & Paths:
```
Workers/
  CyanWorker/FarmerCyan.png
  LimeWorker/FarmerLime.png
  PurpleWorker/FarmerPurple.png
  RedWorker/FarmerRed.png

Soldiers/Melee/
  CyanMelee/SwordsmanCyan.png
  LimeMelee/SwordsmanLime.png
  PurpleMelee/SwordsmanPurple.png
  RedMelee/SwordsmanRed.png

Soldiers/Ranged/
  CyanRanged/MageCyan.png
  LimeRanged/MageLime.png
  PurpleRanged/MagePurple.png
  RedRanged/MageRed.png
```

---

## Miscellaneous

### Well.png (5 cols × 1 row)
- Different well variants

### Bridge.png (2 cols × 3 rows)
| Col | Row | Description |
|-----|-----|-------------|
| 0 | 0-2 | Horizontal bridge pieces |
| 1 | 0-2 | Vertical bridge pieces |

---

## Direction Reference (for app.js)
```javascript
// Base rows for each direction (each direction has 3 rows: idle, walk, action)
const DIRECTION_ROWS = {
    'down': 0,   // rows 0-2
    'left': 3,   // rows 3-5
    'right': 6,  // rows 6-8
    'up': 9      // rows 9-11
};

const SPRITE_COLS = 5;  // 5 animation frames per row
```

## Tile Size
- All tiles: 16×16 pixels
- Scale factor in renderer: typically 2x for display
