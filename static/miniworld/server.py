#!/usr/bin/env python3
"""
Miniworld WebSocket + HTTP Server

Runs the SAGE village simulation with a beautiful island terrain.
Serves both HTTP and WebSocket on the same port for cloudflared compatibility.
"""

import asyncio
import json
import random
import os
import math
from pathlib import Path
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Set

try:
    from aiohttp import web
except ImportError:
    import subprocess
    subprocess.check_call(["pip3", "install", "aiohttp"])
    from aiohttp import web

# ============================================================================
# DATA STRUCTURES
# ============================================================================

@dataclass
class Character:
    id: str
    name: str
    x: int
    y: int
    direction: str = "down"
    state: str = "idle"
    sprite: str = "FarmerCyan"
    anim_frame: int = 0
    destination: Optional[tuple] = None
    speed: float = 4.0  # Faster movement
    move_timer: float = 0.0
    home: Optional[tuple] = None
    workplace: Optional[tuple] = None
    current_goal: str = "wander"
    goal_timer: float = 0.0
    energy: float = 100.0
    social_need: float = 50.0
    talking_to: Optional[str] = None
    talk_timer: float = 0.0
    personality: str = "friendly"
    
    def to_dict(self):
        state_dict = self.state
        if self.talking_to:
            state_dict = {"Talking": {"with": self.talking_to}}
        return {
            "id": self.id, "name": self.name, "x": self.x, "y": self.y,
            "direction": self.direction, "state": state_dict,
            "sprite": self.sprite, "anim_frame": self.anim_frame,
        }
    
    def go_to(self, target: tuple, width: int, height: int, walkable_check=None):
        tx, ty = target
        for radius in range(5):
            for dx in range(-radius, radius + 1):
                for dy in range(-radius, radius + 1):
                    nx, ny = tx + dx, ty + dy
                    if 0 <= nx < width and 0 <= ny < height:
                        if walkable_check is None or walkable_check(nx, ny):
                            self.destination = (nx, ny)
                            self.state = "walking"
                            return True
        return False
    
    def wander(self, width: int, height: int, walkable_check=None):
        for _ in range(10):
            new_x = max(6, min(width - 7, self.x + random.randint(-8, 8)))
            new_y = max(6, min(height - 7, self.y + random.randint(-8, 8)))
            if walkable_check is None or walkable_check(new_x, new_y):
                self.destination = (new_x, new_y)
                self.state = "walking"
                return
    
    def step(self, dt: float) -> bool:
        if self.destination is None:
            return True
        dest_x, dest_y = self.destination
        self.move_timer += dt
        if self.move_timer >= 1.0 / self.speed:
            self.move_timer = 0
            dx, dy = dest_x - self.x, dest_y - self.y
            if dx == 0 and dy == 0:
                self.state = "idle"
                self.destination = None
                return True
            if abs(dx) > abs(dy):
                self.direction = "right" if dx > 0 else "left"
                self.x += 1 if dx > 0 else -1
            else:
                self.direction = "down" if dy > 0 else "up"
                self.y += 1 if dy > 0 else -1
            self.state = "walking"
            self.anim_frame = (self.anim_frame + 1) % 5
        return False
    
    def distance_to(self, other: "Character") -> float:
        return math.sqrt((self.x - other.x)**2 + (self.y - other.y)**2)
    
    def face_towards(self, other: "Character"):
        dx = other.x - self.x
        dy = other.y - self.y
        if abs(dx) > abs(dy):
            self.direction = "right" if dx > 0 else "left"
        else:
            self.direction = "down" if dy > 0 else "up"

@dataclass
class Tile:
    ground: str = "Grass"
    overlay: Optional[str] = None
    sprite_col: int = 0
    sprite_row: int = 0
    team_color: Optional[str] = None
    
    def to_dict(self):
        return {
            "ground": self.ground, "overlay": self.overlay,
            "sprite_col": self.sprite_col, "sprite_row": self.sprite_row,
            "team_color": self.team_color,
        }

@dataclass
class World:
    width: int = 60
    height: int = 48
    name: str = "Sage City"
    tiles: List[List[Tile]] = field(default_factory=list)
    characters: Dict[str, Character] = field(default_factory=dict)
    tick: int = 0
    time_of_day: int = 800
    
    def __post_init__(self):
        if not self.tiles:
            self.tiles = [[Tile() for _ in range(self.width)] for _ in range(self.height)]
    
    def is_walkable(self, x: int, y: int) -> bool:
        if x < 0 or x >= self.width or y < 0 or y >= self.height:
            return False
        tile = self.tiles[y][x]
        if tile.ground in ("Water", "WaterShore"):
            return False
        if tile.overlay in ("TreeOak", "TreePine", "TreeDead", "Rock", "House", "Tavern", "Market", "Well", "Chapel"):
            return False
        return True
    
    def to_dict(self):
        return {
            "config": {"width": self.width, "height": self.height, "name": self.name},
            "tiles": [[t.to_dict() for t in row] for row in self.tiles],
            "characters": {k: v.to_dict() for k, v in self.characters.items()},
            "tick": self.tick,
            "time_of_day": self.time_of_day,
        }
    
    def update(self, dt: float):
        self.tick += 1
        self.time_of_day = (self.time_of_day + 1) % 2400
        hour = self.time_of_day // 100
        
        char_list = list(self.characters.values())
        
        for char in char_list:
            char.goal_timer += dt
            char.energy = max(0, min(100, char.energy - dt * 0.1))
            char.social_need = min(100, char.social_need + dt * 0.05)
            
            if char.talking_to:
                char.talk_timer += dt
                other = self.characters.get(char.talking_to)
                if other and char.distance_to(other) <= 2:
                    char.face_towards(other)
                    char.state = "talking"
                    char.social_need = max(0, char.social_need - dt * 2)
                    if char.talk_timer > 5 + random.random() * 10:
                        char.talking_to = None
                        char.talk_timer = 0
                        char.state = "idle"
                else:
                    char.talking_to = None
                    char.talk_timer = 0
                continue
            
            arrived = char.step(dt)
            
            if char.state == "idle" and char.destination is None:
                action_roll = random.random()
                
                if hour >= 22 or hour < 6:
                    if char.home and char.current_goal != "resting":
                        char.current_goal = "go_home"
                        char.go_to(char.home, self.width, self.height, self.is_walkable)
                    elif char.current_goal == "go_home" and arrived:
                        char.current_goal = "resting"
                        char.energy = min(100, char.energy + dt * 5)
                    continue
                
                elif 6 <= hour < 9:
                    if char.workplace and random.random() < 0.25:
                        char.current_goal = "go_work"
                        char.go_to(char.workplace, self.width, self.height, self.is_walkable)
                    elif action_roll < 0.15:
                        char.wander(self.width, self.height, self.is_walkable)
                
                elif 9 <= hour < 17:
                    if char.workplace and char.current_goal != "working":
                        if random.random() < 0.15:
                            char.current_goal = "go_work"
                            char.go_to(char.workplace, self.width, self.height, self.is_walkable)
                    elif action_roll < 0.12:
                        char.wander(self.width, self.height, self.is_walkable)
                
                else:
                    if char.social_need > 40 and char.personality != "shy":
                        for other in char_list:
                            if other.id != char.id and other.talking_to is None:
                                dist = char.distance_to(other)
                                if dist <= 3 and random.random() < 0.25:
                                    char.talking_to = other.id
                                    other.talking_to = char.id
                                    char.talk_timer = 0
                                    other.talk_timer = 0
                                    char.state = "talking"
                                    other.state = "talking"
                                    break
                                elif dist <= 10 and random.random() < 0.15:
                                    char.go_to((other.x, other.y), self.width, self.height, self.is_walkable)
                                    break
                    elif action_roll < 0.18:
                        char.wander(self.width, self.height, self.is_walkable)

# ============================================================================
# IMPROVED NOISE FUNCTION
# ============================================================================

class SimplexNoise:
    """Simple 2D noise for natural terrain generation."""
    def __init__(self, seed=42):
        random.seed(seed)
        self.perm = list(range(256))
        random.shuffle(self.perm)
        self.perm = self.perm + self.perm
    
    def noise2d(self, x, y):
        """Generate smooth noise value between -1 and 1."""
        # Simple hash-based noise with interpolation
        def fade(t):
            return t * t * t * (t * (t * 6 - 15) + 10)
        
        def lerp(a, b, t):
            return a + t * (b - a)
        
        def grad(h, x, y):
            h = h & 3
            if h == 0: return x + y
            if h == 1: return -x + y
            if h == 2: return x - y
            return -x - y
        
        xi = int(math.floor(x)) & 255
        yi = int(math.floor(y)) & 255
        xf = x - math.floor(x)
        yf = y - math.floor(y)
        
        u = fade(xf)
        v = fade(yf)
        
        aa = self.perm[self.perm[xi] + yi]
        ab = self.perm[self.perm[xi] + yi + 1]
        ba = self.perm[self.perm[xi + 1] + yi]
        bb = self.perm[self.perm[xi + 1] + yi + 1]
        
        x1 = lerp(grad(aa, xf, yf), grad(ba, xf - 1, yf), u)
        x2 = lerp(grad(ab, xf, yf - 1), grad(bb, xf - 1, yf - 1), u)
        
        return lerp(x1, x2, v)
    
    def fbm(self, x, y, octaves=4, persistence=0.5, lacunarity=2.0):
        """Fractional Brownian Motion - combines multiple noise octaves."""
        value = 0
        amplitude = 1
        frequency = 1
        max_value = 0
        
        for _ in range(octaves):
            value += amplitude * self.noise2d(x * frequency, y * frequency)
            max_value += amplitude
            amplitude *= persistence
            frequency *= lacunarity
        
        return value / max_value

# ============================================================================
# BEAUTIFUL ISLAND WORLD GENERATION
# ============================================================================

def create_island_world() -> World:
    """Create a beautiful island with natural terrain like the MiniWorld example."""
    world = World(width=60, height=48, name="Sage City")
    noise = SimplexNoise(seed=42)
    
    cx, cy = world.width // 2, world.height // 2
    island_tiles = set()
    
    # === STEP 1: Create smooth organic island shape ===
    for y in range(world.height):
        for x in range(world.width):
            # Elliptical base with noise for organic edges
            dx = (x - cx) / (world.width * 0.42)
            dy = (y - cy) / (world.height * 0.42)
            base_dist = math.sqrt(dx*dx + dy*dy)
            
            # Add noise for coastline variation
            coast_noise = noise.fbm(x * 0.08, y * 0.08, octaves=3) * 0.25
            
            if base_dist + coast_noise < 0.85:
                island_tiles.add((x, y))
    
    # === STEP 2: Fill terrain with varied grass ===
    for y in range(world.height):
        for x in range(world.width):
            if (x, y) in island_tiles:
                # Multi-octave noise for natural grass variation
                n1 = noise.fbm(x * 0.15, y * 0.15, octaves=3)
                n2 = noise.fbm(x * 0.08 + 100, y * 0.08 + 100, octaves=2)
                combined = (n1 + n2) / 2
                
                # Distance from center affects grass type
                dx = (x - cx) / world.width
                dy = (y - cy) / world.height
                dist_factor = math.sqrt(dx*dx + dy*dy)
                
                # Natural grass distribution
                if combined > 0.2 and dist_factor < 0.3:
                    world.tiles[y][x].ground = "GrassTextured"
                    world.tiles[y][x].sprite_col = int((n1 + 1) * 1.5) % 3
                elif combined > 0:
                    world.tiles[y][x].ground = "Grass"
                elif combined > -0.2:
                    world.tiles[y][x].ground = "GrassLight"
                elif combined > -0.4 and dist_factor > 0.25:
                    world.tiles[y][x].ground = "GrassDark"
                else:
                    world.tiles[y][x].ground = "GrassTextured"
                    world.tiles[y][x].sprite_col = int((n2 + 1) * 1.5) % 3
            else:
                world.tiles[y][x].ground = "Water"
    
    # === STEP 3: Create smooth shorelines with sand ===
    shore_tiles = set()
    for y in range(world.height):
        for x in range(world.width):
            if (x, y) not in island_tiles:
                continue
            # Check neighbors for water
            has_water_neighbor = False
            for dx, dy in [(-1,0), (1,0), (0,-1), (0,1)]:
                nx, ny = x + dx, y + dy
                if (nx, ny) not in island_tiles:
                    has_water_neighbor = True
                    break
            
            if has_water_neighbor:
                shore_tiles.add((x, y))
                world.tiles[y][x].ground = "Sand"
    
    # Add beach/sand layer beyond immediate shore
    for y in range(world.height):
        for x in range(world.width):
            if (x, y) in island_tiles and (x, y) not in shore_tiles:
                for dx, dy in [(-1,0), (1,0), (0,-1), (0,1), (-1,-1), (1,-1), (-1,1), (1,1)]:
                    if (x+dx, y+dy) in shore_tiles:
                        if random.random() < 0.4:
                            world.tiles[y][x].ground = "WaterShore"
                        break
    
    # === STEP 4: Create natural path network ===
    def carve_path(x1, y1, x2, y2, width=2):
        """Carve a winding path between two points."""
        x, y = x1, y1
        while abs(x - x2) > 1 or abs(y - y2) > 1:
            # Add some randomness to path
            if abs(x - x2) > abs(y - y2):
                x += 1 if x2 > x else -1
                if random.random() < 0.3 and abs(y - y2) > 0:
                    y += 1 if y2 > y else -1
            else:
                y += 1 if y2 > y else -1
                if random.random() < 0.3 and abs(x - x2) > 0:
                    x += 1 if x2 > x else -1
            
            # Carve path width
            for dx in range(-width//2, width//2 + 1):
                for dy in range(-width//2, width//2 + 1):
                    px, py = x + dx, y + dy
                    if (px, py) in island_tiles and world.tiles[py][px].ground not in ("Water", "WaterShore"):
                        world.tiles[py][px].ground = "Path"
    
    # Main roads
    carve_path(cx, cy - 12, cx, cy + 12, 2)  # North-South
    carve_path(cx - 15, cy, cx + 15, cy, 2)   # East-West
    
    # Side paths
    carve_path(cx, cy, cx - 12, cy - 8, 1)   # To northwest
    carve_path(cx, cy, cx + 12, cy - 8, 1)   # To northeast
    carve_path(cx, cy, cx - 12, cy + 8, 1)   # To southwest
    carve_path(cx, cy, cx + 12, cy + 8, 1)   # To southeast
    
    # === STEP 5: Town square (stone center) ===
    for y in range(cy - 2, cy + 3):
        for x in range(cx - 3, cx + 4):
            if (x, y) in island_tiles:
                world.tiles[y][x].ground = "Stone"
    
    # === STEP 6: Place buildings ===
    def place_building(bx, by, btype, color, variant):
        """Place a two-tile tall building, ensuring ground is always set."""
        if not (0 <= by < world.height and 0 <= bx < world.width):
            return
        if (bx, by) not in island_tiles:
            return
        
        # Ensure ground is grass (not water/shore) under buildings
        if world.tiles[by][bx].ground in ("Water", "WaterShore", "Sand"):
            world.tiles[by][bx].ground = "Grass"
        # Keep existing ground type if it's already valid land
        
        world.tiles[by][bx].overlay = btype
        world.tiles[by][bx].team_color = color
        world.tiles[by][bx].sprite_col = variant
        world.tiles[by][bx].sprite_row = 0
        
        if by + 1 < world.height and (bx, by + 1) in island_tiles:
            if world.tiles[by+1][bx].ground in ("Water", "WaterShore", "Sand"):
                world.tiles[by+1][bx].ground = "Grass"
            
            world.tiles[by+1][bx].overlay = btype
            world.tiles[by+1][bx].team_color = color
            world.tiles[by+1][bx].sprite_col = variant
            world.tiles[by+1][bx].sprite_row = 1
    
    # Town center buildings
    place_building(cx - 5, cy - 4, "Tavern", "Wood", 0)
    place_building(cx + 5, cy - 4, "Market", "Wood", 0)
    place_building(cx, cy - 6, "Chapel", "Wood", 0)
    
    # Well in center (single tile)
    if (cx, cy) in island_tiles:
        world.tiles[cy][cx].overlay = "Well"
        world.tiles[cy][cx].sprite_col = 0
        world.tiles[cy][cx].sprite_row = 0
    
    # Houses arranged in neighborhoods
    houses = [
        # Northwest neighborhood
        (cx - 10, cy - 8, "Purple", 0),
        (cx - 8, cy - 9, "Purple", 1),
        (cx - 12, cy - 6, "Wood", 2),
        # Northeast neighborhood
        (cx + 10, cy - 8, "Cyan", 0),
        (cx + 8, cy - 9, "Cyan", 1),
        (cx + 12, cy - 6, "Wood", 0),
        # Southwest neighborhood
        (cx - 10, cy + 6, "Lime", 2),
        (cx - 8, cy + 7, "Lime", 0),
        (cx - 12, cy + 4, "Wood", 1),
        # Southeast neighborhood  
        (cx + 10, cy + 6, "Red", 0),
        (cx + 8, cy + 7, "Red", 1),
        (cx + 12, cy + 4, "Wood", 2),
    ]
    
    for hx, hy, color, variant in houses:
        place_building(hx, hy, "House", color, variant)
    
    # === STEP 7: Wheat fields ===
    def place_field(fx, fy, w, h):
        for dy in range(h):
            for dx in range(w):
                px, py = fx + dx, fy + dy
                if (px, py) in island_tiles and world.tiles[py][px].overlay is None:
                    ground = world.tiles[py][px].ground
                    # Place fields on grass/path, ensure valid ground
                    if ground not in ("Water", "WaterShore", "Stone", "Sand"):
                        # Ensure we have grass underneath
                        if ground in ("Path",):
                            world.tiles[py][px].ground = "GrassLight"  # Tilled soil look
                        world.tiles[py][px].overlay = "Wheatfield"
                        world.tiles[py][px].sprite_col = (dx + dy) % 4
                        world.tiles[py][px].sprite_row = 0
    
    place_field(cx + 6, cy + 3, 5, 4)   # East field
    place_field(cx - 11, cy + 3, 4, 3)  # West field
    
    # === STEP 8: Dense forests at edges ===
    for y in range(world.height):
        for x in range(world.width):
            if (x, y) not in island_tiles:
                continue
            if world.tiles[y][x].overlay is not None:
                continue
            if world.tiles[y][x].ground in ("Path", "Stone", "WaterShore", "Sand", "Water"):
                continue
            
            # Distance from center
            dx = abs(x - cx) / world.width
            dy = abs(y - cy) / world.height
            dist = math.sqrt(dx*dx + dy*dy)
            
            # Use noise for natural tree clustering
            tree_noise = noise.fbm(x * 0.12, y * 0.12, octaves=2)
            
            # Very dense at edges, sparse in center
            base_chance = 0.6 * dist + 0.05  # 5% in center, up to 35% at edge
            cluster_bonus = 0.3 if tree_noise > 0.2 else 0
            tree_chance = base_chance + cluster_bonus
            
            if random.random() < tree_chance:
                # Ensure ground is valid grass under trees
                if world.tiles[y][x].ground in ("Water", "WaterShore"):
                    continue  # Don't place trees in water
                
                # Pine trees more common at edges
                if dist > 0.35 and random.random() < 0.6:
                    world.tiles[y][x].overlay = "TreePine"
                    world.tiles[y][x].sprite_col = random.randint(0, 2)  # 3 variants
                else:
                    world.tiles[y][x].overlay = "TreeOak"
                    world.tiles[y][x].sprite_col = random.randint(0, 3)  # 4 variants
                world.tiles[y][x].sprite_row = 0
    
    # === STEP 9: Scatter rocks ===
    for _ in range(25):
        rx = random.randint(8, world.width - 8)
        ry = random.randint(8, world.height - 8)
        if (rx, ry) in island_tiles and world.tiles[ry][rx].overlay is None:
            ground = world.tiles[ry][rx].ground
            # Only place rocks on valid ground
            if ground not in ("Path", "Stone", "Water", "WaterShore", "Sand"):
                world.tiles[ry][rx].overlay = "Rock"
                world.tiles[ry][rx].sprite_col = random.randint(0, 2)
                world.tiles[ry][rx].sprite_row = random.randint(0, 3)
    
    return world, island_tiles

def add_sages(world: World, island_tiles: set):
    """Add villagers with homes and jobs."""
    cx, cy = world.width // 2, world.height // 2
    
    tavern_loc = (cx - 5, cy - 3)
    market_loc = (cx + 5, cy - 3)
    town_square = (cx, cy)
    wheat_field = (cx + 8, cy + 4)
    wheat_field_west = (cx - 9, cy + 4)
    
    homes = {
        "purple": (cx - 10, cy - 7),
        "cyan": (cx + 10, cy - 7),
        "lime": (cx - 10, cy + 7),
        "red": (cx + 10, cy + 7),
    }
    
    sages_data = [
        # Mages
        ("cedric", "Cedric", cx - 6, cy - 2, "MagePurple", "purple", homes["purple"], "busy"),
        ("petra", "Petra", cx + 6, cy - 2, "MageCyan", "cyan", homes["cyan"], "friendly"),
        ("ivy", "Ivy", cx + 2, cy + 2, "MageLime", "lime", market_loc, "social"),
        ("sage", "Sage", cx - 2, cy + 2, "MageRed", "red", tavern_loc, "shy"),
        
        # Farmers
        ("aldric", "Aldric", cx - 4, cy + 4, "FarmerCyan", "cyan", wheat_field, "friendly"),
        ("brynn", "Brynn", cx + 4, cy + 4, "FarmerLime", "lime", wheat_field, "busy"),
        ("dara", "Dara", cx + 6, cy + 5, "FarmerPurple", "purple", wheat_field, "social"),
        ("hana", "Hana", cx - 8, cy + 5, "FarmerRed", "red", wheat_field_west, "friendly"),
        ("kira", "Kira", cx - 6, cy + 3, "FarmerCyan", "cyan", wheat_field_west, "busy"),
        ("oak", "Oak", cx, cy + 6, "FarmerLime", "lime", wheat_field, "shy"),
        
        # Warriors/Guards
        ("ember", "Ember", cx + 3, cy - 3, "SwordsmanRed", "red", town_square, "social"),
        ("flint", "Flint", cx - 3, cy - 3, "SwordsmanCyan", "cyan", town_square, "friendly"),
        ("haven", "Haven", cx, cy - 4, "SwordsmanLime", "lime", tavern_loc, "busy"),
        ("thor", "Thor", cx + 8, cy, "SwordsmanPurple", "purple", market_loc, "social"),
        ("vera", "Vera", cx - 8, cy, "SwordsmanRed", "red", tavern_loc, "friendly"),
    ]
    
    for sid, name, x, y, sprite, home_color, workplace, personality in sages_data:
        for radius in range(10):
            found = False
            for dx in range(-radius, radius + 1):
                for dy in range(-radius, radius + 1):
                    nx, ny = x + dx, y + dy
                    if (nx, ny) in island_tiles and world.is_walkable(nx, ny):
                        char = Character(
                            id=sid, name=name, x=nx, y=ny, sprite=sprite,
                            home=homes.get(home_color),
                            workplace=workplace,
                            personality=personality,
                            energy=80 + random.random() * 20,
                            social_need=30 + random.random() * 40,
                        )
                        world.characters[sid] = char
                        found = True
                        break
                if found:
                    break
            if found:
                break

# ============================================================================
# SERVER
# ============================================================================

class MiniworldServer:
    def __init__(self):
        self.world, self.island_tiles = create_island_world()
        add_sages(self.world, self.island_tiles)
        self.clients: Set[web.WebSocketResponse] = set()
        self.static_dir = Path(__file__).parent
        print(f"🏝️  Created {self.world.name} with {len(self.world.characters)} villagers")
    
    async def send_state(self, ws: web.WebSocketResponse):
        msg = {"type": "world_state", "world": self.world.to_dict()}
        try:
            await ws.send_json(msg)
        except:
            pass
    
    async def broadcast_state(self):
        if not self.clients:
            return
        msg = {"type": "world_state", "world": self.world.to_dict()}
        dead = set()
        for ws in self.clients:
            try:
                await ws.send_json(msg)
            except:
                dead.add(ws)
        self.clients -= dead
    
    async def websocket_handler(self, request):
        ws = web.WebSocketResponse()
        await ws.prepare(request)
        
        self.clients.add(ws)
        print(f"Client connected ({len(self.clients)} total)")
        await self.send_state(ws)
        
        try:
            async for msg in ws:
                pass
        except:
            pass
        finally:
            self.clients.discard(ws)
            print(f"Client disconnected ({len(self.clients)} total)")
        
        return ws
    
    async def simulation_loop(self):
        while True:
            self.world.update(0.1)
            await self.broadcast_state()
            await asyncio.sleep(0.1)
    
    async def on_startup(self, app):
        asyncio.create_task(self.simulation_loop())
    
    def create_app(self):
        app = web.Application()
        app.on_startup.append(self.on_startup)
        
        # Landing page at root
        landing_dir = self.static_dir.parent  # static/ directory
        app.router.add_get('/ws', self.websocket_handler)
        app.router.add_get('/', lambda r: web.FileResponse(landing_dir / 'index.html'))
        
        # City/game at /city
        app.router.add_get('/city', lambda r: web.FileResponse(self.static_dir / 'index.html'))
        app.router.add_static('/city/', self.static_dir)
        
        # Sprites and miniworld assets at root level (for backward compat)
        app.router.add_static('/sprites', self.static_dir / 'sprites')
        
        # Dashboard and other static dirs
        app.router.add_static('/dashboard', landing_dir / 'dashboard')
        app.router.add_static('/journals', landing_dir / 'journals')
        
        # Catch-all static from landing dir
        app.router.add_static('/', landing_dir)
        return app
    
    def run(self, port=8888):
        app = self.create_app()
        print(f"🌐 Sage City: http://localhost:{port}")
        web.run_app(app, host='0.0.0.0', port=port, print=None)

if __name__ == "__main__":
    import sys
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8888
    server = MiniworldServer()
    server.run(port)
