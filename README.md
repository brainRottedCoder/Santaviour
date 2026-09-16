# 🎅 Santa's Rescue

<div align="center">

**A precision 2D platformer - Save Christmas, One Present at a Time!**

[![Turbo Genesis](https://img.shields.io/badge/Turbo-5.2.0-red?style=for-the-badge)](https://turbo.computer)
[![Rust Edition](https://img.shields.io/badge/Rust-2021-orange?style=for-the-badge)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

</div>

---

## 📖 Overview

**Santa's Rescue** is an action-packed 2D platformer built with the Turbo Genesis SDK and Rust. Play as Santa on a mission to rescue kidnapped children and defeat the Evil Santa in an epic boss battle to save Christmas!

Navigate through three challenging levels filled with enemies, obstacles, puzzles, and collectibles. Master Santa's abilities including jumping, climbing, crouching, and combat to overcome increasingly difficult challenges.

---

## ✨ Features

### 🎮 Gameplay Mechanics
- **8-Direction Movement**: Smooth platforming with walk, run, jump, climb, and crouch mechanics
- **Advanced Physics**: Realistic gravity, coyote time, jump buffering, and variable jump height
- **Combat System**: Attack enemies with projectiles and place gift bombs strategically
- **Collectibles**: Gather gift bombs, keys, and rescue children throughout levels
- **Power-ups**: Life hearts (increase max HP) and firepower upgrades

### 🗺️ Three Unique Levels
1. **Level 1**: Christmas House (3-floor layout) - 3 minutes time limit
2. **Level 2**: Expanded Christmas Mansion - 8 minutes time limit
3. **Level 3**: Boss Fight Arena - 10 minutes time limit

### 👹 Enemy Types
- **Kickmouse**: Patrolling ground enemy
- **LongEar**: Faster enemy variant
- **Penguin**: Aggressive enemy with unique patterns

### 🎯 Boss Fight
Epic showdown with **Evil Santa** featuring:
- AI-driven attack patterns (Dash, Slam, Projectile)
- Multi-phase combat system
- Dynamic difficulty based on player behavior
- Cinematic arena with custom 48x48 boss sprites

### 🎨 Visual & Audio
- **Pixel-Perfect Graphics**: Custom sprite animations for all characters
- **Multiple Backgrounds**: Level-specific backdrops including boss arena
- **Sound Effects**: 16 unique audio files including:
  - BGM (background music)
  - Jump, attack, and hurt sounds
  - Collection and explosion effects
  - Boss battle audio cues
- **Animated Elements**: Keys, kids, enemies, and explosive effects

### 🔧 Developer Tools
- **Dev Mode** (Press `.` to toggle):
  - Direct level selection (`J`=Level 1, `K`=Level 2, `B`=Boss Level)
  - Level cycling (`L`=Next, `H`=Previous)
  - Debug overlays and collision visualization

---

## 🎯 Game Objectives

### Primary Goals
1. **Rescue Children**: Find and collect kids hidden throughout levels
2. **Collect Keys**: Destroy doors with gift bombs to obtain keys
3. **Unlock Gates**: Use keys to open gates blocking progress
4. **Defeat Evil Santa**: Final boss battle in Level 3
5. **Beat the Clock**: Complete levels before time runs out

### Progression System
- **3 Lives**: Respawn at level start when defeated
- **Health System**: 6 HP starting health (expandable with power-ups)
- **Invulnerability Frames**: 2-3 seconds after taking damage or respawning
- **Score Tracking**: Points for defeating enemies and collecting items

---

## 🕹️ Controls

### Keyboard Controls
| Key | Action |
|-----|--------|
| **Arrow Keys** | Move Left/Right/Up/Down |
| **X** | Jump (hold for higher jump) |
| **Z** | Attack (shoot projectile) |
| **C** | Place Gift Bomb |
| **S** | Toggle Controls Panel |
| **Enter** | Start Game (from menu) |

### Movement Mechanics
- **Climbing**: Press Up/Down near ladders to climb
- **Crouching**: Hold Down while on ground
- **Ladder Jump**: Press X + Left/Right while climbing to jump off

### Developer Controls (Dev Mode - Press `.`)
| Key | Action |
|-----|--------|
| **J** | Jump to Level 1 |
| **K** | Jump to Level 2 |
| **B** | Jump to Boss Level |
| **L** | Next Level |
| **H** | Previous Level |

---

## 🏗️ Technical Specifications

### Built With
- **Language**: Rust (Edition 2021)
- **Engine**: Turbo Genesis SDK v5.2.0
- **Canvas**: 360x240 pixels
- **Frame Rate**: 60 FPS

### Physics Constants
```rust
GRAVITY: 0.5
TERMINAL_VELOCITY: 6.0
JUMP_VELOCITY: -6.5
WALK_SPEED: 1.5
AIR_CONTROL: 0.5
CLIMB_SPEED: 1.5
```

### Game State
The game maintains comprehensive state including:
- Player position, velocity, health, and animations
- Enemy states and respawn timers
- Projectile tracking (up to 6 simultaneous)
- Door, key, and collectible states
- Boss fight progression
- Level timer and completion flags

---

## 📦 Project Structure

```
gamem/
├── Cargo.toml              # Rust project configuration
├── turbo.toml              # Turbo Genesis configuration
├── src/
│   └── lib.rs              # Main game logic (3886 lines)
├── Sprites/                # Game sprites and assets
│   ├── Santa/              # Player character sprites
│   ├── enemy/              # Enemy sprites
│   ├── bossfight/          # Boss fight sprites
│   ├── firepower/          # VFX sprites
│   ├── gift bomb/          # Bomb sprites
│   ├── key/                # Key animations
│   ├── kid/                # Kid sprites
│   └── cloud/              # Respawn cloud effects
├── audio/                  # 16 sound files
│   ├── bgm.mp3             # Background music
│   ├── jump.mp3            # Jump sound
│   ├── collection.ogg      # Collection sound
│   ├── explosion.wav       # Bomb explosion
│   ├── santa_death.wav     # Death sound
│   └── ...                 # Additional SFX
├── api/                    # Serverless routes (accounts, rooms, runs, admin)
│   ├── _lib/               # Shared storage, session, ranking, admin helpers
│   ├── auth/ rooms/ runs/  # Player-facing endpoints
│   └── admin/[...path].js  # Whole admin API in one function
├── scripts/                # Seed, tests, local dev server
└── www/                    # Web build files
    ├── index.html
    ├── admin.html          # Admin panel (password gated)
    ├── main.js
    ├── style.css
    ├── app/                # Overlay + admin UI (survives `turbo export`)
    └── pkg/                # WASM build output
```

---

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Turbo CLI](https://turbo.computer/docs/getting-started)

### Installation

1. **Clone the repository**
```bash
git clone <repository-url>
cd gamem
```

2. **Install Turbo CLI** (if not already installed)
```bash
# Windows (PowerShell)
iwr https://releases.turbo.computer/install.ps1 -useb | iex

# macOS/Linux
curl -fsSL https://releases.turbo.computer/install.sh | sh
```

3. **Build and run the game**
```bash
# Live development (game only)
turbo run

# Web export (regenerates www/, including main.turbo)
turbo export
```

The game will open in your default web browser.

After `turbo export`, restore the overlay hook if it was overwritten — add this line to `www/index.html` before `main.js`:

```html
<script type="module" src="./app/ui.js"></script>
```

See `www/app/README.md`.

4. **Rooms, scores, and ranks** (shared across devices)

Copy `.env.example` to `.env.local` and set `SESSION_SECRET`. For the hosted site, also add Upstash Redis (`UPSTASH_REDIS_REST_URL`, `UPSTASH_REDIS_REST_TOKEN`) in the Vercel project so every device shares one leaderboard. Without those Redis variables, `npx vercel dev` uses a local `.data/store.json` file (this computer only).

```bash
npm install
npx vercel dev        # or: npm run dev  (no Vercel account needed)
npm run seed
```

`npm run dev` starts `scripts/dev-server.mjs`, a small Node server that serves `www/` and dispatches `/api/*` to the same handlers Vercel uses. Set `PORT` to move it off 3000.

Demo room code `DEMO01`. Sample logins (password `DemoPass1`): `SnowMaster`, `Rudolph`, `ElfRunner`. `npm run seed` also loads 10 extra demo players (`HollyDash`, `CocoaKing`, `FrostByte`, `NorthStar`, `SleighGirl`, `GiftNinja`, `IvyLeap`, `JingleBot`, `CoalMiner`, `QuietMouse`) across rooms `WARM4`, `NIGHT1`, `GIFT2`, and `COAL3` so the admin panel has a full board to inspect. Create your own account and join `DEMO01` as the fourth player.

5. **Admin panel**

Set `ADMIN_PASSWORD` (8+ characters) in `.env.local`, and in the Vercel project for the hosted site.

From the game, press **Ctrl+Shift+A** (Mac: **Cmd+Shift+A**). That opens `/admin` — there is no button in the overlay, so players do not see it. You can also bookmark:

```
http://localhost:3000/admin           # local
https://<your-site>/admin             # hosted
```

The panel asks for that password and nothing else — it is deliberately separate from player accounts, so no game login can ever reach it. Leaving `ADMIN_PASSWORD` unset disables the panel: the page loads but every route answers `503`.

It shows every registered username, their global ranking across all rooms, their standing in each individual room, and their full run history. It can also delete a player, delete a room, reset a room's scoreboard, or clear a single run.

---

## 🎮 How to Play

### Starting Out
1. Launch the game - you'll start directly in Level 1
2. Use Arrow Keys to move Santa around
3. Press X to jump and Z to attack enemies
4. Rescue children and collect keys by destroying doors with gift bombs

### Level Progression
- **Level 1**: Learn the basics in a 3-floor Christmas house
  - Collect gift bombs scattered around
  - Find 3 keys by destroying doors
  - Rescue children before time runs out
  
- **Level 2**: Expanded mansion with more complex platforming
  - Navigate longer distances between platforms
  - More enemies and hazards
  - 8-minute time limit
  
- **Level 3**: Epic boss battle
  - Defeat Evil Santa in an arena showdown
  - Dodge multiple attack patterns
  - Final chance to collect power-ups
  - Rescue the final child to win!

### Tips & Strategies
- **Gift Bombs**: Use wisely - limited supply!
- **Timing**: Master jump timing for longer horizontal distances
- **Combat**: Attack from range to avoid enemy contact damage
- **Exploration**: Check every platform for collectibles and power-ups
- **Boss Fight**: Learn attack patterns and maintain distance

---

## 🎨 Game Assets

### Sprite Sheets
- **Santa**: 28x32 sprites (normal), 48x48 (boss fight)
- **Evil Santa**: 48x48 sprites with 6-frame attack animations
- **Enemies**: Various sizes with walk/attack animations
- **Items**: 64x64 gift bombs, animated keys (6 frames)

### Audio Files
- **Background**: `bgm.mp3` (looping)
- **Player SFX**: Jump, hurt (2 variants), death, kill (2 variants)
- **Game Events**: Collection, explosion, completion
- **Boss SFX**: Evil Santa hurt, attack warnings (3 variants)
- **Story**: Kid meeting sounds (2 variants)

---

## 🏆 Achievements

Complete these challenges:
- ✅ Beat Level 1 without losing a life
- ✅ Collect all keys in Level 2
- ✅ Rescue all children across all levels
- ✅ Defeat Evil Santa without taking damage
- ✅ Complete the game in under 15 minutes total
- ✅ Beat the game with maximum HP

---

## 🐛 Known Features

### Game Mechanics
- Coyote time (4 frames) allows jumps shortly after leaving a platform
- Jump buffering (3 frames) registers jump inputs slightly early
- Enemy respawn system with cloud animations (10-second timer)
- Invulnerability flashing effect after taking damage

### Level Design
- Gates automatically open when required keys are collected
- Boss arena locks camera for cinematic combat
- Platform collision uses NES-style hit detection
- Death zones below all platforms for instant death

---

## 🛠️ Development

### Building from Source

```bash
# Install dependencies
cargo build

# Run with Turbo
turbo run

# Export web build (rewrites www/; keep www/app/)
turbo export
```

### Code Structure
- **State Management**: Single `GameState` struct with all game data
- **Update Loop**: 60 FPS game loop with input, physics, and rendering
- **Collision Detection**: Separate systems for platforms, walls, and entities
- **Animation System**: Frame-based sprite animations for all characters
- **Audio System**: Turbo audio API with looping BGM and one-shot SFX

---

## 📝 Credits

### Development Team
- **Team**: Santa Rescue Team
- **Engine**: [Turbo Genesis SDK](https://turbo.computer)
- **Language**: Rust Programming Language

### Special Thanks
- Turbo community for the amazing SDK
- All playtesters and contributors

---

## 📄 License

This project is created for educational and entertainment purposes.

---

## 🔗 Links

- [Turbo Genesis Documentation](https://turbo.computer/docs)
- [Rust Documentation](https://doc.rust-lang.org/)

---

<div align="center">

**Save Christmas, One Present at a Time! 🎄**

Made with ❤️ using Rust and Turbo Genesis

</div>
