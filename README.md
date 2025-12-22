# SANTA'S RESCUE

**Product Requirements Document & Game Design Document**  
NES-Style 2D Platformer | Turbo Engine (Rust)  
Version 1.0 | Production-Ready Specification

---

## 📋 DOCUMENT INFORMATION

**Purpose:** Complete technical specification for Santa's Rescue  
**Audience:** Development team, LLM code generators, QA  
**Scope:** 6-day development sprint  
**Status:** APPROVED

---

## 1. EXECUTIVE SUMMARY

### 1.1 Game Overview

**Working Title:** Santa's Rescue  
**Tagline:** "Save Christmas, One Present at a Time"

Santa's Rescue is a precision 2D platformer recreating 1986 NES action games. Control Santa through trap-filled environments, rescue presents, defeat enemies, race against time.

### 1.2 Core Pillars

- **Retro Authenticity:** Frame-accurate physics, NES palettes, 8-bit audio
- **Precision Gameplay:** Tight controls, predictable mechanics
- **Christmas Theme:** Festive without gimmicky
- **Arcade Challenge:** High score, time pressure, risk-reward
- **Replayability:** Multiple routes, mastery-driven

### 1.3 Inspiration

**Primary:** The Goonies (1986, Konami, NES)
- Multi-screen structure, rescue gameplay, ladder navigation

**Secondary:** Castlevania, Mega Man, Metroid

**Technical:** NES hardware constraints emulation

### 1.4 Hackathon Alignment

- **30% Gameplay & Fun:** Tight controls, clear goals, balanced difficulty
- **25% Technical:** Frame-perfect physics, 60 FPS, robust collision
- **25% Arcade Auth:** NES-accurate visuals/audio, retro design
- **10% Christmas:** Integrated festive elements
- **10% Innovation:** Evil Santa power-up, browser accessible

---

## 2. TECHNICAL FOUNDATION

### 2.1 Platform

**Engine:** Turbo (Rust/WebAssembly)  
**Target:** Web browsers (desktop-first)  
**Input:** Keyboard (arrow keys + Z/X)  
**Performance:** 60 FPS, <50ms input latency

### 2.2 Screen & Coordinates

**Resolution:** 360 × 240 (NES spec)  
**Visible:** 360 × 224 (top 16px HUD)  
**Origin:** Top-left (0,0)  
**Axes:** +X = Right, +Y = Down  
**Precision:** 16-bit fixed-point (8.8)

### 2.3 Tile System

**Tile Size:** 16 × 16 pixels  
**Grid:** 16 wide × 14 tall  

**Tile Types:**
- **Solid:** Blocks movement
- **Platform:** Solid from above only
- **Ladder:** Vertical climbing
- **Spike/Hazard:** Damages player
- **Empty:** No collision

**Scrolling:** Horizontal page-based, no vertical

### 2.4 Color Palette

**NES 2C02 palette** (54 colors), 4 colors per tile.

**Santa Palette:**
- `#F8F8F8` (white)
- `#E01C28` (red)
- `#FCBCB0` (skin)
- `#000000` (black)

**Environment:**

*Snow:*
- `#E0F8FF`, `#9CC4E4`, `#5090D0`, `#003C88`

*Rock:*
- `#B88058`, `#806850`, `#584840`, `#302820`

*Lava:*
- `#FCA044`, `#E45C10`, `#881400`, `#000000`

---

## 3. PLAYER CHARACTER

### 3.1 Visual Specs

**Sprite:** 32 × 32px (2×2 meta-sprite)  
**Hitbox:** 14 × 28px (bottom-aligned, centered)  
**Rationale:** Edge forgiveness, prevents wall-clipping feel.

### 3.2 Animation States

- **Idle:** 2 frames, 30 frames/sprite
- **Walk:** 4 frames, 8 frames/sprite
- **Run:** 4 frames, 6 frames/sprite
- **Jump:** 1 frame (static)
- **Fall:** 1 frame
- **Climb:** 2 frames, 10 frames/sprite
- **Attack:** 3 frames (startup/active/recovery), 4 frames each
- **Hurt:** 1 frame, 30 frames duration (invuln flashing)
- **Death:** 4 frames, 15 frames/sprite
- **Evil Santa:** Palette swap all states

### 3.3 Movement Physics

#### 3.3.1 Horizontal

**Walk:** 1.5 px/frame (384 subpixels)  
**Run:** 2.5 px/frame (640 subpixels)  
**Accel:** 0.2 px/frame² (51 subpixels)  
**Decel:** 0.4 px/frame² (102 subpixels)  
**Air Control:** 50% (0.1 px/frame²)

#### 3.3.2 Vertical

**Gravity:** 0.5 px/frame² (128 subpixels)  
**Terminal:** 6.0 px/frame  
**Jump Vel:** -5.5 px/frame  
**Jump Height (full):** ~48px (3 tiles)  
**Jump Height (short):** ~20px (1.25 tiles)  
**Variable Jump:** Release early = shorter

#### 3.3.3 Special Mechanics

**Coyote Time:** 4 frames (0.067s)  
**Input Buffer:** 3 frames (0.05s)

**Ladder:**
- UP on ladder tile = climb mode
- Climb speed: 1.0 px/frame
- Jump off ladder (left/right + jump)
- Auto-dismount at top

### 3.4 Attack System

**Weapon:** Candy Cane projectiles

**Timing:**
- **Startup:** 4 frames
- **Active:** 1 frame (spawn)
- **Recovery:** 7 frames
- **Total:** 12 frames (0.2s)
- **Cooldown:** 24 frames (0.4s)

**Projectile:**
- **Speed:** 4.0 px/frame
- **Range:** 128px
- **Hitbox:** 8×8px
- **Damage:** 1 HP
- **Max Active:** 2
- **Animation:** 2-frame spin

```rust
// Attack Logic
if attack_btn && cooldown==0:
    state = ATTACKING, frame_ctr = 0
if state == ATTACKING:
    frame_ctr += 1
    if frame_ctr == 4: spawn_projectile()
    if frame_ctr >= 12:
        state = IDLE/WALK/JUMP
        cooldown = 24
```

### 3.5 Health & Damage

**Max Health:** 6 HP (3 hearts, 2 HP each)

**Damage:**
- **Enemy contact:** 2 HP
- **Spike:** 2 HP
- **Projectile:** 1 HP

**Invuln:** 90 frames (1.5s), sprite flashes every 4  
**Knockback:** 16px horizontal, 32px vertical  
**Death:** 60 frame tumble, fade to black  
**Lives:** 3 lives/continue, game over at 0

### 3.6 Evil Santa Power-Up

#### 3.6.1 Activation

**Trigger:** Dark Gift item  
**Duration:** 300 frames (5s) countdown  
**Warning:** Flashes red at 60 frames (1s) left

#### 3.6.2 Visual

- **Palette:** Red→Purple/black (`#5C1D87`, `#2E0854`)
- **Eyes:** White→Glowing red (`#FF0000`)
- **Aura:** Dark particle trail (8 particles, 0.5px/frame decay)

#### 3.6.3 Gameplay

- **Invincibility:** No damage from any source
- **Collision:** Enemies destroyed on contact (100 pts each)
- **Attack:** 2× damage, 1.5× speed (6.0 px/frame)
- **Movement:** 10% faster (Walk: 1.65, Run: 2.75)
- **Jump:** +0.5 px/frame velocity

**Strategic:** Encourages aggression, speedrun optimization.

---

## 4. ENEMIES & AI

### 4.1 Design Philosophy

NES-era AI: simple, readable patterns, no randomness, predictable when learned. Enemies as routing obstacles.

### 4.2 Enemy Types

#### 4.2.1 Gingerbread Soldier (Ground Patrol)

**Size:** 16×16px  
**Hitbox:** 12×14px  
**Health:** 2 HP  
**Damage:** 2 HP  
**Movement:** Horizontal patrol, 0.75 px/frame  
**Behavior:** Reverses at boundaries, no falling

```rust
// FSM: PATROL_LEFT/RIGHT, reverses at boundaries
```

#### 4.2.2 Nutcracker Jumper (Vertical)

**Size:** 16×24px  
**Hitbox:** 12×20px  
**Health:** 3 HP  
**Damage:** 2 HP  
**Movement:** Vertical bounce, fixed location  
**Cycle:** 60 frames grounded, 40 airborne  
**Jump:** -4.5 px/frame, ~40px height

```rust
// FSM: GROUNDED/AIRBORNE, timer-based jumps
```

#### 4.2.3 Elf Climber (Ladder)

**Size:** 16×16px  
**Hitbox:** 12×14px  
**Health:** 2 HP  
**Damage:** 2 HP  
**Movement:** Climbs ladders, patrols top/bottom  
**Speeds:** Climb: 0.8 px/frame, Walk: 0.5

```rust
// FSM: CLIMBING_UP/DOWN, PATROL_TOP/BOTTOM
```

#### 4.2.4 Snowman Turret (Projectile)

**Size:** 24×24px  
**Hitbox:** 20×20px  
**Health:** 4 HP (tankier)  
**Movement:** Stationary  
**Fire Rate:** 1 shot/120 frames (2s)  
**Projectile:** 2.5 px/frame speed, 1 HP damage, 6×6px hitbox

```rust
// FSM: IDLE/FIRING, aims toward player
```

#### 4.2.5 Reindeer Stalker (Pressure)

**Size:** 24×24px  
**Hitbox:** 18×20px  
**Health:** 5 HP (toughest)  
**Spawn:** Timer < 60s (anti-stalling)  
**Movement:** Chases player, phases through walls  
**Speed:** 1.2 px/frame (slower than walk)

```rust
// FSM: CHASE, always moves toward player, ignores terrain
```

### 4.3 Spawning & Hitstun

- **Placement:** Manual in level data
- **Spawn Delay:** 30 frames (0.5s) after screen transition
- **No Respawn:** Enemies stay dead
- **Screen Limit:** Max 8 active/screen
- **Hitstun:** 12 frames, flash white, 8px knockback
- **Death:** Particle explosion (8 particles, radial)

---

## 5. CORE SYSTEMS

### 5.1 Scoring

- **Gingerbread:** 100
- **Nutcracker:** 150
- **Elf:** 100
- **Snowman:** 200
- **Reindeer:** 500
- **Present:** 250
- **Cookie:** 50
- **Star:** 100
- **Evil Santa Kill:** 100
- **Time Bonus:** remaining_seconds × 10
- **Extra Life:** Every 10,000 points

### 5.2 Lives & Continue

**Starting:** 3 lives  
**Death:** Lose 1 life, respawn at checkpoint with full HP  
**Game Over:** Lives = 0  
**Continue:** 3 continues, each restores 3 lives, resets score

### 5.3 Timer

**Time Limit:** 180 seconds (3 min)/level  
**Display:** MM:SS in HUD  
**Warning:** Flashes red < 30s  
**Countdown SFX:** Beep every second < 10s  
**Time Out:** Instant death at 0:00  
**Reindeer:** Spawns at < 60s

### 5.4 Collectibles

#### 5.4.1 Presents (Objective)

- 16×16px gift box with sparkle, 3-5 per level
- +250 points, need ≥1 to unlock exit
- **Perfect Bonus:** All presents = +1,000 pts

#### 5.4.2 Cookies (Health)

- 8×8px, enemy drops (20% basic, 50% tough)
- Restore 2 HP, +50 pts
- Despawns after 300 frames (5s)

#### 5.4.3 Stars (Optional)

- 8×8px golden star, hidden areas (5-10/level)
- +100 pts, 100% completion tracker

#### 5.4.4 Dark Gift (Power-Up)

- 16×16px black box, purple aura
- 1 per level, challenging location
- Activates Evil Santa (300 frames)

### 5.5 Collision

**Layers:**
- **0:** Terrain (tiles)
- **1:** Player
- **2:** Enemies
- **3:** Player Projectiles
- **4:** Enemy Projectiles
- **5:** Collectibles

**Interactions:**
- **Player vs Terrain:** Block, platform passthrough below
- **Player vs Enemies:** Damage, knockback, invuln
- **Projectiles vs Enemies:** Damage, knockback
- **Player vs Collectibles:** Auto-collect

```rust
// Collision Detection
For each entity:
    pred_pos = pos + vel
    tiles = get_overlapping(hitbox, pred_pos)
    for tile: resolve_collision(entity, tile)
    nearby = spatial_grid.query(hitbox)
    for other: if overlap: handle_collision()
```

---

## 6. LEVEL DESIGN

### 6.1 Structure

**Format:** Multi-screen horizontal (4-8 screens/stage)  
**Dimensions:** 256×224px (16×14 tiles)  
**Traversal:** Vertical + horizontal, ladder-centric  
**Scrolling:** Page-based at screen boundaries  
**Checkpoints:** 1 per 2 screens

### 6.2 Themes

- **Snow Caves:** Icy blue, stalactites, slippery floors
- **Haunted Fort:** Dark stone, collapsing platforms, spike pits
- **Lava Workshop:** Industrial, lava pits, conveyor belts, flame jets

### 6.3 Difficulty Curve

**Level 1-1 (Tutorial):**
- 5-8 enemies (Ground Patrol only), no hazards, 3 easy presents, 4 screens

**Level 1-2 (Escalation):**
- 10-12 enemies (Patrol + Jumper), telegraphed spikes, 4 presents, 5 screens

**Level 1-3 (Challenge):**
- 12-15 enemies (all except Reindeer), multiple hazards, 5 presents, 6 screens

**Final Level (Gauntlet):**
- 18-20 enemies (all types + Reindeer), lava/flames/conveyors, 5 presents, 8 screens

### 6.4 Design Principles

- **Golden Path:** Always provide main route
- **Risk-Reward:** High-value items in dangerous areas
- **Multiple Routes:** ≥2 paths when possible
- **Visibility:** See threats before committing
- **Pacing:** Alternate intense/breather sections
- **Secrets:** Hidden areas for stars/cookies
- **Ladder Networks:** Key navigation element

---

## 7. ART & ANIMATION

### 7.1 Sprite Count

- **Player:** 26 unique sprites (all animations)
- **Enemies:** 18 unique sprites
- **Collectibles:** 8 unique sprites
- **Tiles:** 84 unique (24 per theme + 12 universal)

**Total:** 136 unique sprites

### 7.2 Animation Timing

- **Idle:** 30-60 frames/cycle (breathing)
- **Walk:** 6-8 frames/sprite (NES standard)
- **Run:** 4-6 frames/sprite (faster)
- **Attack:** 4 frames/sprite (crisp)
- **Sparkle:** 15-20 frames/cycle (eye-catching)

### 7.3 Particle Effects

- **Enemy Death:** 8 particles, radial, 30 frame life, fade
- **Evil Santa:** 8 trail particles, 0.5px/frame decay, purple
- **Projectile Impact:** 4 particles, small burst, 15 frames
- **Damage Spark:** 6 particles, radial, white/yellow, 12 frames
- **Collect Sparkle:** 12 particles, upward drift, gold, 30 frames

### 7.4 HUD

```
┌─────────────────────────────────────────────────┐
│ ♥♥♥ SCORE:000000 PRESENTS:0/5 TIME: 5:00 LIVES:3│
└─────────────────────────────────────────────────┘
```

- **Hearts:** 8×8px each, red filled/half/empty
- **Score:** 6-digit zero-padded
- **Present Counter:** Current/Total
- **Timer:** MM:SS, flashes red < 30s
- **Lives:** Simple number
- **Font:** NES pixel (5×7px/char)

---

## 8. AUDIO DESIGN

### 8.1 System

**Channels:** 4 (NES APU emulation)
- **Ch1:** Pulse (melody)
- **Ch2:** Pulse (harmony)
- **Ch3:** Triangle (bass)
- **Ch4:** Noise (percussion/FX)

**Sample Rate:** 48kHz output  
**Priority:** SFX can interrupt Ch1-2, never Ch3-4

### 8.2 Music Tracks

- **Title Theme:** Cheerful, 32-bar loop (60s)
- **Level Theme 1 (Snow):** Mysterious, icy, 48-bar (90s)
- **Level Theme 2 (Fort):** Eerie, urgent, 48-bar (90s)
- **Level Theme 3 (Lava):** Intense, mechanical, 48-bar (90s)
- **Evil Santa:** Pitch shift (-2 semitones, +5% tempo)
- **Level Complete:** 8-bar victory jingle (15s)
- **Game Over:** 4-bar sad melody (10s)

**Total:** 7 tracks  
**Tempo:** 120-140 BPM

### 8.3 SFX

**Player:**
- **Jump:** Rising sweep C4→G4 (6f)
- **Land:** Thud 50Hz (3f)
- **Attack:** Whoosh F5→C5 (8f)
- **Damage:** Harsh noise (15f)
- **Death:** Descending arpeggio E4→F3 (60f)
- **Collect:** Present=C5→G5 (12f), Cookie=E5 (4f), Star=E5→C6 (18f)
- **Evil Santa:** Activate=C2→Ab2 (10f), Expire=Descending chime

**Enemy:**
- **Hit:** Noise 80Hz (4f)
- **Defeat:** Explosion (20f)
- **Snowball:** Puff 50Hz (6f)
- **Reindeer:** Ominous drone (30f)

**Environment:**
- **Timer Beep:** 1kHz (3f) at <10s
- **Checkpoint:** C5→G5 (8f)
- **Door:** Mechanical noise (30f)
- **Victory:** 4-bar fanfare

---

## 9. TECHNICAL ARCHITECTURE

### 9.1 Engine

**Language:** Rust → WebAssembly  
**Framework:** Turbo game engine  
**Build:** WASM for browsers

### 9.2 Game Loop

```rust
fn main_loop():
    fixed_timestep = 1.0/60.0 # 16.67ms
    accumulator = 0.0
    loop:
        delta = get_time() - last_time
        accumulator += delta
        while accumulator >= fixed_timestep:
            process_input()
            update_game_state(fixed_timestep)
            accumulator -= fixed_timestep
        render()
```

### 9.3 Entity-Component System

```rust
struct Entity {
    id: EntityId,
    components: HashMap<Type, Box<Component>>
}

// Components
Transform { x, y, vx, vy }
Sprite { texture_id, frame, animation }
Hitbox { width, height, offset_x, offset_y }
Health { current, max, invuln_frames }
AI { state, state_timer, fsm }
```

### 9.4 Render Pipeline

- **Layer 0:** Background tiles
- **Layer 1:** Collectibles
- **Layer 2:** Player
- **Layer 3:** Enemies
- **Layer 4:** Projectiles
- **Layer 5:** Particles
- **Layer 6:** HUD

**Sprite batching:** Sort by layer, then texture ID

### 9.5 Input System

**Mapping:**
- **Arrow/WASD:** Move
- **Z/Space:** Jump
- **X/Shift:** Attack
- **Enter:** Pause
- **ESC:** Title

**Buffer:** 3 frames for jump/attack (prevents missed inputs)

### 9.6 Save System

**Method:** Browser LocalStorage  
**Triggers:** Level complete, manual save

**Saved Data:**
- Current level
- High score
- Play time
- Best times
- Settings

```json
{
  "version": 1,
  "current_level": "1-2",
  "high_score": 45000,
  "level_times": {"1-1": 125, "1-2": 142},
  "settings": {"music_vol": 0.8, "sfx_vol": 1.0}
}
```

---

## 10. DEVELOPMENT & TESTING

### 10.1 6-Day Schedule

**Day 1: Core Systems**
- Setup, game loop, rendering, player movement, collision

**Day 2: Player & Combat**
- Animations, attack system, climbing, health/damage

**Day 3: Enemies & AI**
- All 5 enemy types, FSMs, spawning, combat loop

**Day 4: Levels & Content**
- Level format/loader, 3 levels, collectibles, Evil Santa, transitions

**Day 5: Polish & Systems**
- UI/HUD, scoring, lives, timer, particles, menus

**Day 6: Audio & Final**
- Music, SFX, balance testing, bug fixes, optimization

### 10.2 Testing

**Unit Tests:**
- Physics calcs, fixed-point math, input buffer, collision, AI

**Integration Tests:**
- Player+enemy, projectiles, level loading, scoring, save/load

**Playtesting:**
- Animations correct, jump feels good, attack not spam/slow
- Enemy patterns readable/fair, hitboxes accurate, difficulty appropriate
- 60 FPS maintained, no bugs/softlocks

### 10.3 Performance

- **Frame Time:** <16.67ms (60 FPS min)
- **Input Latency:** <50ms (3 frames)
- **Memory:** <16MB WASM heap
- **Load:** Initial <2s, Level <500ms

---

## 11. SCOPE MANAGEMENT

### 11.1 Core (Must Ship)

- Player with all mechanics
- All 5 enemies with AI
- Attack + projectiles
- 3 complete levels
- Health/damage
- Collectibles (presents/cookies)
- Evil Santa
- Scoring
- Lives/continue
- Timer + Reindeer
- HUD/pause
- Basic music + SFX
- Save/load LocalStorage

### 11.2 Expanded (If Time)

- 5 total levels
- Combo multiplier
- Stars + secrets
- Level select
- Multiple music tracks
- Additional enemy variants
- Effect polish
- Screen shake

### 11.3 Stretch (Low Priority)

- Boss encounter
- Bonus stages
- Leaderboard
- CRT shader
- Gamepad support
- Mobile touch
- New playable characters
- New Game+

---

## 12. APPENDICES

### 12.1 Controls

```
┌──────────────────────────────────────┐
│ KEYBOARD CONTROLS                    │
├──────────────────────────────────────┤
│ Arrow/WASD       : Move              │
│ UP/W (ladder)    : Climb Up          │
│ DOWN/S (ladder)  : Climb Down        │
│ Z/Space          : Jump              │
│ X/Shift          : Attack            │
│ Enter            : Pause             │
│ ESC (pause)      : Quit to Title     │
└──────────────────────────────────────┘
```

### 12.2 Scoring Reference

```
┌────────────────────────────┬────────┐
│ ACTION                     │ POINTS │
├────────────────────────────┼────────┤
│ Gingerbread Soldier        │    100 │
│ Nutcracker Jumper          │    150 │
│ Elf Climber                │    100 │
│ Snowman Turret             │    200 │
│ Reindeer Stalker           │    500 │
│ Evil Santa Kill            │    100 │
│ Present / Cookie / Star    │250/50/100│
│ Perfect Bonus              │  1,000 │
│ Time Bonus (per second)    │     10 │
│ Extra Life                 │ 10,000 │
└────────────────────────────┴────────┘
```

### 12.3 Tech Specs Summary

```
┌──────────────────────────────────────┐
│ Resolution     : 256 × 240 pixels    │
│ Visible        : 256 × 224 pixels    │
│ Frame Rate     : 60 FPS (fixed)      │
│ Tile Size      : 16 × 16 pixels      │
│ Player Sprite  : 32 × 32 pixels      │
│ Player Hitbox  : 14 × 28 pixels      │
│ Audio          : 4 channels (NES APU)│
│ Coordinates    : 16-bit fixed (8.8)  │
│ Engine         : Turbo (Rust/WASM)   │
│ Max Enemies    : 8 per screen        │
│ Max Projectiles: 2 player, 4 enemy   │
└──────────────────────────────────────┘
```

### 12.4 Credits

**Game Design:** [Team Name]  
**Programming:** [Developers]  
**Art:** [Artists]  
**Audio:** [Sound Designers]  
**Inspiration:** The Goonies (1986, Konami), Castlevania, Mega Man, Metroid  
**Engine:** Turbo (Rust game engine)

### 12.5 Revision History

**Version 1.0 - December 2024**  
Status: Approved for Implementation  
Production-Ready PRD/GDD

---

## END OF DOCUMENT

Production-ready and implementation-complete.  
All technical specs, game systems, and design details documented for immediate development.