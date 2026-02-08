# 🎨 MiniWorld Sprite Sheet Reference

**Generated from deep analysis of actual sprite sheets**

---

## 📐 Base Grid
- **Tile Size**: 16×16 pixels
- **All sprites align to 16px grid**
- **Buildings may span multiple tiles (32×32, 48×32, etc.)**

---

## 🌍 GROUND TILES (`assets/Ground/`)

### Grass.png (80×16 = 5 tiles × 1 row)
| Col 0 | Col 1 | Col 2 | Col 3 | Col 4 |
|-------|-------|-------|-------|-------|
| Light Green | Medium Green | Dark Green | Yellow-Green | Variation |

### TexturedGrass.png (96×32 = 6 tiles × 2 rows)
- **Row 0**: Grass with flowers/details (6 variations)
- **Row 1**: Alternate grass textures (6 variations)
- Use random variation for visual interest

### Shore.png (80×16 = 5 tiles × 1 row)
| Col 0 | Col 1 | Col 2 | Col 3 | Col 4 |
|-------|-------|-------|-------|-------|
| Deep Water | Shallow | Sand/Shore | Light Sand | Beach |
- Transition tiles from water to land

### Cliff.png (Complex tileset ~160×80)
Multi-part cliff/elevation system:
- **Top-left section**: Cliff tops and edges
- **Middle section**: Cliff faces
- **Bottom section**: Corners and transitions
- Use for elevated terrain with proper edge pieces

### Cliff-Water.png (Approximately 80×48)
- Island/landmass shapes surrounded by water
- Pre-composed cliff-to-water transitions

### DeadGrass.png, Winter.png
- Seasonal/biome variations of grass tiles

---

## 🌲 NATURE (`assets/Nature/`)

### Trees.png (48×16 = 3 trees)
| Col 0 | Col 1 | Col 2 |
|-------|-------|-------|
| Small Tree/Bush | Medium Tree | Large Tree |
- Trees are 16×16 but visually extend above their tile
- Place with y-offset for proper depth sorting

### PineTrees.png (48×16 = 3 trees)
| Col 0 | Col 1 | Col 2 |
|-------|-------|-------|
| Small Pine | Medium Pine | Large Pine |

### CoconutTrees.png (80×16 = 5 palms)
- Various palm tree sizes and poses
- Good for coastal/beach areas

### Rocks.png (Approximately 64×48 = 4×3 grid)
- **Row 0**: Gray rocks (4 sizes)
- **Row 1**: Gold ore rocks (4 sizes)  
- **Row 2**: Additional variations
- Good for decoration and resource nodes

### Wheatfield.png (64×16 = 4 tiles)
| Col 0 | Col 1 | Col 2 | Col 3 |
|-------|-------|-------|-------|
| Seedling | Growing | Mature | Ready to Harvest |
- Show crop growth stages

### Other Nature
- `Cactus.png` - Desert vegetation
- `DeadTrees.png` - Leafless trees
- `WinterTrees.png` - Snow-covered trees
- `Tumbleweed.png` - Animated rolling

---

## 🏠 BUILDINGS (`assets/Buildings/`)

### VERIFIED Building Dimensions:

#### Wood/Huts.png (80×16 = 5×1)
- **5 simple 16×16 hut variants in a row**
- Single-tile buildings, good for farms/outskirts

#### Wood/Houses.png (48×64 = 3×4 in 16px units)
- **Each house is 16×32 (1 tile wide × 2 tiles tall)**
- 3 columns × 2 building rows = 6 house variants
- Draw at `(x, y-16)` to account for height

#### Wood/Taverns.png (48×64 = 3×4 in 16px units)
- **Each tavern is 16×32 (1 tile wide × 2 tiles tall)**
- 3 columns × 2 building rows = 6 tavern variants

#### Wood/Tower.png (48×96 = 3×6 in 16px units)
- **Each tower is 16×32 (1 tile wide × 2 tiles tall)**
- 3 columns × 3 building rows = 9 tower variants
- Different damage states or upgrade levels

#### Wood/Keep.png (96×64 = 6×4 in 16px units)
- **Each keep is 32×32 (2 tiles wide × 2 tiles tall)**
- 3 keep variants in 2 rows = 6 total
- Large castle/fortress main buildings

#### Wood/Chapels.png (48×32 = 3×2 in 16px units)
- **Each chapel is 16×16 (single tile)**
- 3 columns × 2 rows = 6 chapel variants

#### Wood/Market.png (48×64 = 3×4 in 16px units)
- **Each market stall is 16×16**
- 3 columns × 4 rows = 12 market variants

#### Wood/Workshops.png (48×48 = 3×3 in 16px units)
- **Each workshop is 16×16**
- 3 columns × 3 rows = 9 variants

#### Wood/Resources.png (48×80 = 3×5 in 16px units)
- 3 columns × 5 rows of resource buildings
- Lumbermill, Windmill, Mining Shaft, Silo, etc.

#### Wood/Docks.png (64×32)
- Dock/pier structures

#### Colored Variants (Cyan/, Lime/, Purple/, Red/)
- Same buildings with faction colors
- Use for different districts or families

---

## 👤 CHARACTERS (`assets/Characters/`)

### ⚠️ CRITICAL: Animation Row Layout (All Characters)

**VERIFIED: Character sheets are 5 frames wide × 12 rows (or 14 for soldiers with shields)**

**FarmerRed.png: 80×192 = 5 frames × 12 rows (16×16 per frame)**
**SpearmanCyan.png: 80×224 = 5 frames × 14 rows**

```
Row 0:  IDLE   facing DOWN  (5 frames)
Row 1:  WALK   facing DOWN  (5 frames)
Row 2:  ATTACK facing DOWN  (5 frames)
Row 3:  IDLE   facing LEFT  (5 frames)
Row 4:  WALK   facing LEFT  (5 frames)
Row 5:  ATTACK facing LEFT  (5 frames)
Row 6:  IDLE   facing RIGHT (5 frames)
Row 7:  WALK   facing RIGHT (5 frames)
Row 8:  ATTACK facing RIGHT (5 frames)
Row 9:  IDLE   facing UP    (5 frames)
Row 10: WALK   facing UP    (5 frames)
Row 11: ATTACK facing UP    (5 frames)
Row 12-13: Shield animations (soldiers only)
```

**Formula for row selection:**
```javascript
const DIR = { DOWN: 0, LEFT: 1, RIGHT: 2, UP: 3 };
const ROWS_PER_DIR = 3; // idle, walk, attack
const FRAMES_PER_ROW = 5;

const IDLE_ROW = (direction) => direction * ROWS_PER_DIR + 0;
const WALK_ROW = (direction) => direction * ROWS_PER_DIR + 1;
const ATTACK_ROW = (direction) => direction * ROWS_PER_DIR + 2;
```

**Animation timing (from guide):**
- IDLE: 300ms per frame
- WALK: 200ms per frame
- ATTACK: 100ms per frame

### Workers/ (FarmerTemplate.png and colored variants)
- `FarmerTemplate.png` - Gray/template version
- `FarmerRed.png`, `FarmerCyan.png`, `FarmerLime.png`, `FarmerPurple.png`
- 64×144 (4 frames × 9 rows) = 16×16 per frame

### Soldiers/Melee/ (Swordsman, Axeman, Spearman, Assassin)
- Same 4×9 grid layout
- Colored variants for each faction

### Soldiers/Ranged/ (Bowman, Mage, Musketeer)
- Same layout structure
- Mage may have spell-casting frames

### Soldiers/Mounted/ (Knights)
- 32×32 sprites (rider + horse)
- Different frame counts

### Champions/ (Named heroes)
- Unique character designs
- Same animation structure

---

## 🐾 ANIMALS (`assets/Animals/`)

### VERIFIED Animal Dimensions:
- `Chicken.png` - 64×64 = 4 frames × 4 rows
- `Sheep.png` - 64×64 = 4 frames × 4 rows
- `Pig.png` - 64×128 = 4 frames × 8 rows
- `Boar.png` - 64×128 = 4 frames × 8 rows
- `Horse(32x32).png` - 128×192 = 4 frames × 6 rows (32×32 per frame)

### Animal Layout (4 frames × 4-8 rows)
```
Row 0: facing DOWN (4 frames)
Row 1: facing LEFT (4 frames)
Row 2: facing RIGHT (4 frames)
Row 3: facing UP (4 frames)
Row 4+: Additional animations (for larger animals)
```

**Note**: Animals use 4 columns, not 5!

---

## 👹 MONSTERS (`assets/Characters/Monsters/`)

### Slimes/ (Simple creatures)
- `SlimeBlue.png` - 80×80 (5 frames × 5 rows)
- Simpler animation: bounce, attack, die

### Orcs/
- `Orc.png` - Full character sheet (4×9 grid)
- Goblins: `ArcherGoblin.png`, `ClubGoblin.png`, etc.
- Same structure as player characters

### Undead/
- `Skeleton-Soldier.png` - Standard character layout
- `Necromancer.png`

### Dragons/
- Large sprites (32×32 or bigger)
- Flying animations

---

## 🎯 MISCELLANEOUS (`assets/Miscellaneous/`)

### Bridge.png (32×16 = 2 bridges)
| Col 0 | Col 1 |
|-------|-------|
| Horizontal Bridge | Vertical Bridge |
- Each bridge is 16×16

### Well.png (16×16)
- Single tile well sprite

### Ships (Various sizes)
- `Boat.png` - Small 16×16
- `TransportShip.png` - Larger
- `WarShip.png` - Largest

### Signs.png
- Directional signs, signposts

### Portal.png
- Animated portal effect

### Tombstones.png
- Graveyard decorations

### Chests.png
- Treasure/storage containers

---

## 🖼️ UI (`assets/User Interface/`)

### UiIcons.png
- Game UI icons (hearts, coins, etc.)

### Icons-Essentials.png
- Basic UI elements

### BoxSelector.png, Highlighted-Boxes.png
- Selection indicators

---

## 📋 IMPLEMENTATION CHECKLIST

### Terrain Rendering:
- [ ] Use actual grass tile sprites instead of solid fills
- [ ] Implement shore transitions properly
- [ ] Add cliff edges for elevation
- [ ] Randomize tile variations

### Building Rendering:
- [ ] Draw buildings at correct multi-tile size
- [ ] Offset y-position for tall buildings
- [ ] Use proper building sprite regions

### Character Animation:
- [ ] Fix row calculation: `dir * 2 + (isWalking ? 1 : 0)`
- [ ] Use correct frame timing
- [ ] Handle attack animations when implemented

### Animal Animation:
- [ ] Use 5-column layout (not 4)
- [ ] Simpler row structure than characters

### Depth Sorting:
- [ ] Sort all entities by y-position
- [ ] Trees/buildings need y-offset consideration

---

## 🎬 ANIMATION TIMING QUICK REFERENCE

| Animation | MS per Frame | Total Frames |
|-----------|--------------|--------------|
| Idle | 300 | 4 (loop) |
| Walk | 200 | 4 (loop) |
| Attack | 100 | 4-6 |
| Animal Idle | 350 | 5 |
| Animal Walk | 200 | 5 |

---

*Reference built from visual analysis of MiniWorld sprite assets by Shade*
