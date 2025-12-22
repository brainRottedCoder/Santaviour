# Neuro Santa – Gameplay & Technical Guide

This README is an implementation-accurate guide to the current Neuro Santa game. It explains how to play, every key binding, core mechanics, collectibles, enemies, boss fight, scoring, gates/keys, bombs/doors, UI/HUD, touch and gamepad support, and developer hotkeys. It reflects the actual Rust/WebAssembly logic and the web integration used by the game.

## Overview
- Retro arcade platformer built with the Turbo Engine (Rust → WebAssembly) for browsers.
- Tight, readable controls and predictable enemy patterns for fair challenge.
- Three levels: two platforming stages and a boss arena against Evil Santa.
- NES-inspired resolution (~360×240 logical), sprite-centric visuals, and a simple HUD.

## How to Start
- Open the web build (served via `index.html` + `main.js`) in a modern browser.
- Start screen shows a splash image and prompt.
- Press `Enter` or `Space` to start.

## Controls
Keyboard (primary):
- Move: `Arrow Left/Right`
- Jump: `X` (variable jump and buffering; see Mechanics)
- Attack: `Z` (5-frame attack; projectile spawns at frame 4)
- Place Bomb: `C` (if you have a gift bomb and are on ground)
- Crouch: `Arrow Down` (slow crawl on ground)
- Climb: `Arrow Up/Down` while overlapping a ladder
- Toggle Controls Panel: `S`
- Restart Level (on Game Over or when useful): `Enter`

Developer hotkeys:
- Toggle Developer Mode: `.` period
- Jump to Level: `J` = Level 1, `K` = Level 2, `B` = Level 3
- Cycle Level: `L` = next, `H` = previous

Gamepad (standard controllers):
- D‑pad: mapped to Arrow keys.
- `A` → `Z` (Attack)
- `B` → `X` (Jump)
- `X` → `C` (Place Bomb)
- `Y` → `V` (currently unused by the game logic)
- `Select` → `Enter` (Start game / restart level)
- `Start` → `Space` (Start game)
- Left analog stick: dispatches Arrow key presses when moved past a threshold.

Touch (mobile/tablet):
- A virtual gamepad overlay renders on top of the canvas.
- D‑pad and buttons follow the mapping above (A=Attack/Z, B=Jump/X, X=Bomb/C, Y=V unused, Start=Space, Select=Enter).
- Gliding over the D‑pad supports diagonals; buttons have sensible press/release handling.

## HUD & UI
- `HP` bar: proportional to current HP vs max HP.
- `LIVES`: heart icons for remaining lives.
- `SCORE`: total score.
- `KEYS`: keys collected in current level.
- `KIDS`: rescued kids vs total spawned in the level.
- `BOMBS`: number of gift bombs currently held.
- `TIME`: level countdown (seconds). Under 30 seconds, the timer turns red. If time expires, Santa dies.
- Controls Panel: Press `S` to toggle a helpful overlay with a concise control list and notes.

## Core Mechanics
Movement & physics:
- Horizontal: acceleration/deceleration with clamped walk speed.
  - `WALK_SPEED = 1.5`, `ACCEL = 0.2`, `DECEL = 0.4`.
- Gravity and fall: `GRAVITY = 0.5`, `TERMINAL_VEL = 6.0`.
- Jump: `JUMP_VEL = -6.5`.
- Variable jump height: releasing `X` early reduces jump height.
- Jump buffering: `INPUT_BUFFER = 3` frames; a jump performs if pressed within the buffer.
- Coyote time: `COYOTE_TIME = 4` frames; you can still jump just after leaving ground.
- Air control: extended horizontal momentum during jumps (`AIR_CONTROL = 0.5`) with a boost applied when holding left/right; jump length multiplier used to extend travel.

Ladders:
- Press `Up` or `Down` near a ladder to enter climb state.
- `Up/Down` move on the ladder; gravity is disabled during climbing.
- Jump off ladder: press `X` + `Left/Right` for a small jump.
- Automatically exit ladder if you step onto solid ground or leave overlap.

Crouch:
- Press `Down` while on ground to crouch and move slowly.

Attack:
- Press `Z` to attack. Attack lasts 20 frames (5 sprites × 4 frames each).
- Projectile spawns at attack frame 4.
- Attack cooldown: 24 frames.
- With `PowerUp1` or in boss sprite mode, attack shows enhanced VFX. Damage remains the same.

Health, lives, invulnerability:
- Starting HP: 6; max HP grows via Life powerup (capped at 99).
- On enemy hit: take damage (scales as 10% of max HP, minimum 1 HP) with short invulnerability.
- On bomb explosion: take 2 damage if inside radius; brief invulnerability afterward.
- Lives: start with 3. If HP hits 0, lose a life and respawn; with no lives, transition to Game Over.

## Collectibles & Scoring
Keys:
- Spawn from destroyed doors (see Bombs & Doors) and animate in place.
- Pickup adds `+100` score and increments `KEYS`.

Kids (rescue):
- Certain doors spawn kids instead of keys in non-boss levels.
- Walking over a kid rescues them, adds `+500` score, shows “KID SAVED!”, and restores `+1 HP` if not at max.

Gift Bombs (items):
- Dropped by enemies and placed in the world.
- Pickup adds `+50` score and sets `BOMBS = 1` (you can only carry one at a time).

PowerUp1 (firepower):
- Spawns from a specific door in Level 3.
- Collecting grants attack VFX upgrade and adds `+750` score.

Life Powerup:
- Spawns from a specific door in Level 3.
- Increases `max HP` by 3 (capped at 99) and heals `+3 HP` immediately.
- Adds `+1000` score.

Enemy kills:
- Normal projectile kill: `+100` score.
- Bomb explosion kill: `+150` score.

Doors destroyed:
- Destroying a door with a bomb: `+200` score.

## Bombs & Doors (Destruction Loop)
Carrying and placing bombs:
- You can carry at most one gift bomb (`BOMBS: 0 or 1`).
- Place a bomb with `C` while standing on ground.
- Bomb animation warns for ~5 seconds, then explodes.

Explosion effects (radius ~50 px horizontally and vertically):
- Damages Santa: `-2 HP` if inside the radius (with invulnerability afterward).
- Kills enemies: starts their respawn timer (`~10 seconds`), awards `+150` per kill.
- Destroys doors: awards `+200` and spawns a key/kid/powerup depending on level/door.

Door spawn rules:
- Level 1 & 2: one door (random index per level) spawns a kid; all other destroyed doors spawn keys.
- Level 3 (boss level):
  - Door 0 and Door 1 spawn keys that are used to open the gates near the end ladder.
  - Door 2 spawns the Life powerup.
  - Door 3 spawns `PowerUp1` (firepower).

## Keys → Gates → Level Completion
- Level requirement: collect enough keys to open the path and finish.
  - Levels 1 & 2: gates open when you have `3` keys.
  - Level 3 (outside boss fight): gate segments open progressively: `1` key opens Gate #0, `2` keys open Gate #1.
- Completion trigger: enter the final zone once you meet `required_keys`.
- Transition: displays “LEVEL COMPLETE!” and starts a 2‑second transition to the next level.

## Enemies
Types (as implemented):
- Mouse (`type 1`): patrols platforms.
- Kickmouse (`type 2`): patrols and performs short attacks when close.
- Penguin (`type 3`): patrols; broader horizontal attack trigger; faces Santa when attacking.

Behavior:
- Simple patrol along defined ranges; flip direction at ends or at collisions.
- Attack triggers when Santa is within a small trigger zone (penguins use a wider horizontal range).
- After a brief attack animation (3 frames), enemies resume patrolling.
- When killed, enemies respawn after ~10 seconds with a cloud animation.

## Boss Fight – Evil Santa (Level 3)
Trigger & setup:
- Boss activates in Level 3 after Santa reaches the arena area.
- Sets up a fixed camera, corner walls, and switches to boss sprite sets.
- Other enemies are disabled during the fight.

Boss stats & UI:
- HP bar shows current vs max HP (max HP = 8).
- Takes damage from Santa’s projectiles; flashes briefly on hit.

States & attacks:
- Patterned state machine with idle, walk, and attack phases.
- Attack types:
  - Dash: fast horizontal rush; distance scales with difficulty.
  - Slam: jump then ground impact near Santa.
  - Projectile: multiple projectiles with windup/telegraph.
- Phase difficulty ramps as HP drops (`enraged` below 50%, `desperate` below 25%)—timings speed up and combos increase.
- Anti‑stall: if Santa idles too long, boss forces an attack.

Victory & death:
- When Evil Santa’s HP reaches 0, he enters a death sequence; the arena resets afterward.

## Game States
- `START_MENU`: splash screen. Press `Enter` or `Space` to start.
- `PLAYING`: normal gameplay.
- `GAME_OVER`: shows final score and “Press ENTER to restart”.

## Touch/Gamepad Implementation Details
- The web layer provides a virtual touch gamepad that overrides `navigator.getGamepads` to present itself as a standard controller.
- The overlay buttons dispatch synthetic keyboard events, so the Rust game logic sees consistent input across keyboard, gamepad, and touch.
- Analog stick movement dispatches Arrow key presses when crossing a threshold.
- Double‑tap zoom is disabled to avoid accidental zoom while tapping.

## Technical Notes
- Engine: Turbo Engine targeting WebAssembly.
- Rendering/UI: sprite and text primitives; fixed logical resolution.
- Input: unified keyboard/gamepad/touch via web shim (`main.js`) translating to keyboard events.
- Assets: sprites for Santa, enemies (mouse, kickmouse, penguin), boss, doors, keys, kids, gift bombs, and powerups.

## Running Locally
- Serve the folder with any static web server so the browser can load WebAssembly and assets.
- Open `index.html` in your browser.
- Use a modern desktop or mobile browser; gamepad and touch overlays are supported.

## Developer Tips
- Toggle the Controls Panel with `S` to see a concise reminder of inputs.
- Use Developer Mode (`.`) to hop between levels quickly with `J`, `K`, `B` or cycle via `L`/`H`.
- During testing, watch the HUD for `KEYS`, `KIDS`, and `BOMBS` to understand level progress and available tools.

## Notes vs Original Concept
The original design document mentioned additional enemies and collectibles (e.g., cookies, stars). The current build implements `keys`, `kids`, `gift bombs`, `life`, and `powerUp1`, plus three enemy types (mouse, kickmouse, penguin) and a full Evil Santa boss encounter. This README prioritizes what is implemented in code today.

Enjoy, and happy holidays!