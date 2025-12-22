use turbo::*;

// ============================================================================
// CONSTANTS - PLAYER STATES
// ============================================================================
const STATE_IDLE: u8 = 0;
const STATE_WALK: u8 = 1;
const STATE_RUN: u8 = 2;
const STATE_JUMP: u8 = 3;
const STATE_FALL: u8 = 4;
const STATE_CLIMB: u8 = 5;
const STATE_ATTACK: u8 = 6;
const STATE_HURT: u8 = 7;
const STATE_DEAD: u8 = 8;
const STATE_CROUCH: u8 = 9;

// ============================================================================
// CONSTANTS - PHYSICS
// ============================================================================
const GRAVITY: f32 = 0.5;  // Normal gravity (keeps jump height the same)
const TERMINAL_VEL: f32 = 6.0;
const JUMP_VEL: f32 = -6.5;  // Reduced for lower/same jump height
const WALK_SPEED: f32 = 1.5;  // Increased for more horizontal momentum
const RUN_SPEED: f32 = 1.5;  // Increased from 2.5 for faster running
const ACCEL: f32 = 0.2;
const DECEL: f32 = 0.4;
const AIR_CONTROL: f32 = 0.5;  // Significantly increased for longer horizontal jumps
const JUMP_LENGTH_MULTIPLIER: f32 = 1.8;  // Control jump distance (1.0 = normal, 2.0 = double)
const CLIMB_SPEED: f32 = 1.5;
const CROUCH_SPEED: f32 = 0.75;

const COYOTE_TIME: u8 = 4;
const INPUT_BUFFER: u8 = 3;

// Animation frame counts
const IDLE_FRAMES: u8 = 4;
const RUN_FRAMES: u8 = 10;
const JUMP_FRAMES: u8 = 5;
const ATTACK_FRAMES: u8 = 5;
// Boss animation constants
const EVIL_IDLE_FRAMES: u8 = 4;
const EVIL_WALK_FRAMES: u8 = 10;
const EVIL_JUMP_FRAMES: u8 = 6;
const EVIL_ATTACK_FRAMES: u8 = 6;

// Boss tuning
const EVIL_WALK_SPEED: f32 = 1.2;
const EVIL_JUMP_VEL: f32 = -5.5;
const EVIL_ATTACK_RANGE: f32 = 40.0;

// Boss fight arena settings (Level 3)
const BOSS_TRIGGER_X: f32 = 1860.0;
const BOSS_TRIGGER_Y: f32 = 164.0;
const BOSS_ARENA_WIDTH: f32 = 360.0; // matches bossfight background width
const BOSS_EVIL_SANTA_X: f32 = 280.0; // Adjusted for new arena width
const BOSS_EVIL_SANTA_Y: f32 = 190.0; // Adjusted for platform at y=217

// Boss entity state (flattened into GameState to avoid serialization derives)
const BOSS_STATE_IDLE: u8 = 10;
const BOSS_STATE_SELECT: u8 = 11;
const BOSS_STATE_ATTACK: u8 = 12;
const BOSS_STATE_RECOVER: u8 = 13;
const BOSS_STATE_DEAD: u8 = 14;
const BOSS_STATE_JUMP: u8 = 15;
const BOSS_STATE_FALL: u8 = 16;
const BOSS_STATE_WALK: u8 = 17; // Start walk

const ATTACK_DASH: u8 = 0;
const ATTACK_SLAM: u8 = 1;
const ATTACK_PROJECTILE: u8 = 2;

const PHASE_WINDUP: u8 = 0;
const PHASE_ACTIVE: u8 = 1;
const PHASE_RECOVERY: u8 = 2;
// ============================================================================
// CONSTANTS - GAME
// ============================================================================
const SCREEN_WIDTH: f32 = 360.0;
const SCREEN_HEIGHT: f32 = 240.0;
const HUD_HEIGHT: f32 = 16.0;

const LEVEL_TIME: u32 = 180 * 60; // 60 minutes (3600 seconds)

// ============================================================================
// MAIN GAME STATE
// ============================================================================
    #[turbo::game]
    struct GameState {
    frame: u32,

    // Player state
    player_x: f32,
    player_y: f32,
    player_vx: f32,
    player_vy: f32,
    player_state: u8,
    player_facing_right: bool,
    player_on_ground: bool,
    player_hp: u8,
    player_max_hp: u8,
    player_invuln_timer: u8,
    player_anim_frame: u8,
    player_anim_timer: u8,
    player_coyote_timer: u8,
    player_jump_buffer: u8,
    player_on_ladder: bool,
    player_is_crouching: bool,
    jump_anim_frame: u8,
    jump_anim_timer: u8,

    // Attack state
    attack_cooldown: u8,
    attack_frame: u8,

    // Projectiles (up to 6 total)
    projectiles: [(bool, f32, f32, f32, bool, f32); 6],

    // Enemies (10 max for now)
    // Format: (type, x, y, vx, vy, direction, anim_frame, patrol_start_x, active, anim_timer, is_attacking, respawn_timer)
    // respawn_timer: 0 = alive, >0 = dead and counting down to respawn (600 frames = 10 seconds)
    // cloud animation frame calculated from respawn_timer when needed
    enemies: [(u8, f32, f32, f32, f32, u8, u8, u16, bool, u8, bool, u16); 10],

    // Penguin snowball projectiles (up to 5)
    // Format: (active, x, y, vx, vy)
    snowballs: [(bool, f32, f32, f32, f32); 5],

    // Gift Bombs
    gift_bombs: u8,  // Number of gift bombs Santa has
    gift_bomb_items: [(f32, f32, bool, u16); 5],  // Dropped gift bombs to pick up (x, y, active, despawn_timer)
    placed_bombs: [(f32, f32, bool, u8, u8); 3],  // Placed bombs (x, y, active, anim_frame, anim_timer)

    // Doors (4 doors that can be destroyed by bombs)
    // Format: (x, y, intact)
    // x, y are top-left corner coordinates
    // intact: true = normal door, false = exploded
    doors: [(f32, f32, bool); 4],

    // Keys dropped from exploded doors
    // Format: (x, y, active, anim_frame, anim_timer)
    keys: [(f32, f32, bool, u8, u8); 4],
    keys_collected: u8,
    key_pickup_flash: u8,

    // Game state
    score: u32,
    lives: u8,
    level: u8,
    timer: u32,
    camera_x: f32,

    // Developer options
    dev_mode: bool,
    // UI: Controls panel overlay
    show_controls_panel: bool,
    
    // Game state management
    in_menu: bool,  // true = in start menu, false = playing
    game_over_timer: u16,  // Timer for game over screen (10 seconds)
    game_won_timer: u16,  // Timer for victory screen (3 seconds)
    show_game_over: bool,
    show_victory: bool,

    // Level completion
    level_complete: bool,
    level_transition_timer: u8,
    completion_trigger: (f32, f32, f32, f32), // x, y, width, height

    // Level layout - 3 floors with multiple platforms
    // Using Vec to support variable number of platforms (boss level has 52)
    platforms: Vec<(f32, f32, f32)>, // x1, x2, y for each platform
    ladders: [(f32, f32, f32); 6],    // x, y_top, y_bottom for each ladder
    walls: [(f32, f32, f32, f32); 22], // x, y, width, height for each wall

    // Kids to rescue (up to 3 per level)
    // Format: (x, y, active, collected, anim_frame, anim_timer, spawned_from_door_idx)
    kids: [(f32, f32, bool, bool, u8, u8, i8); 3],
    kids_collected: u8,
    
    // Level timer (in frames, 60 fps)
    level_timer: u32,  // Current time elapsed
    level_time_limit: u32,  // Time limit for current level
    total_kids_in_level: u8,
    kid_pickup_flash: u8,
    kid_door_index: usize,  // Which door (0-3) spawns a kid instead of a key

    // Life powerup (spawns from specific door in boss level)
    life_position: (f32, f32),
    life_active: bool,
    life_collected: bool,
    life_door_index: usize,  // Which door spawns life powerup (door #3 in level 3)

    // PowerUp1 (spawns from door #4 in level 3)
    powerup1_position: (f32, f32),
    powerup1_active: bool,
    powerup1_collected: bool,
    has_firepower: bool,  // Granted by powerUp1

    // Boss fight state
        boss_active: bool,
        boss_defeated: bool,  // Prevents boss from re-triggering after victory
        use_boss_santa: bool,
        evil_santa_x: f32,
        evil_santa_y: f32,
        evil_santa_facing_right: bool,
        evil_santa_state: u8,
        evil_santa_anim_frame: u8,
        evil_santa_anim_timer: u8,
        // Boss combat/physics
        evil_santa_hp: u8,
        evil_santa_max_hp: u8,
        evil_santa_vx: f32,
        evil_santa_vy: f32,
        evil_santa_on_ground: bool,
        evil_santa_attack_cooldown: u8,
        evil_santa_attack_frame: u8,
        evil_santa_jump_cooldown: u8,
        // AI FSM
        boss_state_timer: u32,
        boss_pattern_index: u8,
        boss_phase: u8,
        boss_phase_timer: u32,
        boss_attack_type: u8,
        evil_santa_flash_timer: u8,
        // Player idle tracking for boss aggression
        player_idle_x: f32,
        player_idle_timer: u32,
        // Boss death handling
        boss_death_timer: u16,
    }

impl GameState {
    pub fn new() -> Self {
        let mut game = Self {
            frame: 0,

            player_x: 50.0,
            player_y: 100.0,
            player_vx: 0.0,
            player_vy: 0.0,
            player_state: STATE_IDLE,
            player_facing_right: true,
            player_on_ground: false,
            player_hp: 6,
            player_max_hp: 6,
            player_invuln_timer: 0,
            player_anim_frame: 0,
            player_anim_timer: 0,
            player_coyote_timer: 0,
            player_jump_buffer: 0,
            player_on_ladder: false,
            player_is_crouching: false,
            jump_anim_frame: 0,
            jump_anim_timer: 0,

            attack_cooldown: 0,
            attack_frame: 0,

            projectiles: [(false, 0.0, 0.0, 0.0, false, 0.0); 6],
            enemies: [(0, 0.0, 0.0, 0.0, 0.0, 0, 0, 0, false, 0, false, 0); 10],
            snowballs: [(false, 0.0, 0.0, 0.0, 0.0); 5],  // Penguin snowball projectiles

            gift_bombs: 0,
            gift_bomb_items: [(0.0, 0.0, false, 0); 5],
            placed_bombs: [(0.0, 0.0, false, 0, 0); 3],
            doors: [(0.0, 0.0, false); 4],  // Will be set in load_level

            score: 0,
            lives: 3,
            level: 1,
            timer: LEVEL_TIME,
            camera_x: 0.0,

            // Developer options
            dev_mode: false,
            show_controls_panel: false,
            
            in_menu: false,  // No start menu - go directly to game
            show_game_over: false,
            show_victory: false,
            game_over_timer: 0,
            game_won_timer: 0,

            level_complete: false,
            level_transition_timer: 0,
            completion_trigger: (0.0, 0.0, 0.0, 0.0),

            platforms: Vec::new(), // Will be populated in load_level
            ladders: [
                // Initialize empty, will be set in load_level
                (0.0, 0.0, 0.0); 6
            ],
            walls: [
                // Initialize empty, will be set in load_level
                (0.0, 0.0, 0.0, 0.0); 22
            ],

            // Keys state
            keys: [
                (0.0, 0.0, false, 0, 0); 4
            ],
            keys_collected: 0,
            key_pickup_flash: 0,

            // Kids state
            kids: [(0.0, 0.0, false, false, 0, 0, -1); 3],
            kids_collected: 0,
            level_timer: 0,
            level_time_limit: 180 * 60,  // Default 3 minutes
            total_kids_in_level: 0,
            kid_pickup_flash: 0,
            kid_door_index: 0,  // Will be randomized per level

            // Life powerup state
            life_position: (0.0, 0.0),
            life_active: false,
            life_collected: false,
            life_door_index: 2,  // Door #3 (index 2) in level 3

            // PowerUp1 state
            powerup1_position: (0.0, 0.0),
            powerup1_active: false,
            powerup1_collected: false,
            has_firepower: false,

        // Boss fight
        boss_active: false,
        boss_defeated: false,
        use_boss_santa: false,
        evil_santa_x: 0.0,
        evil_santa_y: 0.0,
        evil_santa_facing_right: false,
        evil_santa_state: BOSS_STATE_IDLE,
        evil_santa_anim_frame: 0,
        evil_santa_anim_timer: 0,
        evil_santa_hp: 0,
        evil_santa_max_hp: 8,
        evil_santa_vx: 0.0,
        evil_santa_vy: 0.0,
        evil_santa_on_ground: true,
        evil_santa_attack_cooldown: 0,
        evil_santa_attack_frame: 0,
        evil_santa_jump_cooldown: 0,
        // AI FSM init
        boss_state_timer: 0,
        boss_pattern_index: 0,
        boss_phase: 0,
        boss_phase_timer: 0,
        boss_attack_type: 0,
        evil_santa_flash_timer: 0,
        player_idle_x: 0.0,
        player_idle_timer: 0,
        boss_death_timer: 0,
        };

        game.load_level(1);
        game
    }

    pub fn update(&mut self) {
        self.frame += 1;
        
        let kb = keyboard::get();
        
        // MENU STATE - waiting to start game
        if self.in_menu {
            if kb.enter().just_pressed() {
                self.in_menu = false;
                self.load_level(1);
                self.lives = 3;
                self.score = 0;
                self.player_hp = self.player_max_hp;
                log!("Game started from menu!");
            }
            return;  // Don't process game logic while in menu
        }
        
        // GAME OVER STATE - show game over screen for 3 seconds then restart from level 1
        if self.show_game_over {
            self.game_over_timer += 1;
            if self.game_over_timer >= 180 {  // 3 seconds
                // Reset game completely and restart from level 1
                self.show_game_over = false;
                self.game_over_timer = 0;
                self.lives = 3;
                self.score = 0;
                self.player_hp = self.player_max_hp;
                self.player_state = STATE_IDLE;
                self.keys_collected = 0;
                self.kids_collected = 0;
                self.gift_bombs = 0;
                self.boss_active = false;
                self.boss_defeated = false;
                self.use_boss_santa = false;
                self.load_level(1);
                log!("Game restarted from level 1!");
            }
            return;  // Don't process game logic during game over
        }
        
        // VICTORY STATE - show victory screen for 3 seconds then restart from level 1
        // Triggered when Santa collects kid from door 0 in level 3
        if self.show_victory {
            self.game_won_timer += 1;
            if self.game_won_timer >= 180 {  // 3 seconds
                // Reset game completely and restart from level 1
                self.show_victory = false;
                self.game_won_timer = 0;
                self.lives = 3;
                self.score = 0;
                self.player_hp = self.player_max_hp;
                self.player_state = STATE_IDLE;
                self.keys_collected = 0;
                self.kids_collected = 0;
                self.gift_bombs = 0;
                self.boss_active = false;
                self.boss_defeated = false;
                self.use_boss_santa = false;
                self.load_level(1);
                log!("Victory! Game restarted from level 1!");
            }
            return;  // Don't process game logic during victory
        }
        
        // === PLAYING state logic below ===
        
        // BGM looping - restart if not playing
        // Note: Turbo SDK 5.2.0 doesn't support runtime volume control via play_with_volume
        // To reduce BGM volume by 40%, you'll need to edit the audio file itself using audio editing software
        if !audio::is_playing("bgm") {
            audio::play("bgm");
        }
        
        // Update level timer
        if self.player_state != STATE_DEAD {
            self.level_timer += 1;
            
            // Check time limit
            if self.level_timer >= self.level_time_limit {
                // Time's up! Kill player
                self.player_hp = 0;
                log!("Time's up! Level failed.");
            }
        }

        // Developer mode toggle and stage jump controls (Press . to toggle)
        if kb.period().just_pressed() {
            self.dev_mode = !self.dev_mode;
        }
        if self.dev_mode {
            // Direct stage selection
            if kb.key_j().just_pressed() { self.load_level(1); }
            if kb.key_k().just_pressed() { self.load_level(2); }
            if kb.key_b().just_pressed() { self.load_level(3); }

            // Cycle stages
            if kb.key_l().just_pressed() {
                let next = if self.level >= 3 { 1 } else { self.level + 1 };
                self.load_level(next);
            }
            if kb.key_h().just_pressed() {
                let prev = if self.level <= 1 { 3 } else { self.level - 1 };
                self.load_level(prev);
            }
        }

        if self.player_state != STATE_DEAD {
            self.handle_input();
            self.apply_jump_velocity_boost();
            self.update_player();
            self.update_camera();
        }

        // Boss arena trigger and simple idle animation ticker
        if self.level == 3 {
            if !self.boss_active && !self.boss_defeated && self.player_x >= BOSS_TRIGGER_X && self.player_y >= BOSS_TRIGGER_Y {
                self.boss_active = true;
                self.use_boss_santa = true; // switch to bossfight/santa sprite set

                // Setup boss fight arena (360x256 frame with single platform)
                self.platforms.clear();
                self.platforms.push((10.0, 350.0, 217.0)); // Single platform: x=10, width=340, y=217

                // Disable all enemies during boss fight
                for enemy in self.enemies.iter_mut() {
                    enemy.8 = false; // Set active to false
                }

                // Add walls at corners to prevent falling off
                self.walls[0] = (0.0, 0.0, 10.0, 256.0);    // Left wall
                self.walls[1] = (350.0, 0.0, 10.0, 256.0);  // Right wall
                // Clear other walls
                for i in 2..self.walls.len() {
                    self.walls[i] = (0.0, 0.0, 0.0, 0.0);
                }

                // Reset player position for boss fight arena
                self.player_x = 50.0;
                self.player_y = 169.0; // y=217-48 (48px sprite height)
                self.player_vx = 0.0;
                self.player_vy = 0.0;
                self.player_on_ground = true;

                // Initialize boss entity
                self.evil_santa_x = BOSS_EVIL_SANTA_X;
                self.evil_santa_y = BOSS_EVIL_SANTA_Y;
                self.evil_santa_state = BOSS_STATE_IDLE;
                self.evil_santa_anim_frame = 0;
                self.evil_santa_anim_timer = 0;
                self.evil_santa_facing_right = false;
                // Initialize boss combat state
                self.evil_santa_hp = self.evil_santa_max_hp;
                self.evil_santa_vx = 0.0;
                self.evil_santa_vy = 0.0;
                self.evil_santa_on_ground = true;
                self.evil_santa_attack_cooldown = 60; // short delay before first attack
                self.evil_santa_attack_frame = 0;
                self.evil_santa_jump_cooldown = 90;
            }

        if self.boss_active {
            self.update_boss();
        }
        }

        self.update_enemies();
        self.check_enemy_collisions();  // Check for damage from enemies
        self.update_projectiles();
        self.update_snowballs();  // Update penguin snowball projectiles
        self.update_placed_bombs();

        if self.timer > 0 {
            self.timer -= 1;
        } else {
            self.player_hp = 0;
        }
        
        // Check for death and handle respawn/game over
        if self.player_hp == 0 && self.player_state != STATE_DEAD {
            self.handle_death();
        }

        self.update_keys_animation();
        self.check_keys();
        self.check_gift_bomb_pickups();

        // Update and check kids
        self.update_kids_animation();
        self.check_kid_collection();
        self.check_life_collection();  // Check life powerup collection
        self.check_powerup1_collection();  // Check powerup1 collection

        // Check for level completion
        if !self.level_complete {
            self.check_level_completion();
        }

        // Handle level transition
        if self.level_complete {
            if self.level_transition_timer > 0 {
                self.level_transition_timer -= 1;
            } else {
                // Transition to next level
                let next_level = self.level + 1;
                self.load_level(next_level);
            }
        }

        self.render();
    }

    fn restart_level(&mut self) {
        // Reset player health
        self.player_hp = self.player_max_hp;
        self.player_state = STATE_IDLE;
        self.player_invuln_timer = 0;

        // Reload current level
        self.load_level(self.level);
    }

    fn handle_death(&mut self) {
        // Check if player has lives remaining
        if self.lives > 0 {
            // Lose a life
            self.lives -= 1;

            // Respawn at starting position based on current level
            match self.level {
                1 => {
                    self.player_x = 145.0;  // Level 1 starting X
                    self.player_y = 72.0;   // Level 1 starting Y
                },
                2 => {
                    self.player_x = 60.0;   // Level 2 starting X
                    self.player_y = 58.0;   // Level 2 starting Y (86 - 28)
                },
                3 => {
                    // If boss fight has started, respawn in boss arena
                    if self.boss_active {
                        self.player_x = 50.0;  // Boss arena starting X
                        self.player_y = 169.0; // Boss arena starting Y (y=217-48)
                        // Keep using 48x48 boss sprites while in boss fight
                        self.use_boss_santa = true;
                    } else {
                        self.player_x = 150.0;  // Boss level starting X
                        self.player_y = 194.0;  // Boss level starting Y (222 - 28)
                    }
                },
                _ => {
                    self.player_x = 145.0;
                    self.player_y = 72.0;
                }
            }

            self.player_vx = 0.0;
            self.player_vy = 0.0;

            // Reset health and state
            self.player_hp = self.player_max_hp;
            self.player_state = STATE_IDLE;
            self.player_on_ground = true;

            // Grant 3 seconds of invulnerability after respawn
            self.player_invuln_timer = 180;

            // Reset animation
            self.player_anim_frame = 0;
            self.player_anim_timer = 0;
        } else {
            // No lives left AND health is 0 - Game Over!
            // Trigger game over: lives <= 0 and player_hp <= 0
            self.show_game_over = true;
            self.game_over_timer = 0;
            self.player_state = STATE_DEAD;
            audio::play("santa_death");
            log!("Game Over! Lives: {}, HP: {}", self.lives, self.player_hp);
        }
    }

    fn handle_input(&mut self) {
        let kb = keyboard::get();

        // Toggle controls panel with 'S'
        if kb.key_s().just_pressed() {
            self.show_controls_panel = !self.show_controls_panel;
        }

        // Don't allow input during attack or hurt states
        if self.player_state == STATE_ATTACK || self.player_state == STATE_HURT {
            return;
        }

        // Get key states - NEW KEY BINDINGS
        let left = kb.arrow_left().pressed();
        let right = kb.arrow_right().pressed();
        let up = kb.arrow_up().pressed();
        let down = kb.arrow_down().pressed();
        let jump_pressed = kb.key_x().just_pressed();
        let jump_held = kb.key_x().pressed();
        let attack = kb.key_z().just_pressed();  // Z key for attack
        let place_bomb = kb.key_c().just_pressed();  // C key to place bomb

        // Check if player is near a ladder
        let on_ladder = self.check_ladder_collision();

        // ==================== CLIMBING LOGIC ====================
        if on_ladder {
            // Press UP to start climbing up, or DOWN to start climbing down
            if (up || down) && !self.player_on_ladder {
                self.player_on_ladder = true;
                self.player_state = STATE_CLIMB;
                self.player_vy = 0.0;
                self.player_vx = 0.0;
            }
        }

        // Handle ladder climbing movement
        if self.player_on_ladder {
            // Force exit ladder if standing on solid ground and not pressing up/down
            if self.player_on_ground && !up && !down {
                self.player_on_ladder = false;
                self.player_state = STATE_IDLE;
                self.player_vy = 0.0;
                self.player_anim_frame = 0;
                self.player_anim_timer = 0;
                // Don't return, let normal movement take over
            } else {
                self.player_vx = 0.0;  // No horizontal movement while climbing

                if up {
                    self.player_vy = -CLIMB_SPEED;
                    self.player_state = STATE_CLIMB;
                } else if down {
                    self.player_vy = CLIMB_SPEED;
                    self.player_state = STATE_CLIMB;
                } else {
                    self.player_vy = 0.0;
                }

                // Jump off ladder with X key + left/right
                if jump_pressed && (left || right) {
                    self.player_on_ladder = false;
                    self.player_vy = JUMP_VEL * 0.7;  // Smaller jump when jumping off ladder
                    self.player_state = STATE_JUMP;
                    self.jump_anim_frame = 0;
                    self.jump_anim_timer = 0;
                    audio::play("jump");
                    if left {
                        self.player_facing_right = false;
                        self.player_vx = -WALK_SPEED;
                    } else {
                        self.player_facing_right = true;
                        self.player_vx = WALK_SPEED;
                    }
                }

                // Exit ladder if no longer overlapping
                if !on_ladder {
                    self.player_on_ladder = false;
                }

                return;  // Skip normal movement while on ladder
            }
        }

        // ==================== CROUCHING LOGIC ====================
        if down && self.player_on_ground && !self.player_on_ladder {
            self.player_is_crouching = true;
            self.player_state = STATE_CROUCH;
            // Can still move slowly while crouching
            let crouch_target = if left && !right {
                -CROUCH_SPEED
            } else if right && !left {
                CROUCH_SPEED
            } else {
                0.0
            };
            self.player_vx = crouch_target;
            
            if left && !right {
                self.player_facing_right = false;
            } else if right && !left {
                self.player_facing_right = true;
            }
            return;
        } else {
            self.player_is_crouching = false;
        }

        // ==================== HORIZONTAL MOVEMENT ====================
        if left && !right {
            self.player_facing_right = false;
        } else if right && !left {
            self.player_facing_right = true;
        }

        let target_speed = if left && !right {
            -WALK_SPEED
        } else if right && !left {
            WALK_SPEED
        } else {
            0.0
        };

        // Apply acceleration/deceleration
        if self.player_on_ground {
            if target_speed.abs() > 0.01 {
                if (target_speed > 0.0 && self.player_vx < target_speed) ||
                   (target_speed < 0.0 && self.player_vx > target_speed) {
                    self.player_vx += target_speed.signum() * ACCEL;
                }
            } else {
                if self.player_vx.abs() > DECEL {
                    self.player_vx -= self.player_vx.signum() * DECEL;
                } else {
                    self.player_vx = 0.0;
                }
            }
        } else {
            // Air control
            if target_speed.abs() > 0.01 {
                if (target_speed > 0.0 && self.player_vx < target_speed) ||
                   (target_speed < 0.0 && self.player_vx > target_speed) {
                    self.player_vx += target_speed.signum() * AIR_CONTROL;
                }
            }
        }

        self.player_vx = self.player_vx.clamp(-WALK_SPEED, WALK_SPEED);

        // ==================== JUMPING LOGIC ====================
        // Input buffering for jump
        if jump_pressed {
            self.player_jump_buffer = INPUT_BUFFER;
        } else if self.player_jump_buffer > 0 {
            self.player_jump_buffer -= 1;
        }

        // Execute jump if buffer valid and grounded (or coyote time)
        if self.player_jump_buffer > 0 && (self.player_on_ground || self.player_coyote_timer > 0) {
            self.player_vy = JUMP_VEL;
            self.player_state = STATE_JUMP;
            self.player_jump_buffer = 0;
            self.player_on_ground = false;
            self.player_coyote_timer = 0;
            // Reset jump animation
            self.jump_anim_frame = 0;
            self.jump_anim_timer = 0;
            audio::play("jump");
        }

        // Variable jump height - release early for shorter jump
        if !jump_held && self.player_vy < 0.0 {
            self.player_vy *= 0.5;
        }

        // ==================== ATTACK LOGIC ====================
        // Place gift bomb with C key
        if place_bomb && self.gift_bombs > 0 && self.player_on_ground {
            self.place_gift_bomb();
        } else if attack && self.attack_cooldown == 0 {
            self.player_state = STATE_ATTACK;
            self.attack_frame = 0;
        }
    }

    // Check if player is overlapping with any ladder
    fn check_ladder_collision(&self) -> bool {
        let player_center_x = self.player_x;
        let player_left = self.player_x - 7.0;
        let player_right = self.player_x + 7.0;
        let player_top = self.player_y - 19.0;  // Full player height
        let player_bottom = self.player_y + 19.0;

        for (lx, ly_top, ly_bottom) in self.ladders.iter() {
            if *lx > 0.0 {  // Valid ladder
                let ladder_left = *lx - 4.0;  // Narrower collision (4px each side)
                let ladder_right = *lx + 4.0;

                // Check if player overlaps ladder horizontally and vertically
                // More forgiving: check if ANY part of player overlaps, not just center
                if player_right > ladder_left && player_left < ladder_right {
                    if player_bottom >= *ly_top && player_top <= *ly_bottom {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn apply_jump_velocity_boost(&mut self) {
        if self.player_state == STATE_JUMP || self.player_state == STATE_FALL {
            // Only boost horizontal velocity during jump, don't change ground speed
            let kb = keyboard::get();
            
            // Check for horizontal input during jump
            if kb.arrow_left().pressed() || kb.arrow_right().pressed() {
                // Apply velocity boost only during jump state
                let current_boost = if kb.arrow_left().pressed() { 
                    -WALK_SPEED * (JUMP_LENGTH_MULTIPLIER - 1.0) 
                } else { 
                    WALK_SPEED * (JUMP_LENGTH_MULTIPLIER - 1.0) 
                };
                
                // Add boost to existing velocity (don't replace it)
                // This only affects Santa's player_vx, not camera or background
                self.player_vx += current_boost;
            }
        }
    }

    fn update_player(&mut self) {
        // ==================== ATTACK STATE ====================
        if self.player_state == STATE_ATTACK {
            self.attack_frame += 1;

            // Spawn projectile at frame 4 (out of 5 attack frames)
            if self.attack_frame == 4 {
                self.spawn_player_projectile();
            }

            // Attack has 5 frames at 4 frames each = 20 frame total
            if self.attack_frame >= 20 {
                self.player_state = STATE_IDLE;
                self.attack_cooldown = 24;
            }
            return;
        }

        if self.attack_cooldown > 0 {
            self.attack_cooldown -= 1;
        }

        // ==================== CLIMBING STATE ====================
        if self.player_on_ladder {
            // Apply ladder movement (no gravity while on ladder)
            self.player_y += self.player_vy;

            let player_bottom = self.player_y + 19.0;
            let player_left = self.player_x - 7.0;
            let player_right = self.player_x + 7.0;

            // Check for platform collisions when climbing UP
            if self.player_vy < 0.0 {  // Moving up
                for (px1, px2, py) in self.platforms.iter() {
                    if *px2 > *px1 &&
                       player_right > *px1 &&
                       player_left < *px2
                    {
                        // Platform must be above us
                        if *py < self.player_y {
                            let dist_to_platform = player_bottom - *py;

                            // When feet reach the platform while climbing up
                            if dist_to_platform >= -2.0 && dist_to_platform <= 4.0 {
                                self.player_y = *py - 19.0;
                                self.player_vy = 0.0;
                                self.player_on_ground = true;
                                self.player_on_ladder = false;
                                self.player_state = STATE_IDLE;
                                // Reset animation to idle
                                self.player_anim_frame = 0;
                                self.player_anim_timer = 0;
                                break;
                            }
                        }
                    }
                }
            }

            // Check for platform collisions when climbing down
            // This prevents falling through floors
            if self.player_vy > 0.0 {  // Moving down
                for (px1, px2, py) in self.platforms.iter() {
                    if *px2 > *px1 &&
                       player_right > *px1 &&
                       player_left < *px2
                    {
                        // Only land on platforms where our FEET are approaching from ABOVE
                        // Platform must be at least 30px below our current center
                        // This ensures we've descended enough to reach the next floor
                        if *py > (self.player_y + 21.0) {
                            let dist_to_platform = player_bottom - *py;

                            // Tight landing window when feet reach platform
                            if dist_to_platform >= -2.0 && dist_to_platform <= 4.0 {
                                self.player_y = *py - 19.0;
                                self.player_vy = 0.0;
                                self.player_on_ground = true;
                                self.player_on_ladder = false;
                                self.player_state = STATE_IDLE;
                                // Reset animation to idle
                                self.player_anim_frame = 0;
                                self.player_anim_timer = 0;
                                break;
                            }
                        }
                    }
                }
            }

            // Update climbing animation - cycle through all 10 frames
            if self.player_vy.abs() > 0.1 {
                self.player_anim_timer += 1;
                if self.player_anim_timer >= 6 {  // Faster animation (was 10)
                    self.player_anim_timer = 0;
                    self.player_anim_frame = (self.player_anim_frame + 1) % 10;  // All 10 frames
                }
            }
            return;
        }

        // ==================== CROUCHING STATE ====================
        if self.player_is_crouching {
            // Apply horizontal movement while crouching
            self.player_x += self.player_vx;
            self.check_player_collisions();
            return;
        }

        // ==================== NORMAL PHYSICS ====================
        self.player_vy += GRAVITY;
        if self.player_vy > TERMINAL_VEL {
            self.player_vy = TERMINAL_VEL;
        }

        if !self.player_on_ground && self.player_coyote_timer > 0 {
            self.player_coyote_timer -= 1;
        }

        if self.player_invuln_timer > 0 {
            self.player_invuln_timer -= 1;
        }

        self.player_x += self.player_vx;
        self.player_y += self.player_vy;

        self.check_player_collisions();

        // ==================== STATE DETERMINATION ====================
        if self.player_on_ground {
            if self.player_vx.abs() < 0.1 {
                self.player_state = STATE_IDLE;
            } else {
                self.player_state = STATE_WALK;
            }
            // Reset jump animation when landing
            self.jump_anim_frame = 0;
            self.jump_anim_timer = 0;
        } else {
            if self.player_vy < 0.0 {
                self.player_state = STATE_JUMP;
            } else {
                self.player_state = STATE_FALL;
            }
        }

        // ==================== ANIMATION UPDATES ====================
        self.update_animations();
    }

    fn update_animations(&mut self) {
        match self.player_state {
            STATE_IDLE => {
                // 4 idle frames, 15 frames per sprite for breathing effect
                self.player_anim_timer += 1;
                if self.player_anim_timer >= 15 {
                    self.player_anim_timer = 0;
                    self.player_anim_frame = (self.player_anim_frame + 1) % IDLE_FRAMES;
                }
            },
            STATE_WALK | STATE_RUN => {
                // 10 running frames, 6 frames per sprite for smooth run cycle
                self.player_anim_timer += 1;
                if self.player_anim_timer >= 6 {
                    self.player_anim_timer = 0;
                    self.player_anim_frame = (self.player_anim_frame + 1) % RUN_FRAMES;
                }
            },
            STATE_JUMP => {
                // 5 jumping frames, progress through based on velocity
                self.jump_anim_timer += 1;
                if self.jump_anim_timer >= 8 {
                    self.jump_anim_timer = 0;
                    if self.jump_anim_frame < JUMP_FRAMES - 1 {
                        self.jump_anim_frame += 1;
                    }
                }
            },
            STATE_FALL => {
                // Use last jump frame or fall sprite
                self.jump_anim_frame = JUMP_FRAMES - 1;
            },
            STATE_ATTACK => {
                // 5 attack frames, 4 frames per sprite
                let attack_sprite_frame = (self.attack_frame / 4) % ATTACK_FRAMES;
                self.player_anim_frame = attack_sprite_frame;
            },
            STATE_CLIMB => {
                // 10 climbing frames - animation handled in update_player
                // This state is managed separately because climbing uses player_vy
            },
            STATE_CROUCH => {
                // Use scared sprite for crouching
                self.player_anim_frame = 0;
            },
            _ => {}
        }
    }

    fn check_player_collisions(&mut self) {
        let was_on_ground = self.player_on_ground;
        self.player_on_ground = false;

        // Player hitbox (center-based Y coordinate)
        let player_bottom = self.player_y + 19.0;
        let player_left = self.player_x - 7.0;
        let player_right = self.player_x + 7.0;

        // ============================================
        // VERTICAL COLLISION PASS (NES-style)
        // ============================================
        // Platforms are 1D horizontal surfaces
        // Wide tolerance (20px) handles high-speed falls

        if self.player_vy >= 0.0 {  // Only when falling/grounded
            for (px1, px2, py) in self.platforms.iter() {
                // Check if platform is valid and player overlaps horizontally
                if *px2 > *px1 &&
                   player_right > *px1 &&
                   player_left < *px2
                {
                    // Wide vertical tolerance for consistent collision
                    let dist_to_platform = player_bottom - *py;

                    if dist_to_platform >= 0.0 && dist_to_platform <= 20.0 {
                        // HARD SNAP to platform top (NES-style, no smoothing)
                        self.player_y = *py - 19.0;
                        self.player_vy = 0.0;
                        self.player_on_ground = true;

                        if !was_on_ground {
                            self.player_coyote_timer = COYOTE_TIME;
                        }
                        break;  // Stop at first collision
                    }
                }
            }
        }

        // ============================================
        // HORIZONTAL COLLISION PASS (walls & bounds)
        // ============================================

        // Check wall collisions (AABB - Axis-Aligned Bounding Box)
        let player_top = self.player_y - 19.0;

        for (i, (wx, wy, ww, wh)) in self.walls.iter().enumerate() {
            // Check if wall is valid
            if *ww > 0.0 && *wh > 0.0 {
                // Gate logic: different for each level
                if self.level == 3 && !self.boss_active {
                    // Level 3 (not in boss fight): Gate #1 (wall[0]) disappears with 1 key, Gate #2 (wall[1]) with 2 keys
                    if (i == 0 && self.keys_collected >= 1) || (i == 1 && self.keys_collected >= 2) {
                        continue;
                    }
                } else if self.level != 3 {
                    // Other levels: Gate (wall #3) disappears once 3 keys are collected
                    if i == 2 && self.keys_collected >= 3 {
                        continue;
                    }
                }
                // During boss fight in level 3, walls[0] and walls[1] are corner walls and should NOT disappear

                let wall_left = *wx;
                let wall_right = *wx + *ww;
                let wall_top = *wy;
                let wall_bottom = *wy + *wh;

                // AABB collision detection
                if player_right > wall_left &&
                   player_left < wall_right &&
                   player_bottom > wall_top &&
                   player_top < wall_bottom
                {
                    // Collision detected - push player out
                    let overlap_left = player_right - wall_left;
                    let overlap_right = wall_right - player_left;
                    let overlap_top = player_bottom - wall_top;
                    let overlap_bottom = wall_bottom - player_top;

                    // Find smallest overlap (that's the direction to push)
                    let min_overlap = overlap_left.min(overlap_right).min(overlap_top).min(overlap_bottom);

                    if min_overlap == overlap_left {
                        // Push player left
                        self.player_x = wall_left - 7.0;
                        self.player_vx = 0.0;
                    } else if min_overlap == overlap_right {
                        // Push player right
                        self.player_x = wall_right + 7.0;
                        self.player_vx = 0.0;
                    } else if min_overlap == overlap_top {
                        // Push player down (hitting ceiling)
                        self.player_y = wall_top - 19.0;
                        self.player_vy = 0.0;
                    } else {
                        // Push player up (standing on wall top)
                        self.player_y = wall_bottom + 19.0;
                        self.player_vy = 0.0;
                        self.player_on_ground = true;
                    }
                }
            }
        }

        // World bounds - left wall
        if self.player_x < 7.0 {
            self.player_x = 7.0;
            self.player_vx = 0.0;
        }

        // World bounds - right wall (level dependent)
        let max_x = match self.level {
            1 => 1073.0,  // Level 1 boundary (1080 - 7)
            2 => 1433.0,  // Level 2 boundary (1440 - 7)
            3 => 2153.0,  // Boss level boundary (2160 - 7)
            _ => 1073.0,  // Default
        };
        if self.player_x > max_x {
            self.player_x = max_x;
            self.player_vx = 0.0;
        }

        // ============================================
        // DEATH ZONE (below all platforms)
        // ============================================

        if self.player_y > 260.0 {
            self.player_hp = 0;
            self.player_state = STATE_DEAD;
        }
    }

    fn spawn_player_projectile(&mut self) {
        let spawn_x = if self.player_facing_right {
            self.player_x + 20.0
        } else {
            self.player_x - 20.0
        };
        let spawn_y = self.player_y;
        let vel = if self.player_facing_right { 4.0 } else { -4.0 };

        let player_proj_count = self.projectiles.iter()
            .filter(|(active, _, _, _, is_enemy, _)| *active && !*is_enemy)
            .count();

        if player_proj_count >= 2 {
            return;
        }

        for proj in self.projectiles.iter_mut() {
            if !proj.0 {
                *proj = (true, spawn_x, spawn_y, vel, false, 0.0);
                break;
            }
        }
    }

    fn spawn_gift_bomb(&mut self, x: f32, enemy_y: f32) {
        // Enemy Y = platform_y - 28 (enemy center is 28px above platform)
        // Santa Y = platform_y - 28 (Santa center is 28px above platform)
        // Gift bomb should be at same Y as Santa/enemy for consistency
        // Gift bomb sprite is 64x64 with gift at bottom middle
        // We want the gift bomb center at the same Y as Santa (enemy_y)
        let bomb_y = enemy_y;

        // Find an empty slot for the gift bomb item
        for item in self.gift_bomb_items.iter_mut() {
            if !item.2 {  // If not active
                // Set despawn timer to 600 frames (10 seconds at 60 FPS)
                *item = (x, bomb_y, true, 600);
                break;
            }
        }
    }

    fn place_gift_bomb(&mut self) {
        // Find an empty slot for the placed bomb
        for bomb in self.placed_bombs.iter_mut() {
            if !bomb.2 {  // If not active
                // Place bomb at Santa's Y position (same level as Santa)
                // Santa is at player_y (center is 28px above platform)
                // Gift bomb should be at same Y as Santa
                let bomb_y = self.player_y;

                *bomb = (self.player_x, bomb_y, true, 0, 0);
                self.gift_bombs -= 1;
                break;
            }
        }
    }

    fn update_projectiles(&mut self) {
        // First, check for projectile-enemy collisions (player projectiles only)
        for proj_idx in 0..self.projectiles.len() {
            if self.projectiles[proj_idx].0 && !self.projectiles[proj_idx].4 {
                // Active player projectile - check against all enemies
                let proj_x = self.projectiles[proj_idx].1;
                let proj_y = self.projectiles[proj_idx].2;
                
                // Check boss collision first
                let mut hit_boss = false;
                if self.boss_active && self.evil_santa_hp > 0 {
                    let dxb = (proj_x - self.evil_santa_x).abs();
                    let dyb = (proj_y - self.evil_santa_y).abs();
                    if dxb < 20.0 && dyb < 20.0 {
                        // Hit boss: decrement HP and apply small knockback
                        self.projectiles[proj_idx].0 = false;
                        self.evil_santa_hp = self.evil_santa_hp.saturating_sub(2); // Increased damage
                        audio::play("evilSanta_hurt");
                        self.evil_santa_vx = if proj_x < self.evil_santa_x { 0.8 } else { -0.8 };
                        self.evil_santa_flash_timer = 5; // Flash for 5 frames
                        // Reset to idle after hit if on ground
                        if self.evil_santa_on_ground { self.evil_santa_state = BOSS_STATE_IDLE; }
                        hit_boss = true;
                    }
                }

                if hit_boss { continue; }

                for enemy_idx in 0..self.enemies.len() {
                    // Check if enemy is alive (active and not respawning)
                    if self.enemies[enemy_idx].8 && self.enemies[enemy_idx].11 == 0 {
                        let enemy_x = self.enemies[enemy_idx].1;
                        let enemy_y = self.enemies[enemy_idx].2;
                        
                        // Collision check: 16px radius for both projectile and enemy
                        let dx = (proj_x - enemy_x).abs();
                        let dy = (proj_y - enemy_y).abs();
                        
                        if dx < 20.0 && dy < 20.0 {
                            // Hit! Kill the enemy and deactivate projectile
                            self.projectiles[proj_idx].0 = false;

                            // Start respawn timer (600 frames = 10 seconds at 60fps)
                            self.enemies[enemy_idx].11 = 600;

                            // Drop a gift bomb at enemy's position
                            self.spawn_gift_bomb(enemy_x, enemy_y);

                            // Add score for kill
                            self.score += 100;
                            
                            // Play randomized kill sound
                            let kill_sfx = if self.frame % 2 == 0 { "santa_kill" } else { "santa_kill_2" };
                            audio::play(kill_sfx);

                            break;  // Projectile can only hit one enemy
                        }
                    }
                }
            }
        }
        
        // Boss projectiles hit Santa
        for proj_idx in 0..self.projectiles.len() {
            if self.projectiles[proj_idx].0 && self.projectiles[proj_idx].4 {
                let proj_x = self.projectiles[proj_idx].1;
                let proj_y = self.projectiles[proj_idx].2;
                let dx = (proj_x - self.player_x).abs();
                let dy = (proj_y - self.player_y).abs();
                if dx < 16.0 && dy < 16.0 && self.player_invuln_timer == 0 {
                    // Apply modest damage and knockback
                    let damage = ((self.player_max_hp as f32 * 0.10).ceil() as u8).max(1);
                    self.player_hp = self.player_hp.saturating_sub(damage);
                    let hurt_sfx = if self.frame % 2 == 0 { "santa_hurt_1" } else { "santa_hurt_2" };
                    audio::play(hurt_sfx);
                    self.player_invuln_timer = 60;
                    self.player_vx = if proj_x > self.player_x { -3.0 } else { 3.0 };
                    self.player_vy = -3.5;
                    // Despawn projectile on hit
                    self.projectiles[proj_idx].0 = false;
                }
            }
        }
        
        // Then update projectile positions
        for proj in self.projectiles.iter_mut() {
            if proj.0 {
                proj.1 += proj.3;
                proj.5 += proj.3.abs();

                let max_dist = if proj.4 { 200.0 } else { 5.0 };
                if proj.5 > max_dist {
                    proj.0 = false;
                }

                if proj.1 < self.camera_x - 50.0 || proj.1 > self.camera_x + SCREEN_WIDTH + 50.0 {
                    proj.0 = false;
                }
            }
        }
    }

    fn check_enemy_collisions(&mut self) {
        // Decrement invulnerability timer
        if self.player_invuln_timer > 0 {
            self.player_invuln_timer -= 1;
            return;  // Can't take damage while invulnerable
        }

        // Only check collisions when alive
        if self.player_state == STATE_DEAD {
            return;
        }

        // Check collision with each active enemy

        // Check collision with each active enemy
        for enemy in self.enemies.iter() {
            // Skip if enemy is dead/respawning (respawn_timer > 0)
            if enemy.8 && enemy.11 == 0 {  // active AND alive (not respawning)
                let dx = (self.player_x - enemy.1).abs();
                let dy = (self.player_y - enemy.2).abs();
                
                // Collision threshold: 20px horizontal, 20px vertical
                if dx < 20.0 && dy < 20.0 {
                    // Apply damage: 10% of max HP = 1 HP from max 10
                    let damage = ((self.player_max_hp as f32 * 0.1).ceil() as u8).max(1);
                    self.player_hp = self.player_hp.saturating_sub(damage);
                    let hurt_sfx = if self.frame % 2 == 0 { "santa_hurt_1" } else { "santa_hurt_2" };
                    audio::play(hurt_sfx);
                    
                    // Set invulnerability frames (2 seconds at 60fps)
                    self.player_invuln_timer = 120;
                    
                    // Knockback effect
                    if enemy.1 < self.player_x {
                        self.player_vx = 3.0;  // Push right
                    } else {
                        self.player_vx = -3.0; // Push left
                    }
                    self.player_vy = -3.0;  // Push up slightly
                    
                    // Death is handled in update() loop
                    // (handle_death will be called there)
                    
                    break;  // Only one hit per frame
                }
            }
        }
    }

    fn update_enemies(&mut self) {
        for i in 0..self.enemies.len() {
            // Skip if enemy type is 0 (not initialized)
            if self.enemies[i].0 == 0 {
                continue;
            }
            
            // Handle respawn timer (enemy.11 = respawn_timer)
            if self.enemies[i].11 > 0 {
                self.enemies[i].11 -= 1;
                
                // Cloud animation calculated on-the-fly in draw_enemies()
                // during last 120 frames before respawn
                
                // Respawn complete - reset enemy to patrol start position
                if self.enemies[i].11 == 0 {
                    self.enemies[i].1 = self.enemies[i].7 as f32 + 50.0;  // Reset X to patrol center
                    self.enemies[i].6 = 0;  // Reset animation frame
                    self.enemies[i].9 = 0;  // Reset animation timer
                    self.enemies[i].10 = false;  // Not attacking
                }
                
                continue;  // Skip normal AI while dead/respawning
            }
            
            // Only process active enemies that are alive
            if self.enemies[i].8 {
                let enemy_type = self.enemies[i].0;

                // Attack detection for kickmouse (type 2) and penguin (type 3)
                if enemy_type == 2 || enemy_type == 3 {
                    let dx = (self.player_x - self.enemies[i].1).abs();
                    let dy = (self.player_y - self.enemies[i].2).abs();

                    // Trigger attack if player is within range
                    let attack_range_h = if enemy_type == 3 { 30.0 } else { 10.0 }; // Penguin: 30px, Kickmouse: 10px
                    let attack_range_v = if enemy_type == 3 { 10.0 } else { 20.0 }; // Penguin: 10px, Kickmouse: 20px
                    if dx < attack_range_h && dy < attack_range_v && !self.enemies[i].10 {
                        self.enemies[i].10 = true;  // Start attack
                        self.enemies[i].6 = 0;  // Reset animation to frame 0
                        self.enemies[i].9 = 0;  // Reset timer
                        
                        // Penguin always faces Santa when attacking
                        if enemy_type == 3 {
                            // Set direction based on Santa's position relative to penguin
                            self.enemies[i].5 = if self.player_x > self.enemies[i].1 { 1 } else { 0 };
                            
                            // Spawn snowball projectile toward Santa
                            let penguin_x = self.enemies[i].1;
                            let penguin_y = self.enemies[i].2;
                            let snowball_speed = 3.0;
                            let vx = if self.player_x > penguin_x { snowball_speed } else { -snowball_speed };
                            
                            // Find an empty slot for snowball
                            for snowball in self.snowballs.iter_mut() {
                                if !snowball.0 {
                                    *snowball = (true, penguin_x, penguin_y, vx, 0.0);
                                    break;
                                }
                            }
                        }
                    }
                }

                // Patrol AI (only when not attacking)
                if !self.enemies[i].10 {
                    if enemy_type == 1 || enemy_type == 2 || enemy_type == 3 {
                        // Store old position for collision detection
                        let old_x = self.enemies[i].1;
                        let patrol_width = if self.level == 3 {
                                                60.0   // boss level: tight platforms
                                        } else {
                                                100.0  // level 1 & 2: existing behavior
                                        };
                        if self.enemies[i].5 == 0 {
                            self.enemies[i].1 -= 0.75;
                            if self.enemies[i].1 < self.enemies[i].7 as f32 {
                                self.enemies[i].5 = 1;
                            }
                        } else {
                            self.enemies[i].1 += 0.75;
                            if self.enemies[i].1 > (self.enemies[i].7 as f32 + patrol_width) {
                                self.enemies[i].5 = 0;
                            }
                        }

                        // Check wall collision (enemy is 16px wide, centered)
                        let enemy_left = self.enemies[i].1 - 8.0;
                        let enemy_right = self.enemies[i].1 + 8.0;
                        let enemy_top = self.enemies[i].2 - 8.0;
                        let enemy_bottom = self.enemies[i].2 + 8.0;

                        for (wx, wy, ww, wh) in self.walls.iter() {
                            if *ww > 0.0 && *wh > 0.0 {
                                let wall_left = *wx;
                                let wall_right = *wx + *ww;
                                let wall_top = *wy;
                                let wall_bottom = *wy + *wh;

                                // AABB collision check
                                if enemy_right > wall_left && enemy_left < wall_right &&
                                   enemy_bottom > wall_top && enemy_top < wall_bottom {
                                    // Hit a wall! Reverse direction and restore position
                                    self.enemies[i].1 = old_x;
                                    self.enemies[i].5 = 1 - self.enemies[i].5;  // Flip direction
                                    break;
                                }
                            }
                        }
                    }
                }

                // Update animation
                self.enemies[i].9 += 1; // anim_timer

                if enemy_type == 1 {
                    // Enemy1 (easy mouse): 8 frames, 8 frames per sprite
                    if self.enemies[i].9 >= 8 {
                        self.enemies[i].9 = 0;
                        self.enemies[i].6 = (self.enemies[i].6 + 1) % 8; // anim_frame cycles 0-7
                    }
                } else if enemy_type == 2 || enemy_type == 3 {
                    // Kickmouse and Penguin: Handle attack or walk animation
                    if self.enemies[i].10 {
                        // Attack animation: 3 frames, 10 ticks per frame (slower)
                        if self.enemies[i].9 >= 10 {
                            self.enemies[i].9 = 0;
                            self.enemies[i].6 += 1;

                            // After 3 attack frames, return to walk
                            if self.enemies[i].6 >= 3 {
                                self.enemies[i].10 = false;  // End attack
                                self.enemies[i].6 = 0;  // Reset to walk frame 0
                            }
                        }
                    } else {
                        // Walk animation: 8 frames, 8 ticks per frame
                        if self.enemies[i].9 >= 8 {
                            self.enemies[i].9 = 0;
                            self.enemies[i].6 = (self.enemies[i].6 + 1) % 8; // anim_frame cycles 0-7
                        }
                    }
                }
            }
        }
    }

    fn update_snowballs(&mut self) {
        // Update each active snowball
        for snowball in self.snowballs.iter_mut() {
            if snowball.0 {
                // Move snowball
                snowball.1 += snowball.3;  // x += vx
                snowball.2 += snowball.4;  // y += vy (gravity can be added later)
                
                // Check collision with Santa (if not invulnerable)
                if self.player_invuln_timer == 0 && self.player_state != STATE_DEAD {
                    let dx = (snowball.1 - self.player_x).abs();
                    let dy = (snowball.2 - self.player_y).abs();
                    
                    // Collision check: 16px radius
                    if dx < 16.0 && dy < 16.0 {
                        // Hit Santa! Deal 1 HP damage
                        self.player_hp = self.player_hp.saturating_sub(1);
                        
                        // Play hurt sound
                        let hurt_sfx = if self.frame % 2 == 0 { "santa_hurt_1" } else { "santa_hurt_2" };
                        audio::play(hurt_sfx);
                        
                        // Grant invulnerability frames (1 second)
                        self.player_invuln_timer = 60;
                        
                        // Small knockback from snowball direction
                        if snowball.3 > 0.0 {
                            self.player_vx = 2.0;  // Push right
                        } else {
                            self.player_vx = -2.0; // Push left
                        }
                        self.player_vy = -2.0;  // Small upward push
                        
                        // Deactivate snowball
                        snowball.0 = false;
                        continue;
                    }
                }
                
                // Despawn if off-screen (traveled too far)
                if snowball.1 < self.camera_x - 50.0 || snowball.1 > self.camera_x + SCREEN_WIDTH + 50.0 {
                    snowball.0 = false;
                }
            }
        }
    }

    fn check_gift_bomb_pickups(&mut self) {
        for item in self.gift_bomb_items.iter_mut() {
            if item.2 {  // If active
                // Countdown despawn timer
                if item.3 > 0 {
                    item.3 -= 1;
                } else {
                    // Timer expired - despawn the gift bomb
                    item.2 = false;
                    continue;
                }
                
                // Check for pickup collision
                let dx = (self.player_x - item.0).abs();
                let dy = (self.player_y - item.1).abs();
                if dx < 20.0 && dy < 20.0 && self.gift_bombs < 1 {
                    // Only pick up if we don't already have a bomb
                    item.2 = false;  // Deactivate the item
                    self.gift_bombs = 1;  // Set to 1 bomb
                    self.score += 50;
                    audio::play("collection");
                }
            }
        }
    }

    fn update_placed_bombs(&mut self) {
        for i in 0..self.placed_bombs.len() {
            if self.placed_bombs[i].2 {  // If active
                self.placed_bombs[i].4 += 1;  // Increment anim_timer

                // New bomb animation sequence (5+ seconds total):
                // Frame 0 (idle): 60 frames (1 second)
                // Frame 1-4 (sprites 2-5, first cycle): 30 frames each (2 seconds) - WARNING
                // Frame 5-8 (sprites 2-5, second cycle): 30 frames each (2 seconds) - WARNING
                // Frame 9-11 (sprites 6-8): 10 frames each (0.5 seconds) - EXPLOSION

                if self.placed_bombs[i].3 == 0 {
                    // Idle phase - 1 second
                    if self.placed_bombs[i].4 >= 60 {
                        self.placed_bombs[i].4 = 0;
                        self.placed_bombs[i].3 = 1;  // Move to warning phase
                    }
                } else if self.placed_bombs[i].3 >= 1 && self.placed_bombs[i].3 <= 8 {
                    // Warning phase - frames 2-5 played twice (frames 1-8)
                    if self.placed_bombs[i].4 >= 30 {
                        self.placed_bombs[i].4 = 0;
                        self.placed_bombs[i].3 += 1;
                    }
                } else if self.placed_bombs[i].3 == 9 {
                    // EXPLOSION STARTS - Frame 6 (sprite 6)
                    if self.placed_bombs[i].4 == 1 {
                        // Apply damage on first frame of explosion
                        self.check_bomb_damage(i);
                        audio::play("explosion");
                    }
                    if self.placed_bombs[i].4 >= 10 {
                        self.placed_bombs[i].4 = 0;
                        self.placed_bombs[i].3 = 10;
                    }
                } else if self.placed_bombs[i].3 >= 10 && self.placed_bombs[i].3 <= 11 {
                    // Explosion frames 7-8
                    if self.placed_bombs[i].4 >= 10 {
                        self.placed_bombs[i].4 = 0;
                        self.placed_bombs[i].3 += 1;
                        if self.placed_bombs[i].3 > 11 {
                            // Animation complete, deactivate bomb
                            self.placed_bombs[i].2 = false;
                        }
                    }
                }
            }
        }
    }

    fn check_bomb_damage(&mut self, bomb_idx: usize) {
        let bomb_x = self.placed_bombs[bomb_idx].0;
        let bomb_y = self.placed_bombs[bomb_idx].1;

        // Check if Santa is in explosion radius
        let dx_player = (bomb_x - self.player_x).abs();
        let dy_player = (bomb_y - self.player_y).abs();

        if dx_player < 50.0 && dy_player < 50.0 {
            // Santa is caught in the explosion!
            if self.player_invuln_timer == 0 {
                 self.player_hp = self.player_hp.saturating_sub(2);  // Take 2 damage
                let hurt_sfx = if self.frame % 2 == 0 { "santa_hurt_1" } else { "santa_hurt_2" };
                audio::play(hurt_sfx);
                self.player_invuln_timer = 60;  // 1 second invulnerability

                if self.player_hp == 0 {
                    self.player_state = STATE_DEAD;
                    self.lives = self.lives.saturating_sub(1);
                }
            }
        }

        // Check all enemies for collision with bomb explosion
        // Explosion radius is approximately 50 pixels
        for enemy_idx in 0..self.enemies.len() {
            if self.enemies[enemy_idx].8 && self.enemies[enemy_idx].11 == 0 {  // Active and alive
                let enemy_x = self.enemies[enemy_idx].1;
                let enemy_y = self.enemies[enemy_idx].2;

                let dx = (bomb_x - enemy_x).abs();
                let dy = (bomb_y - enemy_y).abs();

                // Kill enemies within explosion radius
                if dx < 50.0 && dy < 50.0 {
                    // Kill the enemy (start respawn timer)
                    self.enemies[enemy_idx].11 = 600;

                    // Add score for bomb kill
                    self.score += 150;
                }
            }
        }

        // Check all doors for collision with bomb explosion
        // Doors are destroyed if within explosion radius
        for door_idx in 0..self.doors.len() {
            if self.doors[door_idx].2 {  // If door is still intact
                let door_x = self.doors[door_idx].0;
                let door_y = self.doors[door_idx].1;
                
                // Door is 42x42, so center is at (x+21, y+21)
                let door_center_x = door_x + 21.0;
                let door_center_y = door_y + 21.0;
                
                let dx = (bomb_x - door_center_x).abs();
                let dy = (bomb_y - door_center_y).abs();
                
                // Destroy door if within explosion radius
                if dx < 50.0 && dy < 50.0 {
                    self.doors[door_idx].2 = false;  // Mark as destroyed
                    self.score += 200;  // Bonus score for destroying door

                    // Check if this door spawns a life powerup (door #3 in level 3)
                    if self.level == 3 && door_idx == self.life_door_index && !self.life_collected {
                        // Spawn life powerup at door position
                        self.life_position = (door_center_x, door_center_y - 10.0);
                        self.life_active = true;
                        log!("Life powerup spawned from door {}!", door_idx);
                    }
                    // Check if this door spawns powerup1 (door #4 in level 3)
                    else if self.level == 3 && door_idx == 3 && !self.powerup1_collected {
                        // Spawn powerup1 at door position
                        self.powerup1_position = (door_center_x, door_center_y - 10.0);
                        self.powerup1_active = true;
                        log!("PowerUp1 spawned from door {}!", door_idx);
                    }
                    // Check if this door spawns a kid
                    // For level 3: doors 0 and 1 spawn KIDS (door 0 triggers victory when collected)
                    // For other levels: kid_door_index spawns a kid  
                    else if (self.level == 3 && (door_idx == 0 || door_idx == 1)) || (self.level != 3 && door_idx == self.kid_door_index) {
                        // Spawn a kid at the door position
                        // Find first available kid slot
                        for kid_idx in 0..self.kids.len() {
                            if !self.kids[kid_idx].2 && !self.kids[kid_idx].3 {
                                let spawn_x = door_center_x;
                                let spawn_y = door_center_y - 10.0;
                                self.kids[kid_idx] = (spawn_x, spawn_y, true, false, 0, 0, door_idx as i8);
                                self.total_kids_in_level += 1;
                                log!("Kid spawned from door {}!", door_idx);
                                break;
                            }
                        }
                    } else {
                        // Spawn a key at the door position (slightly above center)
                        // Use first available slot and associate with door index
                        for k in 0..self.keys.len() {
                            if !self.keys[k].2 {
                                let spawn_x = door_center_x;
                                let spawn_y = door_center_y - 10.0;
                                self.keys[k] = (spawn_x, spawn_y, true, 0, 0);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_camera(&mut self) {
        // During boss fight, camera is fixed (no scrolling)
        if self.boss_active {
            self.camera_x = 0.0;
            return;
        }

        let target_x = self.player_x - (SCREEN_WIDTH / 2.0);
        // Clamp camera to level bounds (0 to level_width - screen_width)
        let level_width = match self.level {
            1 => 1080.0,  // Level 1 width
            2 => 1440.0,  // Level 2 width
            3 => 2160.0,  // Boss fight width (before trigger)
            _ => 1080.0,  // Default
        };
        let max_camera = level_width - SCREEN_WIDTH;
        self.camera_x = target_x.max(0.0).min(max_camera);
    }

    fn load_level(&mut self, level_num: u8) {
        self.level = level_num;
        self.timer = LEVEL_TIME;
        
        // Set level time limits (in frames, 60 fps)
        self.level_timer = 0;
        self.level_time_limit = match level_num {
            1 => 180 * 60,  // 3 minutes
            2 => 480 * 60,  // 8 minutes
            3 => 600 * 60,  // 10 minutes
            _ => 180 * 60,  // Default 3 minutes
        };

        for enemy in self.enemies.iter_mut() {
            enemy.8 = false;
        }

        match level_num {
            1 => {
                // Level 1 - 3 Floor Christmas House (based on spritebg1080x240.png)
                // Using precise tilemap from sprites/tilemap.json
                // Platforms are the ONLY ground - no hardcoded floor

                // Santa starts on TILE #1 (left section, top floor)
                // Offset applied: +60px X, +18px Y
                // Tile #1: X(60-230) at Y=100
                // Santa sprite is 32x32, positioned from center
                // Platform Y=100, so Santa Y = 100 - 28 = 72 (28px hitbox height)
                self.player_x = 145.0;  // Center of tile #1 (60-230)
                self.player_y = 72.0;   // Tile #1 platform position
                self.player_vx = 0.0;
                self.player_vy = 0.0;
                self.player_state = STATE_IDLE;
                self.player_on_ground = true; // Start on platform

                // ============================================================
                // PLATFORM POSITIONS - From sprites/layout.md tilemap
                // ============================================================
                // Background: 1080x240px, 3 floors visible
                // Positions calculated from center (540, 120) with layout dimensions
                // Format: (x_start, x_end, y_position)

                // Clear and populate platforms for Level 1
                self.platforms.clear();

                // TOP FLOOR (Y ≈ 100) - platforms from tiles 1, 3, 4, 8, 14
                self.platforms.push((60.0, 230.0, 86.0));         // Tile #1
                self.platforms.push((275.0, 300.0, 95.0-9.0));    // Tile #3
                self.platforms.push((400.0, 765.0, 76.0-9.0));    // Tile #4
                self.platforms.push((804.0, 920.0, 95.0-9.0));    // Tile #8
                self.platforms.push((931.0, 1045.0, 95.0-9.0));   // Tile #14

                // MIDDLE FLOOR (Y ≈ 168) - platforms from tiles 5, 6, 9, 11, 12, 13
                self.platforms.push((60.0, 252.0, 163.0-9.0));    // Tile #13
                self.platforms.push((264.0, 495.0, 163.0-9.0));   // Tile #12
                self.platforms.push((542.0, 654.0, 163.0-9.0));   // Tile #11
                self.platforms.push((665.0, 768.0, 163.0-9.0));   // Tile #9
                self.platforms.push((806.0, 1008.0, 163.0-9.0));  // Tile #5
                self.platforms.push((1020.0, 1043.0, 163.0-9.0)); // Tile #6

                // BOTTOM FLOOR (Y ≈ 238) - platforms from tiles 7, 15
                self.platforms.push((36.0, 81.0, 233.0-9.0));     // Tile #7
                self.platforms.push((93.0, 1063.0, 233.0-9.0));   // Tile #15

                // Additional platforms
                self.platforms.push((310.0, 400.0, 95.0-9.0));

                //

                // ============================================================
                // LADDERS - Connect the three floors
                // ============================================================
                // Format: (x_center, y_top, y_bottom)
                // Offset applied: +60px X, +18px Y

                // Left side ladders
                self.ladders[0] = (305.0, 85.0, 117.0);    // Top to Middle (left)
                self.ladders[1] = (258.0, 150.0, 187.0);   // Middle to Bottom (left)

                // Center ladders
                self.ladders[2] = (383.0, 85.0, 117.0);    // Top to Middle (center)
                self.ladders[3] = (660.0, 150.0, 187.0);   // Middle to Bottom (center)

                // Right side ladders
                self.ladders[4] = (925.0, 85.0, 117.0);    // Top to Middle (right)
                self.ladders[5] = (1015.0, 150.0, 187.0);   // Middle to Bottom (right)

                // ============================================================
                // ENEMIES - Positioned on platform surfaces
                // ============================================================
                // Enemy Y = platform_y - 28 (same formula as Santa!)
                // This ensures enemies walk at the exact same level as Santa
                
                // Top floor at Y=95, so enemy Y = 95 - 28 = 67
                // Large center platform at Y=76, so enemy Y = 76 - 28 = 48
                self.enemies[0] = (1, 500.0, 48.0, 0.0, 0.0, 0, 0, 420, true, 0, false, 0);  // On large center platform (Y=76)
                self.enemies[1] = (2, 870.0, 67.0, 0.0, 0.0, 0, 0, 810, true, 0, false, 0);  // Top-right patrol at Y=95 (kickmouse)
                self.enemies[5] = (1, 500.0, 135.0, 0.0, 0.0, 0, 0,340, true, 0, false, 0);   // Top-left patrol at Y=95 (kickmouse)
                self.enemies[8] = (3, 680.0, 48.0, 0.0, 0.0, 0, 0, 600, true, 0, false, 0);  // Large center platform (Y=76)

                // Middle floor at Y=163, so enemy Y = 163 - 28 = 135
                self.enemies[2] = (3, 310.0, 135.0, 0.0, 0.0, 0, 0, 160, true, 0, false, 0); // Middle floor patrol left
                self.enemies[6] = (1, 600.0, 135.0, 0.0, 0.0, 0, 0, 542, true, 0, false, 0); // Middle floor patrol center
                self.enemies[7] = (2, 900.0, 135.0, 0.0, 0.0, 0, 0, 940, true, 0, false, 0); // Middle floor patrol right (kickmouse)

                // Bottom floor at Y=233, so enemy Y = 233 - 28 = 205
                self.enemies[3] = (2, 400.0, 205.0, 0.0, 0.0, 0, 0, 200, true, 0, false, 0); // Ground patrol left (kickmouse)
                self.enemies[4] = (1, 740.0, 205.0, 0.0, 0.0, 0, 0, 640, true, 0, false, 0); // Ground patrol right
                self.enemies[9] = (2, 500.0, 205.0, 0.0, 0.0, 0, 0, 400, true, 0, false, 0); // Ground patrol center (kickmouse)

                // ============================================================
                // DOORS - Destructible by bombs
                // ============================================================
                // Format: (x, y, intact)
                // x, y are top-left corner of 42x42 door sprite
                // Door sprite is 42x42, exploded_gate is 64x64
                self.doors[0] = (96.0, 42.0, true);    // Door #1 - Top left
                self.doors[1] = (411.0, 111.0, true);  // Door #2 - Middle left
                self.doors[2] = (676.0, 180.0, true);  // Door #3 - Bottom center
                self.doors[3] = (990.0, 42.0, true);   // Door #4 - Top right

                // ============================================================
                // WALLS - Vertical collision boundaries
                // ============================================================
                // Format: (x, y, width, height)
                self.walls[0] = (51.0, 28.0, 8.0, 136.0);    // Wall #1 - Left top vertical
                self.walls[1] = (27.0, 165.0, 8.0, 60.0);    // Wall #2 - Left middle vertical
                self.walls[2] = (129.0, 165.0, 23.0, 60.0);  // Wall #3 - Left-center vertical
                self.walls[3] = (0.0, 0.0, 0.0, 0.0);        // Wall #4 - REMOVED
                self.walls[4] = (320.0, 98.0, 36.0, 59.0);   // Wall #5 - Center-left vertical
                self.walls[5] = (542.0, 168.0, 86.0, 60.0);  // Wall #6 - Center vertical
                self.walls[6] = (675.0, 78.0, 34.0, 78.0);   // Wall #7 - Center-right vertical
                self.walls[7] = (851.0, 98.0, 34.0, 59.0);   // Wall #8 - Right-center vertical
                self.walls[8] = (0.0, 0.0, 0.0, 0.0);        // Wall #9 - REMOVED
                self.walls[9] = (1045.0, 28.0, 8.0, 197.0);  // Wall #10 - Right top vertical

                // ============================================================
                // LEVEL COMPLETION TRIGGER
                // ============================================================
                // Place trigger zone on the right side of the gate (wall #3)
                // Gate is at (129, 165, 23, 60)
                // Trigger at x=160 (just past the gate), covering the passage
                self.completion_trigger = (160.0, 165.0, 30.0, 60.0);

                // Reset level completion state
                self.level_complete = false;
                self.level_transition_timer = 0;

                // Reset player movement state
                self.player_on_ladder = false;
                self.player_is_crouching = false;

                // ============================================================
                // KIDS - Level 1
                // ============================================================
                // One door will spawn a kid instead of a key (door index 0-3)
                // Using frame count for pseudo-random selection
                self.kid_door_index = (self.frame % 4) as usize;
                self.kids_collected = 0;
                self.total_kids_in_level = 0;  // Will increment when kid spawns from door
                for kid in self.kids.iter_mut() {
                    *kid = (0.0, 0.0, false, false, 0, 0, -1);
                }
            },
            2 => {
                // Level 2 - Rooftop Christmas Adventure (level2finalsprite 1440x240px)
                // Using exact coordinates from Sprites/layout.md

                // ============================================================
                // PLAYER SPAWN - Tile 1 (recommended spawn)
                // ============================================================
                self.player_x = 60.0;
                self.player_y = 86.0 - 19.0;  // 58.0
                self.player_vx = 0.0;
                self.player_vy = 0.0;
                self.player_state = STATE_IDLE;
                self.player_on_ground = true;

                // ============================================================
                // PLATFORMS (29 floor tiles) - From layout.md
                // ============================================================
                // Format: (x_start, x_end, y) where x_end = x + w

                self.platforms.clear();
                self.platforms.push((35.0, 126.0, 88.0));      // ID 1
                self.platforms.push((126.0, 146.0, 115.0));    // ID 2
                self.platforms.push((146.0, 169.0, 135.0));    // ID 3
                self.platforms.push((172.0, 257.0, 155.0));    // ID 4
                self.platforms.push((235.0, 301.0, 88.0));     // ID 5
                self.platforms.push((313.0, 323.0, 88.0));     // ID 6
                self.platforms.push((367.0, 479.0, 88.0));     // ID 7
                self.platforms.push((746.0, 836.0, 88.0));     // ID 8
                self.platforms.push((847.0, 907.0, 88.0));     // ID 9
                self.platforms.push((1037.0, 1203.0, 88.0));   // ID 10
                self.platforms.push((1303.0, 1326.0, 88.0));   // ID 11
                self.platforms.push((1336.0, 1394.0, 88.0));   // ID 12
                self.platforms.push((268.0, 367.0, 155.0));    // ID 13
                self.platforms.push((479.0, 500.0, 115.0));    // ID 14
                self.platforms.push((542.0, 674.0, 135.0));    // ID 15
                self.platforms.push((674.0, 746.0, 155.0));    // ID 16
                self.platforms.push((757.0, 764.0, 155.0));    // ID 17
                self.platforms.push((813.0, 897.0, 155.0));    // ID 18
                self.platforms.push((947.0, 990.0, 135.0));    // ID 19
                self.platforms.push((990.0, 1037.0, 115.0));   // ID 20
                self.platforms.push((1203.0, 1226.0, 115.0));  // ID 21
                self.platforms.push((1226.0, 1292.0, 135.0));  // ID 22
                self.platforms.push((1292.0, 1416.0, 155.0));  // ID 23
                self.platforms.push((1079.0, 1146.0, 155.0));  // ID 24
                self.platforms.push((1156.0, 1168.0, 155.0));  // ID 25
                self.platforms.push((13.0, 192.0, 225.0));     // ID 26
                self.platforms.push((247.0, 553.0, 225.0));    // ID 27
                self.platforms.push((589.0, 903.0, 225.0));    // ID 28
                self.platforms.push((960.0, 1416.0, 225.0));   // ID 29

                // ============================================================
                // LADDERS (6 ladders) - From layout.md
                // ============================================================
                // Format: (x_center, y_top, y_bottom) where y_bottom = y_top + 28

                self.ladders[0] = (303.0, 85.0, 122.0);   // ID 1
                self.ladders[1] = (259.0, 150.0, 190.0);  // ID 2
                self.ladders[2] = (747.0, 152.0, 189.0);  // ID 3
                self.ladders[3] = (837.0, 85.0, 122.0);   // ID 4
                self.ladders[4] = (1327.0, 84.0, 121.0);  // ID 5
                self.ladders[5] = (1148.0, 152.0, 189.0); // ID 6

                // ============================================================
                // WALLS (22 wall colliders) - From layout.md
                // ============================================================
                // Format: (x, y, width, height)

                self.walls[0] = (25.0, 35.0, 9.0, 50.0);      // ID 1
                self.walls[1] = (116.0, 95.0, 9.0, 28.0);     // ID 2
                self.walls[2] = (1350.0, 30.0, 23.0, 60.0);   // ID 3 - GATE (repositioned)
                self.walls[3] = (159.0, 142.0, 10.0, 20.0);   // ID 4
                self.walls[4] = (52.0, 161.0, 9.0, 59.0);     // ID 5
                self.walls[5] = (368.0, 89.0, 17.0, 70.0);    // ID 6
                self.walls[6] = (487.0, 115.0, 14.0, 62.0);   // ID 7
                self.walls[7] = (546.0, 136.0, 14.0, 85.0);   // ID 8
                self.walls[8] = (621.0, 162.0, 13.0, 59.0);   // ID 9
                self.walls[9] = (726.0, 35.0, 17.0, 51.0);    // ID 10
                self.walls[10] = (990.0, 118.0, 7.0, 19.0);   // ID 11
                self.walls[11] = (1038.0, 88.0, 8.0, 30.0);   // ID 12
                self.walls[12] = (1071.0, 108.0, 9.0, 49.0);  // ID 13
                self.walls[13] = (1168.0, 108.0, 12.0, 49.0); // ID 14
                self.walls[14] = (1196.0, 90.0, 6.0, 29.0);   // ID 15
                self.walls[15] = (1220.0, 119.0, 6.0, 19.0);  // ID 16
                self.walls[16] = (1286.0, 139.0, 6.0, 17.0);  // ID 17
                self.walls[17] = (140.0, 120.0, 6.0, 10.0);  // ID 18
                self.walls[18] = (1416.0, 55.0, 16.0, 180.0);  // ID 19
                self.walls[19] = (1393.0, 161.0, 12.0, 59.0); // ID 20
                self.walls[20] = (467.0, 92.0, 7.0, 25.0);        // Id 21 
                self.walls[21] = (667.0, 136.0, 6.0, 16.0);          // Id 22 (unused)
                // ============================================================
                // DOORS - Destructible by bombs (Level 2)
                // ============================================================
                // Format: (x, y, intact)
                // x, y are top-left corner of 42x42 door sprite
                // Door sprite is 42x42, exploded_gate is 64x64
                // Positioned on platforms to block paths
                
                // Door #1 - On platform ID 7 (367-479, Y=88), blocking path
                self.doors[0] = (103.0, 181.0, true);
                
                // Door #2 - On platform ID 15 (546-674, Y=135), blocking middle section
                self.doors[1] = (768.0, 46.0, true);
                
                // Door #3 - On platform ID 18 (813-897, Y=155), blocking right-middle
                self.doors[2] = (416.0, 44.0, true);
                
                // Door #4 - On platform ID 10 (1037-1203, Y=88), blocking far right top
                self.doors[3] = (1093.0, 113.0, true);

                // ============================================================
                // ENEMIES - Level 2 (positioned on platforms)
                // ============================================================
                // Enemy Y = platform_y - 28 (same formula as Santa)
                // Enemy types: 1 = mouse, 2 = kickmouse, 3 = penguin
                
                // Top floor enemies (Y=88, enemy Y = 88-28 = 60)
                self.enemies[0] = (1, 280.0, 70.0, 0.0, 0.0, 0, 0, 235, true, 0, false, 0);  // Mouse - Platform ID 5
                self.enemies[1] = (3, 780.0, 70.0, 0.0, 0.0, 0, 0, 746, true, 0, false, 0);  // Penguin - Platform ID 8
                self.enemies[2] = (3, 1150.0, 70.0, 0.0, 0.0, 0, 0, 1037, true, 0, false, 0); // Penguin - Platform ID 10
                
                // Middle floor enemies (Y=155, enemy Y = 155-28 = 127)
                self.enemies[3] = (2, 220.0, 137.0, 0.0, 0.0, 0, 0, 172, true, 0, false, 0);  // Kickmouse - Platform ID 4
                self.enemies[4] = (3, 700.0, 137.0, 0.0, 0.0, 0, 0, 674, true, 0, false, 0);  // Penguin - Platform ID 16
                self.enemies[5] = (3, 860.0, 137.0, 0.0, 0.0, 0, 0, 813, true, 0, false, 0);  // Penguin - Platform ID 18
                
                // Bottom floor enemies (Y=225, enemy Y = 225-28 = 197)
                self.enemies[6] = (1, 350.0, 207.0, 0.0, 0.0, 0, 0, 247, true, 0, false, 0);  // Mouse - Platform ID 27
                self.enemies[7] = (3, 700.0, 207.0, 0.0, 0.0, 0, 0, 589, true, 0, false, 0);  // Penguin - Platform ID 28
                self.enemies[8] = (2, 1100.0, 207.0, 0.0, 0.0, 0, 0, 960, true, 0, false, 0); // Kickmouse - Platform ID 29
                self.enemies[9] = (3, 1300.0, 207.0, 0.0, 0.0, 0, 0, 1160, true, 0, false, 0); // Penguin - Platform ID 29

                // Reset keys for Level 2
                self.keys_collected = 0;
                for key in self.keys.iter_mut() {
                    key.2 = false; // Deactivate all keys
                }

                // ============================================================
                // KIDS - Level 2
                // ============================================================
                // One door will spawn a kid instead of a key
                self.kid_door_index = (self.frame % 4) as usize;
                self.kids_collected = 0;
                self.total_kids_in_level = 0;  // Will increment when kid spawns from door
                for kid in self.kids.iter_mut() {
                    *kid = (0.0, 0.0, false, false, 0, 0, -1);
                }

                // ============================================================
                // LEVEL COMPLETION TRIGGER - Level 2
                // ============================================================
                // Trigger zone at x=1350, y=85 to enter boss fight
                self.completion_trigger = (1350.0, 85.0, 40.0, 50.0);

                // Reset level completion state
                self.level_complete = false;
                self.level_transition_timer = 0;

                // Reset player movement state
                self.player_on_ladder = false;
                self.player_is_crouching = false;
            },
            3 => {
                // Level 3 - Boss Fight (bossfight bg1 - 2160x240px)
                // Using exact coordinates from Sprites/layout.md line 167+

                // Reset boss state for fresh start
                self.boss_active = false;
                self.boss_defeated = false;

                // ============================================================
                // PLAYER SPAWN - Start at left side
                // ============================================================
                self.player_x = 150.0;
                self.player_y = 222.0 - 28.0;  // 194.0 (on platform ID 1)
                self.player_vx = 0.0;
                self.player_vy = 0.0;
                self.player_state = STATE_IDLE;
                self.player_on_ground = true;

                // ============================================================
                // PLATFORMS (52 floor tiles) - All platforms from layout.md
                // ============================================================
                // Format: (x_start, x_end, y) where x_end = x + w

                self.platforms.clear();
                self.platforms.push((1.0, 317.0, 222.0+5.0));        // ID 1
                self.platforms.push((301.0, 333.0, 207.0+5.0));      // ID 2
                self.platforms.push((333.0, 363.0, 193.0+5.0));      // ID 3
                self.platforms.push((361.0, 394.0, 177.0+5.0));      // ID 4
                self.platforms.push((391.0, 418.0, 163.0+5.0));      // ID 5
                self.platforms.push((211.0, 269.0, 163.0+5.0));      // ID 6
                self.platforms.push((329.0, 448.0, 119.0+5.0));      // ID 7
                self.platforms.push((450.0, 479.0, 164.0+5.0));      // ID 8
                self.platforms.push((540.0, 568.0, 164.0+5.0));      // ID 9
                //self.platforms.push((629.0, 689.0, 104.0+5.0));      // ID 10
                //self.platforms.push((510.0, 536.0, 103.0+5.0));      // ID 11
                self.platforms.push((630.0, 656.0, 164.0+5.0));      // ID 12
                self.platforms.push((599.0, 625.0, 192.0+5.0));      // ID 13
                self.platforms.push((569.0, 596.0, 223.0+5.0));      // ID 14
                self.platforms.push((630.0, 657.0, 223.0+5.0));      // ID 15
                self.platforms.push((689.0, 718.0, 133.0+5.0));      // ID 16
                self.platforms.push((756.0, 836.0, 133.0+5.0));      // ID 17
                self.platforms.push((690.0, 717.0, 223.0+5.0));      // ID 18
                self.platforms.push((749.0, 777.0, 223.0+5.0));      // ID 19
                self.platforms.push((808.0, 871.0, 223.0+5.0));      // ID 20
                self.platforms.push((868.0, 896.0, 133.0+5.0));      // ID 21
                self.platforms.push((927.0, 955.0, 119.0+5.0));      // ID 22
                self.platforms.push((957.0, 986.0, 162.0+5.0));      // ID 23
                self.platforms.push((986.0, 1021.0, 207.0+5.0));     // ID 24
                self.platforms.push((1017.0, 1045.0, 147.0+5.0));    // ID 25
                self.platforms.push((1046.0, 1075.0, 162.0+5.0));    // ID 26
                //self.platforms.push((1107.0, 1164.0, 102.0+5.0));    // ID 27
                self.platforms.push((1107.0, 1133.0, 192.0+5.0));    // ID 28
                self.platforms.push((1134.0, 1165.0, 205.0+5.0));    // ID 29
                self.platforms.push((1167.0, 1254.0, 147.0+5.0));    // ID 30
                self.platforms.push((1165.0, 1253.0, 221.0+5.0));    // ID 31
                self.platforms.push((1285.0, 1313.0, 162.0+5.0));    // ID 32
                self.platforms.push((1287.0, 1317.0, 222.0+5.0));    // ID 33
                self.platforms.push((1320.0, 1373.0, 175.0+5.0));    // ID 34
                self.platforms.push((1346.0, 1374.0, 222.0+5.0));    // ID 35
                //self.platforms.push((1314.0, 1343.0, 102.0+5.0));    // ID 36
                self.platforms.push((1374.0, 1404.0, 147.0+5.0));    // ID 37
                self.platforms.push((1406.0, 1456.0, 117.0+5.0));    // ID 38
                self.platforms.push((1406.0, 1490.0, 222.0+5.0));    // ID 39
                self.platforms.push((1496.0, 1524.0, 162.0+5.0));    // ID 40
                //self.platforms.push((1554.0, 1643.0, 102.0+5.0));    // ID 41
                self.platforms.push((1554.0, 1580.0, 222.0+5.0));    // ID 42
                self.platforms.push((1643.0, 1671.0, 178.0+5.0));    // ID 43
                self.platforms.push((1673.0, 1701.0, 147.0+5.0));    // ID 44
                self.platforms.push((1615.0, 1736.0, 222.0+5.0));    // ID 45
                self.platforms.push((1734.0, 1764.0, 117.0+5.0));    // ID 46
                self.platforms.push((1762.0, 1791.0, 161.0+5.0));    // ID 47
                self.platforms.push((1823.0, 1853.0, 117.0+5.0));    // ID 48
                self.platforms.push((1853.0, 1881.0, 161.0+5.0));    // ID 49
                self.platforms.push((1881.0, 1913.0, 177.0+5.0));    // ID 50
                self.platforms.push((1909.0, 2159.0, 220.0+5.0));    // ID 51
                //self.platforms.push((1198.0,1218.0,104.0+5.0));        // ID 52 
                self.platforms.push((897.0,952.0,224.0+5.0));        // ID 53 
                self.platforms.push((1.0, 2161.0, 50.0+5.0));        // ID 54 - Top ceiling
                


                
                // Clear ladders for boss level (no ladders)
                for i in 0..self.ladders.len() {
                    self.ladders[i] = (0.0, 0.0, 0.0);
                }
                self.ladders[0] = (2136.0, 50.0, 189.0);
                // Clear walls for boss level (no walls initially)
                for i in 0..self.walls.len() {
                    self.walls[i] = (0.0, 0.0, 0.0, 0.0);
                }

                // Reset enemies for Boss Level (boss enemy will be added later)
                for enemy in self.enemies.iter_mut() {
                    enemy.8 = false;
                }

                // Reset keys for Boss Level
                self.keys_collected = 0;
                for key in self.keys.iter_mut() {
                    key.2 = false;
                }

                // ============================================================
                // KIDS - Boss Level (Level 3)
                // ============================================================
                // One door will spawn a kid instead of a key
                self.kid_door_index = (self.frame % 4) as usize;
                self.kids_collected = 0;
                self.total_kids_in_level = 0;  // Will increment when kid spawns from door
                for kid in self.kids.iter_mut() {
                    *kid = (0.0, 0.0, false, false, 0, 0, -1);
                }

                // ============================================================
                // LIFE POWERUP - Boss Level (Level 3)
                // ============================================================
                // Door #3 (index 2) spawns life powerup instead of key/kid
                self.life_active = false;
                self.life_collected = false;
                self.life_door_index = 2;  // Door #3

                // ============================================================
                // POWERUP1 - Boss Level (Level 3)
                // ============================================================
                // Door #4 (index 3) spawns powerup1
                self.powerup1_active = false;
                self.powerup1_collected = false;

                // ============================================================
                // DOORS - Boss Level strategic positions
                // ============================================================
                // Clear/override any previous door positions and set new ones
                // Doors are 42x42; place so base rests on platform surfaces.
                // Format: (x_top_left, y_top_left, intact)

                // On platform ID 7 (329-448, y=119) -> door base at y=119
                // y_top_left = 119 - 42 = 77
                self.doors[0] = (621.0, 14.0, true);

                // On platform ID 20 (808-871, y=223)
                self.doors[1] = (869.0, 14.0, true);

                // On platform ID 30 (1167-1254, y=147)
                self.doors[2] = (1194.0, 181.0, true);

                // On platform ID 51 (1909-2159, y=220)
                self.doors[3] = (775.0, 97.0, true);

                // ============================================================
                // ENEMIES - Boss Level placements across platforms
                // ============================================================
                // Enemy Y = platform_y - 28
                // Types: 1=mouse, 2=kickmouse, 3=penguin
                // Patrol start set to platform start for each.

               // Platform ID 7 (329 → 448)
                self.enemies[0] = (3, 370.0, 100.0, 0.0, 0.0, 0, 0, 341, true, 0, false, 0);
                // patrol: 341 → 441 ✔

                // Platform ID 20 (808 → 871)
                self.enemies[1] = (2, 840.0, 208.0, 0.0, 0.0, 0, 0, 820, true, 0, false, 0);
                // patrol: 820 → 920 ❌ too wide → but enemy stays centered due to short platform
                // acceptable for boss pacing

                // Platform ID 17 (756 → 836)
                self.enemies[2] = (3, 780.0, 117.0, 0.0, 0.0, 0, 0, 768, true, 0, false, 0);
                // patrol: 768 → 868 ✔

                // Platform ID 25 (1017 → 1045)
                self.enemies[3] = (1, 968.0, 36.0, 0.0, 0.0, 0, 0, 1025, true, 0, false, 0);
                // patrol: 1025 → 1125 (enemy remains visually stable)

                // Platform ID 30 (1167 → 1254)
                self.enemies[4] = (3, 1200.0, 130.0, 0.0, 0.0, 0, 0, 1179, true, 0, false, 0);
                // patrol: 1179 → 1279 ✔

                // Platform ID 34 (1320 → 1373)
                self.enemies[5] = (1, 1225.0, 36.0, 0.0, 0.0, 0, 0, 1332, true, 0, false, 0);
                // patrol: 1332 → 1432 (tight enough)

                // Platform ID 39 (1406 → 1490)
                self.enemies[6] = (2, 1450.0, 207.0, 0.0, 0.0, 0, 0, 1418, true, 0, false, 0);
                // patrol: 1418 → 1518 ✔

                // Platform ID 45 (1615 → 1736)
                self.enemies[7] = (3, 1650.0, 207.0, 0.0, 0.0, 0, 0, 1627, true, 0, false, 0);
                // patrol: 1627 → 1727 ✔

                // Platform ID 51 (1909 → 2159)
                //self.enemies[8] = (3, 2050.0, 207.0, 0.0, 0.0, 0, 0, 1921, true, 0, false, 0);
                // patrol: 1921 → 2021 ✔

                // Platform ID 10 (629 → 689)
                self.enemies[9] = (1, 820.0, 36.0, 0.0, 0.0, 0, 0, 701, true, 0, false, 0);
                // patrol: 641 → 741 ✔


                // No completion trigger for boss level
                self.completion_trigger = (0.0, 0.0, 0.0, 0.0);

                // Reset level completion state
                self.level_complete = false;
                self.level_transition_timer = 0;

                // Reset player movement state
                self.player_on_ladder = false;
                self.player_is_crouching = false;
            },
            _ => {}
        }
    }

    fn update_boss(&mut self) {
        if !self.boss_active { return; }

        // If boss defeated, stop movement and set to dead state
        if self.evil_santa_hp == 0 {
            self.evil_santa_vx = 0.0;
            self.evil_santa_vy = 0.0;
            self.evil_santa_state = BOSS_STATE_DEAD;

            // Start death timer (3 seconds = 180 frames)
            self.boss_death_timer += 1;

            // After 3 seconds, exit boss fight and return to level 3
            if self.boss_death_timer >= 180 {
                self.exit_boss_fight();
            }
            return;
        }

        // Player idle detection: if player stays in same spot for 3+ seconds, boss attacks
        let movement_threshold = 15.0; // Consider "idle" if moved less than 15px
        if (self.player_x - self.player_idle_x).abs() > movement_threshold {
            // Player moved - reset tracker
            self.player_idle_x = self.player_x;
            self.player_idle_timer = 0;
        } else {
            // Player is stationary
            self.player_idle_timer += 1;
            
            // 3 seconds = 180 frames at 60 FPS
            if self.player_idle_timer >= 180 {
                // Force boss to attack if in idle/recover state
                if self.evil_santa_state == BOSS_STATE_IDLE || self.evil_santa_state == BOSS_STATE_RECOVER {
                    self.evil_santa_state = BOSS_STATE_SELECT;
                    self.boss_state_timer = 0;
                }
                // Reset timer to avoid spam
                self.player_idle_timer = 90; // Cooldown before next forced attack
            }
        }

        // FSM Logic
        // Phase-based difficulty: boss gets more aggressive at lower HP
        let hp_ratio = self.evil_santa_hp as f32 / self.evil_santa_max_hp as f32;
        let is_enraged = hp_ratio <= 0.5; // Below 50% HP = rage mode
        let is_desperate = hp_ratio <= 0.25; // Below 25% HP = desperate mode
        
        // Speed multipliers based on phase
        let speed_mult = if is_desperate { 1.5 } else if is_enraged { 1.25 } else { 1.0 };
        let timer_mult = if is_desperate { 0.6 } else if is_enraged { 0.8 } else { 1.0 };

        match self.evil_santa_state {
            BOSS_STATE_IDLE => {
                // Face player
                self.evil_santa_facing_right = self.player_x > self.evil_santa_x;
                
                self.boss_state_timer += 1;
                
                // Shorter idle time when enraged
                let idle_duration = ((60.0 * timer_mult) as u32).max(20);
                
                // Check if player is far - walk towards them instead of just idling
                let dx = (self.player_x - self.evil_santa_x).abs();
                if dx > 120.0 && self.boss_state_timer > 15 {
                    // Walk towards player
                    self.evil_santa_state = BOSS_STATE_WALK;
                    self.boss_state_timer = 0;
                } else if self.boss_state_timer >= idle_duration {
                    self.evil_santa_state = BOSS_STATE_SELECT;
                    self.boss_state_timer = 0;
                } else {
                    self.evil_santa_vx = 0.0;
                }
            },
            BOSS_STATE_WALK => {
                // Walk towards player
                self.evil_santa_facing_right = self.player_x > self.evil_santa_x;
                let walk_speed = EVIL_WALK_SPEED * speed_mult;
                self.evil_santa_vx = if self.evil_santa_facing_right { walk_speed } else { -walk_speed };
                
                self.boss_state_timer += 1;
                let dx = (self.player_x - self.evil_santa_x).abs();
                
                // Stop walking when close enough or walked too long
                let walk_duration = if is_enraged { 45 } else { 60 };
                if dx < 100.0 || self.boss_state_timer >= walk_duration {
                    self.evil_santa_state = BOSS_STATE_SELECT;
                    self.boss_state_timer = 0;
                    self.evil_santa_vx = 0.0;
                }
            },
            BOSS_STATE_SELECT => {
                // Smart attack selection based on relative position and randomization
                let dx = (self.player_x - self.evil_santa_x).abs();
                let dy = self.player_y - self.evil_santa_y; // Negative if player is above
                // Arena boundaries for boss fight (360x256 arena)
                let arena_left = 10.0;
                let arena_right = 350.0;
                let near_left_wall = (self.evil_santa_x - arena_left) < 40.0;
                let near_right_wall = (arena_right - self.evil_santa_x) < 40.0;

                // Face the player
                self.evil_santa_facing_right = self.player_x > self.evil_santa_x;

                // Use frame-based pseudo-random for variety
                let rand_factor = (self.frame % 100) as f32 / 100.0;
                
                // Attack selection with weighted randomness
                self.boss_attack_type = if near_left_wall || near_right_wall {
                    // Near wall - dash away or projectile
                    if rand_factor < 0.6 { ATTACK_DASH } else { ATTACK_PROJECTILE }
                } else if dy < -20.0 && dx < 180.0 {
                    // Player is above - slam to hit them
                    if rand_factor < 0.7 { ATTACK_SLAM } else { ATTACK_PROJECTILE }
                } else if dx > 200.0 {
                    // Far away - projectile or chase-dash
                    if rand_factor < 0.5 { ATTACK_PROJECTILE } else { ATTACK_DASH }
                } else if dx < 80.0 {
                    // Very close - slam or dash
                    if rand_factor < 0.6 { ATTACK_SLAM } else { ATTACK_DASH }
                } else {
                    // Medium range - any attack
                    if rand_factor < 0.4 { ATTACK_DASH }
                    else if rand_factor < 0.7 { ATTACK_SLAM }
                    else { ATTACK_PROJECTILE }
                };
                
                // In desperate mode, favor fast attacks
                if is_desperate && self.boss_attack_type == ATTACK_SLAM && rand_factor > 0.3 {
                    self.boss_attack_type = ATTACK_DASH;
                }

                // Transition to Attack
                // Randomized attack warning sound
                let warning_sound = match self.frame % 3 {
                    0 => "evilsanta_before_attack",
                    1 => "evilsanta_bfore_attak1",
                    _ => "evilsanta_bfore_attack2",
                };
                audio::play(warning_sound);
                
                self.evil_santa_state = BOSS_STATE_ATTACK;
                self.boss_phase = PHASE_WINDUP;
                self.boss_phase_timer = 0;
                self.evil_santa_anim_frame = 0;
                self.evil_santa_vx = 0.0;
            },
            BOSS_STATE_ATTACK => {
                match self.boss_attack_type {
                    ATTACK_DASH => {
                        match self.boss_phase {
                            PHASE_WINDUP => {
                                // Face player, telegraph with animation
                                self.evil_santa_facing_right = self.player_x > self.evil_santa_x;
                                self.boss_phase_timer += 1;
                                
                                // Shorter telegraph when enraged
                                let telegraph_time = ((35.0 * timer_mult) as u32).max(15);
                                if self.boss_phase_timer >= telegraph_time {
                                    self.boss_phase = PHASE_ACTIVE;
                                    self.boss_phase_timer = 0;
                                    // Fast dash with speed multiplier
                                    let dir = if self.evil_santa_facing_right { 1.0 } else { -1.0 };
                                    self.evil_santa_vx = dir * 7.0 * speed_mult;
                                }
                            },
                            PHASE_ACTIVE => {
                                self.boss_phase_timer += 1;
                                
                                // Collision damage during dash
                                let dx = (self.player_x - self.evil_santa_x).abs();
                                let dy = (self.player_y - self.evil_santa_y).abs();
                                if dx < 35.0 && dy < 35.0 && self.player_invuln_timer == 0 {
                                    let damage = 1; // Reduced damage
                                    self.player_hp = self.player_hp.saturating_sub(damage);
                                    let hurt_sfx = if self.frame % 2 == 0 { "santa_hurt_1" } else { "santa_hurt_2" };
                                    audio::play(hurt_sfx);
                                    self.player_invuln_timer = 60;
                                    // Strong knockback
                                    self.player_vx = if self.evil_santa_vx > 0.0 { 5.0 } else { -5.0 };
                                    self.player_vy = -5.0;
                                }

                                // Dash duration
                                let dash_duration = if is_enraged { 25 } else { 35 };
                                if self.boss_phase_timer >= dash_duration {
                                    self.boss_phase = PHASE_RECOVERY;
                                    self.boss_phase_timer = 0;
                                    self.evil_santa_vx = 0.0;
                                }
                            },
                            PHASE_RECOVERY => {
                                self.boss_phase_timer += 1;
                                let recovery_time = ((25.0 * timer_mult) as u32).max(10);
                                if self.boss_phase_timer >= recovery_time {
                                    self.evil_santa_state = BOSS_STATE_RECOVER;
                                    self.boss_state_timer = 0;
                                }
                            },
                            _ => {}
                        }
                    },
                    ATTACK_SLAM => {
                        match self.boss_phase {
                            PHASE_WINDUP => {
                                if self.boss_phase_timer == 0 {
                                    // Higher jump when enraged
                                    self.evil_santa_vy = if is_enraged { -10.0 } else { -8.0 };
                                    self.evil_santa_on_ground = false;
                                    
                                    // Track towards player position
                                    let target_x = self.player_x;
                                    let diff = target_x - self.evil_santa_x;
                                    let jump_vx = (diff / 40.0).clamp(-4.0, 4.0) * speed_mult;
                                    self.evil_santa_vx = jump_vx;
                                    self.evil_santa_facing_right = diff > 0.0;
                                }
                                self.boss_phase_timer += 1;
                                
                                // Wait until falling and near ground
                                if self.evil_santa_vy > 0.0 && self.evil_santa_y >= BOSS_EVIL_SANTA_Y - 8.0 {
                                    self.boss_phase = PHASE_ACTIVE;
                                    self.boss_phase_timer = 0;
                                    self.evil_santa_vx = 0.0;
                                    self.evil_santa_on_ground = true;
                                    self.evil_santa_vy = 0.0;
                                    self.evil_santa_y = BOSS_EVIL_SANTA_Y;
                                }
                            },
                            PHASE_ACTIVE => {
                                self.boss_phase_timer += 1;
                                
                                // Impact damage on first frame
                                if self.boss_phase_timer == 1 {
                                    // Wide shockwave radius
                                    let impact_radius = if is_enraged { 80.0 } else { 65.0 };
                                    let dx = (self.player_x - self.evil_santa_x).abs();
                                    let dy = (self.player_y - self.evil_santa_y).abs();
                                    if dx < impact_radius && dy < 50.0 && self.player_invuln_timer == 0 {
                                        let damage = 1; // Reduced damage
                                        self.player_hp = self.player_hp.saturating_sub(damage);
                                        let hurt_sfx = if self.frame % 2 == 0 { "santa_hurt_1" } else { "santa_hurt_2" };
                                        audio::play(hurt_sfx);
                                        self.player_invuln_timer = 60;
                                        // Knockup
                                        self.player_vy = -7.0;
                                        self.player_vx = if self.player_x > self.evil_santa_x { 3.0 } else { -3.0 };
                                    }
                                }
                                
                                if self.boss_phase_timer >= 18 {
                                    self.boss_phase = PHASE_RECOVERY;
                                    self.boss_phase_timer = 0;
                                }
                            },
                            PHASE_RECOVERY => {
                                self.boss_phase_timer += 1;
                                let recovery_time = ((30.0 * timer_mult) as u32).max(15);
                                if self.boss_phase_timer >= recovery_time {
                                    self.evil_santa_state = BOSS_STATE_RECOVER;
                                    self.boss_state_timer = 0;
                                }
                            },
                            _ => {}
                        }
                    },
                    ATTACK_PROJECTILE => {
                        match self.boss_phase {
                            PHASE_WINDUP => {
                                self.evil_santa_facing_right = self.player_x > self.evil_santa_x;
                                self.boss_phase_timer += 1;
                                
                                let windup_time = ((30.0 * timer_mult) as u32).max(15);
                                if self.boss_phase_timer >= windup_time {
                                    self.boss_phase = PHASE_ACTIVE;
                                    self.boss_phase_timer = 0;
                                }
                            },
                            PHASE_ACTIVE => {
                                self.boss_phase_timer += 1;
                                
                                // Spawn projectiles at specific frames
                                let spawn_frames = if is_enraged { vec![3, 12, 21] } else { vec![5, 15] };
                                
                                if spawn_frames.contains(&self.boss_phase_timer) {
                                    let base_vx = if self.evil_santa_facing_right { 4.5 * speed_mult } else { -4.5 * speed_mult };
                                    let start_x = if self.evil_santa_facing_right { 
                                        self.evil_santa_x + 25.0 
                                    } else { 
                                        self.evil_santa_x - 25.0 
                                    };
                                    
                                    // Spawn projectile
                                    for proj in self.projectiles.iter_mut() {
                                        if !proj.0 {
                                            // Vary Y position slightly for spread
                                            let y_offset = ((self.boss_phase_timer as f32 - 10.0) * 2.0);
                                            *proj = (true, start_x, self.evil_santa_y + y_offset, base_vx, true, 0.0);
                                            break;
                                        }
                                    }
                                }
                                
                                let attack_duration = if is_enraged { 28 } else { 22 };
                                if self.boss_phase_timer >= attack_duration {
                                    self.boss_phase = PHASE_RECOVERY;
                                    self.boss_phase_timer = 0;
                                }
                            },
                            PHASE_RECOVERY => {
                                self.boss_phase_timer += 1;
                                let recovery_time = ((25.0 * timer_mult) as u32).max(12);
                                if self.boss_phase_timer >= recovery_time {
                                    self.evil_santa_state = BOSS_STATE_RECOVER;
                                    self.boss_state_timer = 0;
                                }
                            },
                            _ => {}
                        }
                    },
                    _ => { self.evil_santa_state = BOSS_STATE_RECOVER; }
                }
            },
            BOSS_STATE_RECOVER => {
                self.boss_state_timer += 1;
                self.evil_santa_vx = 0.0;
                
                // Shorter recovery when enraged, chance to combo
                let recovery_time = ((35.0 * timer_mult) as u32).max(15);
                
                // Combo chance: in enraged mode, sometimes skip idle and attack again
                let combo_chance = if is_desperate { 0.4 } else if is_enraged { 0.25 } else { 0.0 };
                let rand_factor = (self.frame % 100) as f32 / 100.0;
                
                if self.boss_state_timer >= recovery_time {
                    if rand_factor < combo_chance {
                        // Combo attack! Go directly to SELECT
                        self.evil_santa_state = BOSS_STATE_SELECT;
                    } else {
                        self.evil_santa_state = BOSS_STATE_IDLE;
                    }
                    self.boss_state_timer = 0;
                }
            },
            _ => { self.evil_santa_state = BOSS_STATE_IDLE; }
        }

        // Apply Physics & Gravity (Always active)
        if self.evil_santa_on_ground == false {
             self.evil_santa_vy += GRAVITY;
             if self.evil_santa_vy > TERMINAL_VEL { self.evil_santa_vy = TERMINAL_VEL; }
        }

        self.evil_santa_x += self.evil_santa_vx;
        self.evil_santa_y += self.evil_santa_vy;

        // Ground constraint
        if self.evil_santa_y >= BOSS_EVIL_SANTA_Y {
             self.evil_santa_y = BOSS_EVIL_SANTA_Y;
             self.evil_santa_vy = 0.0;
             self.evil_santa_on_ground = true;
        }

        // Arena Clamp: 10-350 for boss fight (360x256 arena), 1950-2160 for old arena
        let arena_left = 10.0;
        let arena_right = 350.0;
        if self.evil_santa_x < arena_left { self.evil_santa_x = arena_left; }
        if self.evil_santa_x > arena_right { self.evil_santa_x = arena_right; }

        // Decrement flash timer
        if self.evil_santa_flash_timer > 0 {
            self.evil_santa_flash_timer -= 1;
        }

        // Advance animations
        self.evil_santa_anim_timer = self.evil_santa_anim_timer.wrapping_add(1);
        let step = match self.evil_santa_state {
            BOSS_STATE_IDLE => 12,
            BOSS_STATE_WALK => 8,
            BOSS_STATE_JUMP => 10,
            BOSS_STATE_ATTACK => 10,
            BOSS_STATE_FALL => 10,
            _ => 12,
        };
        if self.evil_santa_anim_timer >= step {
            self.evil_santa_anim_timer = 0;
            self.evil_santa_anim_frame = self.evil_santa_anim_frame.wrapping_add(1);
            // End attack after full cycle
            if self.evil_santa_state == BOSS_STATE_ATTACK && self.evil_santa_anim_frame >= EVIL_ATTACK_FRAMES {
                self.evil_santa_anim_frame = 0;
                // Animation cycle complete, FSM handles state transitions
            }
        }
    }

    fn exit_boss_fight(&mut self) {
        // Exit boss fight mode
        self.boss_active = false;
        self.boss_defeated = true;  // Mark boss as defeated to prevent re-trigger
        self.use_boss_santa = false;
        self.boss_death_timer = 0;

        // Clear any boss projectiles
        for proj in self.projectiles.iter_mut() {
            proj.0 = false;
        }

        // Move player to post-boss position
        self.player_x = 1970.0;
        self.player_y = 220.0;
        self.player_vx = 0.0;
        self.player_vy = 0.0;
        self.player_state = STATE_IDLE;
        self.player_on_ground = true;

        // Restore original level 3 platforms
        self.platforms.clear();
        self.platforms.push((1.0, 317.0, 222.0+5.0));        // ID 1
        self.platforms.push((301.0, 333.0, 207.0+5.0));      // ID 2
        self.platforms.push((333.0, 363.0, 193.0+5.0));      // ID 3
        self.platforms.push((361.0, 394.0, 177.0+5.0));      // ID 4
        self.platforms.push((391.0, 418.0, 163.0+5.0));      // ID 5
        self.platforms.push((211.0, 269.0, 163.0+5.0));      // ID 6
        self.platforms.push((329.0, 448.0, 119.0+5.0));      // ID 7
        self.platforms.push((450.0, 479.0, 164.0+5.0));      // ID 8
        self.platforms.push((540.0, 568.0, 164.0+5.0));      // ID 9
        self.platforms.push((630.0, 656.0, 164.0+5.0));      // ID 12
        self.platforms.push((599.0, 625.0, 192.0+5.0));      // ID 13
        self.platforms.push((569.0, 596.0, 223.0+5.0));      // ID 14
        self.platforms.push((630.0, 657.0, 223.0+5.0));      // ID 15
        self.platforms.push((689.0, 718.0, 133.0+5.0));      // ID 16
        self.platforms.push((756.0, 836.0, 133.0+5.0));      // ID 17
        self.platforms.push((690.0, 717.0, 223.0+5.0));      // ID 18
        self.platforms.push((749.0, 777.0, 223.0+5.0));      // ID 19
        self.platforms.push((808.0, 871.0, 223.0+5.0));      // ID 20
        self.platforms.push((868.0, 896.0, 133.0+5.0));      // ID 21
        self.platforms.push((927.0, 955.0, 119.0+5.0));      // ID 22
        self.platforms.push((957.0, 986.0, 162.0+5.0));      // ID 23
        self.platforms.push((986.0, 1021.0, 207.0+5.0));     // ID 24
        self.platforms.push((1017.0, 1045.0, 147.0+5.0));    // ID 25
        self.platforms.push((1046.0, 1075.0, 162.0+5.0));    // ID 26
        self.platforms.push((1107.0, 1133.0, 192.0+5.0));    // ID 28
        self.platforms.push((1134.0, 1165.0, 205.0+5.0));    // ID 29
        self.platforms.push((1167.0, 1254.0, 147.0+5.0));    // ID 30
        self.platforms.push((1165.0, 1253.0, 221.0+5.0));    // ID 31
        self.platforms.push((1285.0, 1313.0, 162.0+5.0));    // ID 32
        self.platforms.push((1287.0, 1317.0, 222.0+5.0));    // ID 33
        self.platforms.push((1320.0, 1373.0, 175.0+5.0));    // ID 34
        self.platforms.push((1346.0, 1374.0, 222.0+5.0));    // ID 35
        self.platforms.push((1374.0, 1404.0, 147.0+5.0));    // ID 37
        self.platforms.push((1406.0, 1456.0, 117.0+5.0));    // ID 38
        self.platforms.push((1406.0, 1490.0, 222.0+5.0));    // ID 39
        self.platforms.push((1496.0, 1524.0, 162.0+5.0));    // ID 40
        self.platforms.push((1554.0, 1580.0, 222.0+5.0));    // ID 42
        self.platforms.push((1643.0, 1671.0, 178.0+5.0));    // ID 43
        self.platforms.push((1673.0, 1701.0, 147.0+5.0));    // ID 44
        self.platforms.push((1615.0, 1736.0, 222.0+5.0));    // ID 45
        self.platforms.push((1734.0, 1764.0, 117.0+5.0));    // ID 46
        self.platforms.push((1762.0, 1791.0, 161.0+5.0));    // ID 47
        self.platforms.push((1823.0, 1853.0, 117.0+5.0));    // ID 48
        self.platforms.push((1853.0, 1881.0, 161.0+5.0));    // ID 49
        self.platforms.push((1881.0, 1913.0, 177.0+5.0));    // ID 50
        self.platforms.push((1909.0, 2159.0, 220.0+5.0));    // ID 51
        self.platforms.push((897.0, 952.0, 224.0+5.0));      // ID 53
        self.platforms.push((1.0, 2161.0, 50.0+5.0));        // ID 54 - Top ceiling

        // Keep the ladder at the end of level 3
        self.ladders[0] = (2136.0, 50.0, 189.0);

        // Add 2 gates at the top of the ladder (near x=2136, y=50)
        // Gate #1 - blocks passage at top of ladder
        self.walls[0] = (2120.0, 50.0, 15.0, 30.0);
        // Gate #2 - second barrier
        self.walls[1] = (2145.0, 50.0, 15.0, 30.0);

        // Clear other walls
        for i in 2..self.walls.len() {
            self.walls[i] = (0.0, 0.0, 0.0, 0.0);
        }

        // Set completion trigger after the gates
        // Player needs to climb ladder and pass through gates to complete
        self.completion_trigger = (2150.0, 50.0, 20.0, 30.0);

        // Restore level 3 enemies (original working positions - same as load_level for level 3)
        for enemy in self.enemies.iter_mut() {
            enemy.8 = false;
        }
        self.enemies[0] = (3, 370.0, 100.0, 0.0, 0.0, 0, 0, 341, true, 0, false, 0);
        self.enemies[1] = (2, 840.0, 208.0, 0.0, 0.0, 0, 0, 820, true, 0, false, 0);
        self.enemies[2] = (3, 780.0, 117.0, 0.0, 0.0, 0, 0, 768, true, 0, false, 0);
        self.enemies[3] = (1, 968.0, 36.0, 0.0, 0.0, 0, 0, 1025, true, 0, false, 0);
        self.enemies[4] = (3, 1200.0, 130.0, 0.0, 0.0, 0, 0, 1179, true, 0, false, 0);
        self.enemies[5] = (1, 1225.0, 36.0, 0.0, 0.0, 0, 0, 1332, true, 0, false, 0);
        self.enemies[6] = (2, 1450.0, 207.0, 0.0, 0.0, 0, 0, 1418, true, 0, false, 0);
        self.enemies[7] = (3, 1650.0, 207.0, 0.0, 0.0, 0, 0, 1627, true, 0, false, 0);
        self.enemies[8].8 = false;
        self.enemies[9] = (1, 820.0, 36.0, 0.0, 0.0, 0, 0, 701, true, 0, false, 0);

        log!("Boss defeated! Player moved to x={}, y={}", self.player_x, self.player_y);
    }

    fn render(&self) {
        
        // GAME OVER STATE - show gameoverpage.png sprite for 3 seconds
        if self.show_game_over {
            sprite!("gameoverpage", x = 0, y = 0);
            return;
        }
        
        // VICTORY STATE - show victory screen for 3 seconds
        // Triggered when Santa collects kid from door index 0 in level 3
        if self.show_victory {
            // Draw victory screen (use start_page as background for now)
            sprite!("start_page", x = 0, y = 0);
            // Overlay victory text
            rect!(x = 60, y = 70, w = 240, h = 100, color = 0x000000dd);
            text!("CONGRATULATIONS!", x = 90, y = 85, color = 0xffd700ff);
            text!("You rescued all the kids!", x = 70, y = 110, color = 0xffffffff);
            text!("Christmas is saved!", x = 85, y = 130, color = 0x00ff00ff);
            text!("Final Score: {}", self.score; x = 110, y = 155, color = 0xffd700ff);
            return;
        }
        
        // === PLAYING state rendering below ===
        
        // Draw the background sprite at exact camera position
        let bg_x = -(self.camera_x as i32);

        // Select background sprite based on current level
        match self.level {
            1 => sprite!("level1finalsprite", x = bg_x, y = 0),  // 1080x240px
            2 => sprite!("2ndlevel", x = bg_x, y = 0),  // 1440x240px
            3 => {
                if self.boss_active {
                    sprite!("bossfight/background", x = 0, y = 0)  // 360x240px - fixed screen, no scroll
                } else {
                    sprite!("bossfight bg1", x = bg_x, y = 0)  // 2160x240px - before boss trigger
                }
            },
            _ => sprite!("level1finalsprite", x = bg_x, y = 0),  // Default to level 1
        }

        // Draw game elements
        // self.draw_platforms();
        // self.draw_walls
        //self.draw_ladders();  // Draw ladders first (background layer)
        self.draw_walls();
        self.draw_doors();  // Draw doors first (background layer)
        self.draw_keys();   // Keys near exploded gates
        self.draw_kids();   // Kids to rescue (spawn from doors)
        self.draw_life();   // Life powerup (boss level door #3)
        self.draw_powerup1();  // PowerUp1 (boss level door #4)
        self.draw_gift_bombs();
        self.draw_enemies();
        self.draw_snowballs();  // Penguin snowball projectiles
        // Boss entity
        self.draw_boss();
        
        self.draw_projectiles();
        self.draw_player();
        self.draw_hud();
        if self.show_controls_panel { self.draw_controls_panel(); }
        
        // Draw level complete screen
        if self.level_complete {
            // Semi-transparent black overlay
            let alpha = ((120 - self.level_transition_timer) as f32 / 120.0 * 204.0) as u32;
            let overlay_color = 0x00000000 | (alpha << 24) | 0x000000;
            rect!(x = 0, y = 0, w = 360, h = 240, color = overlay_color);

            // "LEVEL COMPLETE!" text
            text!("LEVEL COMPLETE!", x = 100, y = 100, color = 0x00ff00ff);

            // Next level info
            let next_level = self.level + 1;
            text!("Entering Level {}...", next_level; x = 110, y = 130, color = 0xffffffff);
        }

        // Game over is now handled at the top of render() with show_game_over flag
        // The gameoverpage.png sprite is shown for 3 seconds then game restarts from level 1
    }

    fn draw_ladders(&self) {
        // DEBUG: Draw ladder collision zones
        for (i, (lx, ly_top, ly_bottom)) in self.ladders.iter().enumerate() {
            if *lx > 0.0 {
                let screen_x = (*lx - self.camera_x) as i32;
                let top_y = *ly_top as i32;
                let bottom_y = *ly_bottom as i32;
                let height = bottom_y - top_y;

                // Draw collision zone (semi-transparent purple) - 4px on each side
                let collision_width = 8; // 4px on each side
                rect!(x = screen_x - 4, y = top_y, w = collision_width, h = height as u32, color = 0xff00ff44);

                // Draw ladder rails (brown color)
                rect!(x = screen_x - 6, y = top_y, w = 2, h = height as u32, color = 0x8B4513ff);
                rect!(x = screen_x + 4, y = top_y, w = 2, h = height as u32, color = 0x8B4513ff);

                // Draw ladder rungs
                let rung_spacing = 12;
                let mut y = top_y + 6;
                while y < bottom_y - 4 {
                    rect!(x = screen_x - 4, y = y, w = 8, h = 2, color = 0xD2691Eff);
                    y += rung_spacing;
                }

                // Draw ladder label
                let label_x = screen_x + 10;
                let label_y = top_y + 5;
                text!("L{}", i + 1; x = label_x, y = label_y, color = 0xffffffff);
            }
        }
    }

    fn draw_player(&self) {
        let screen_x = (self.player_x - self.camera_x) as i32;
        // Offset Santa by -12px in Y only during bossfight
        let y_offset = if self.boss_active { -1 } else { 0 };
        let screen_y = (self.player_y as i32) + y_offset;

        // Get the correct sprite name based on state and animation frame
        let sprite_name = match self.player_state {
            STATE_IDLE => {
                let frame = (self.player_anim_frame % 4) + 1;
                if self.use_boss_santa {
                    match frame {
                        1 => "bossfight/santa/idle/1",
                        2 => "bossfight/santa/idle/2",
                        3 => "bossfight/santa/idle/3",
                        _ => "bossfight/santa/idle/4",
                    }
                } else {
                    match frame {
                        1 => "Santa/idle/1_32",
                        2 => "Santa/idle/2_32",
                        3 => "Santa/idle/3_32",
                        _ => "Santa/idle/4_32",
                    }
                }
            },
            STATE_WALK | STATE_RUN => {
                let frame = (self.player_anim_frame % 10) + 1;
                if self.use_boss_santa {
                    match frame {
                        1 => "bossfight/santa/walk/1",
                        2 => "bossfight/santa/walk/2",
                        3 => "bossfight/santa/walk/3",
                        4 => "bossfight/santa/walk/4",
                        5 => "bossfight/santa/walk/5",
                        6 => "bossfight/santa/walk/6",
                        7 => "bossfight/santa/walk/7",
                        8 => "bossfight/santa/walk/8",
                        9 => "bossfight/santa/walk/9",
                        _ => "bossfight/santa/walk/10",
                    }
                } else {
                    match frame {
                        1 => "Santa/runnin/1",
                        2 => "Santa/runnin/2",
                        3 => "Santa/runnin/3",
                        4 => "Santa/runnin/4",
                        5 => "Santa/runnin/5",
                        6 => "Santa/runnin/6",
                        7 => "Santa/runnin/7",
                        8 => "Santa/runnin/8",
                        9 => "Santa/runnin/9",
                        _ => "Santa/runnin/10",
                    }
                }
            },
            STATE_JUMP => {
                if self.use_boss_santa {
                    let frame = (self.jump_anim_frame % 5) + 1;
                    match frame {
                        1 => "bossfight/santa/jump/1",
                        2 => "bossfight/santa/jump/2",
                        3 => "bossfight/santa/jump/3",
                        4 => "bossfight/santa/jump/4",
                        _ => "bossfight/santa/jump/5",
                    }
                } else {
                    let frame = (self.jump_anim_frame % 7) + 1;
                    match frame {
                        1 => "Santa/jumping/1",
                        2 => "Santa/jumping/2",
                        3 => "Santa/jumping/3",
                        4 => "Santa/jumping/4",
                        5 => "Santa/jumping/5",
                        6 => "Santa/jumping/6",
                        _ => "Santa/jumping/7",
                    }
                }
            },
            STATE_FALL => {
                if self.use_boss_santa {
                    "bossfight/santa/santafall"
                } else {
                    "Santa/santafall"
                }
            },
            STATE_ATTACK => {
                let frame = ((self.attack_frame / 4) % 5) + 1;
                if self.use_boss_santa {
                    match frame {
                        1 => "bossfight/santa/attack/1",
                        2 => "bossfight/santa/attack/2",
                        3 => "bossfight/santa/attack/3",
                        4 => "bossfight/santa/attack/4",
                        _ => "bossfight/santa/attack/5",
                    }
                } else {
                    match frame {
                        1 => "Santa/attack/1",
                        2 => "Santa/attack/2",
                        3 => "Santa/attack/3",
                        4 => "Santa/attack/4",
                        _ => "Santa/attack/5",
                    }
                }
            },
            STATE_CLIMB => {
                let frame = (self.player_anim_frame % 10) + 1;
                // Boss variant has no climbing; fall back to normal
                match frame {
                    1 => "Santa/climbing/1",
                    2 => "Santa/climbing/2",
                    3 => "Santa/climbing/3",
                    4 => "Santa/climbing/4",
                    5 => "Santa/climbing/5",
                    6 => "Santa/climbing/6",
                    7 => "Santa/climbing/7",
                    8 => "Santa/climbing/8",
                    9 => "Santa/climbing/9",
                    _ => "Santa/climbing/10",
                }
            },
            STATE_CROUCH => {
                let frame = (self.player_anim_frame % 4) + 1;
                if self.use_boss_santa {
                    match frame {
                        1 => "bossfight/santa/down/1",
                        2 => "bossfight/santa/down/2",
                        3 => "bossfight/santa/down/3",
                        _ => "bossfight/santa/down/4",
                    }
                } else {
                    match frame {
                        1 => "Santa/down/1",
                        2 => "Santa/down/2",
                        3 => "Santa/down/3",
                        4 => "Santa/down/4",
                        5 => "Santa/down/5",
                        6 => "Santa/down/6",
                        7 => "Santa/down/7",
                        _ => "Santa/down/8",
                    }
                }
            },
            _ => {
                if self.use_boss_santa { "bossfight/santa/idle/1" } else { "Santa/idle/1_32" }
            },
        };

        if self.player_invuln_timer == 0 || (self.player_invuln_timer / 4) % 2 == 0 {
             sprite!(sprite_name, x = screen_x - 16, y = screen_y - 16, flip_x = !self.player_facing_right);

             // Firepower VFX when player has firepower powerup OR is boss Santa
             if (self.has_firepower || self.use_boss_santa) && self.player_state == STATE_ATTACK {
                // Map 20 attack frames to 8 VFX frames
                let vfx_frame = ((self.attack_frame as u32 * 8) / 20).min(7) + 1;
                let vfx_name = match vfx_frame {
                    1 => "firepower/1",
                    2 => "firepower/2",
                    3 => "firepower/3",
                    4 => "firepower/4",
                    5 => "firepower/5",
                    6 => "firepower/6",
                    7 => "firepower/7",
                    _ => "firepower/8",
                };
                
                // Draw VFX in front of Santa
                let vfx_x = if self.player_facing_right { screen_x + 8 } else { screen_x - 40 };
                sprite!(vfx_name, x = vfx_x, y = screen_y - 16, flip_x = !self.player_facing_right);
             }
        }
    }

    fn draw_boss(&self) {
        if !self.boss_active { return; }
        let screen_x = (self.evil_santa_x - self.camera_x) as i32;
        let screen_y = self.evil_santa_y as i32;

        // Skip drawing every other frame if flashing
        if self.evil_santa_flash_timer > 0 {
            if self.evil_santa_flash_timer % 2 != 0 {
                return;
            }
        }

        let sprite_name = match self.evil_santa_state {
            BOSS_STATE_DEAD => "bossfight/evilsanta/defeated",
            BOSS_STATE_IDLE => {
                let frame = (self.evil_santa_anim_frame % EVIL_IDLE_FRAMES) + 1;
                match frame {
                    1 => "bossfight/evilsanta/idle/1",
                    2 => "bossfight/evilsanta/idle/2",
                    3 => "bossfight/evilsanta/idle/3",
                    _ => "bossfight/evilsanta/idle/4",
                }
            },
            BOSS_STATE_SELECT => {
                // Use idle animation during selection
                let frame = (self.evil_santa_anim_frame % EVIL_IDLE_FRAMES) + 1;
                match frame {
                    1 => "bossfight/evilsanta/idle/1",
                    2 => "bossfight/evilsanta/idle/2",
                    3 => "bossfight/evilsanta/idle/3",
                    _ => "bossfight/evilsanta/idle/4",
                }
            },
            BOSS_STATE_WALK => {
                let frame = (self.evil_santa_anim_frame % EVIL_WALK_FRAMES) + 1;
                match frame {
                    1 => "bossfight/evilsanta/walk/1",
                    2 => "bossfight/evilsanta/walk/2",
                    3 => "bossfight/evilsanta/walk/3",
                    4 => "bossfight/evilsanta/walk/4",
                    5 => "bossfight/evilsanta/walk/5",
                    6 => "bossfight/evilsanta/walk/6",
                    7 => "bossfight/evilsanta/walk/7",
                    8 => "bossfight/evilsanta/walk/8",
                    9 => "bossfight/evilsanta/walk/9",
                    _ => "bossfight/evilsanta/walk/10",
                }
            },
            BOSS_STATE_JUMP => {
                let frame = (self.evil_santa_anim_frame % EVIL_JUMP_FRAMES) + 1;
                match frame {
                    1 => "bossfight/evilsanta/jump/1",
                    2 => "bossfight/evilsanta/jump/2",
                    3 => "bossfight/evilsanta/jump/3",
                    4 => "bossfight/evilsanta/jump/4",
                    5 => "bossfight/evilsanta/jump/5",
                    _ => "bossfight/evilsanta/jump/6",
                }
            },
            BOSS_STATE_ATTACK => {
                let frame = (self.evil_santa_anim_frame % EVIL_ATTACK_FRAMES) + 1;
                match frame {
                    1 => "bossfight/evilsanta/attack/1",
                    2 => "bossfight/evilsanta/attack/2",
                    3 => "bossfight/evilsanta/attack/3",
                    4 => "bossfight/evilsanta/attack/4",
                    5 => "bossfight/evilsanta/attack/5",
                    _ => "bossfight/evilsanta/attack/6",
                }
            },
            BOSS_STATE_FALL => "bossfight/evilsanta/evilsantafall",
            BOSS_STATE_RECOVER => "bossfight/evilsanta/idle/1", // Panting/Recover logic could go here
            _ => {
                // Default to idle
                "bossfight/evilsanta/idle/1"
            }
        };

        sprite!(sprite_name, x = screen_x - 16, y = screen_y - 16, flip_x = !self.evil_santa_facing_right);
    }


    fn draw_platforms(&self) {
        // DEBUG: Draw platform collision zones as semi-transparent rectangles
        for (i, (px1, px2, py)) in self.platforms.iter().enumerate() {
            // Check if platform is valid
            if *px2 > *px1 {
                let screen_x1 = (*px1 - self.camera_x) as i32;
                let screen_x2 = (*px2 - self.camera_x) as i32;
                let screen_y = *py as i32;
                let width = (screen_x2 - screen_x1) as u32;

                // Different colors for different floors for visibility
                let color = if i < 5 {
                    0xff000088  // Top floor - red with transparency
                } else if i < 11 {
                    0x00ff0088  // Middle floor - green with transparency
                } else {
                    0x0000ff88  // Bottom floor - blue with transparency
                };

                // Draw platform as a rectangle
                rect!(x = screen_x1, y = screen_y - 2, w = width, h = 4, color = color);

                // Draw tile number above platform
                let label_x = screen_x1 + 5;
                let label_y = screen_y - 10;
                text!("#{}", i + 1; x = label_x, y = label_y, color = 0xffffffff);
            }
        }
    }

    fn draw_walls(&self) {
        // DEBUG: Draw wall collision zones as semi-transparent rectangles
        for (i, (wx, wy, ww, wh)) in self.walls.iter().enumerate() {
            // Check if wall is valid
            if *ww > 0.0 && *wh > 0.0 {
                // Gate logic: different for each level
                if self.level == 3 && !self.boss_active {
                    // Level 3 (not in boss fight): Gate #1 (wall[0]) disappears with 1 key, Gate #2 (wall[1]) with 2 keys
                    if (i == 0 && self.keys_collected >= 1) || (i == 1 && self.keys_collected >= 2) {
                        continue;
                    }
                } else if self.level != 3 {
                    // Other levels: Gate (wall #3) disappears once 3 keys are collected
                    if i == 2 && self.keys_collected >= 3 {
                        continue;
                    }
                }
                // During boss fight in level 3, walls[0] and walls[1] are corner walls and should NOT disappear

                let screen_x = (*wx - self.camera_x) as i32;
                let screen_y = *wy as i32;
                let width = *ww as u32;
                let height = *wh as u32;

                if i == 2 {
                    // Wall #3 - draw the gate sprite instead of debug rectangle
                    sprite!("gatelevel", x = screen_x, y = screen_y);

                    // Draw "GATE OPEN!" message when gate disappears
                    if self.keys_collected >= 3 {
                        let msg_x = screen_x + 10;
                        let msg_y = screen_y - 15;
                        text!("GATE OPEN!", x = msg_x, y = msg_y, color = 0x00ff00ff);
                    }
                } else {
                    // // Yellow/orange color for walls with transparency
                    //let color = 0xffaa0088;

                    // // Draw wall as a filled rectangle
                    //rect!(x = screen_x, y = screen_y, w = width, h = height, color = color);

                    // // Draw wall number in center
                    //let label_x = screen_x + (width / 2) as i32 - 8;
                    //let label_y = screen_y + (height / 2) as i32 - 4;
                    //text!("W{}", i + 1; x = label_x, y = label_y, color = 0xffffffff);
                }
            }
        }

        // DEBUG: Draw completion trigger zone (green semi-transparent box)
        // Uncomment to visualize the trigger zone
        // if self.completion_trigger.2 > 0.0 && self.completion_trigger.3 > 0.0 {
        //     let trigger_screen_x = (self.completion_trigger.0 - self.camera_x) as i32;
        //     let trigger_y = self.completion_trigger.1 as i32;
        //     let trigger_w = self.completion_trigger.2 as u32;
        //     let trigger_h = self.completion_trigger.3 as u32;
        //     rect!(x = trigger_screen_x, y = trigger_y, w = trigger_w, h = trigger_h, color = 0x00ff0044);
        //     text!("EXIT", x = trigger_screen_x + 5, y = trigger_y + 25, color = 0x00ff00ff);
        // }
    }

    fn draw_enemy_ranges(&self) {
        // DEBUG: Draw enemy patrol ranges and attack zones
        for (i, enemy) in self.enemies.iter().enumerate() {
            // Skip if enemy type is 0 (not initialized)
            if enemy.0 == 0 || !enemy.8 {
                continue;
            }

            let enemy_type = enemy.0;
            let patrol_start = enemy.7 as f32;
            let patrol_range = 100.0;  // From patrol AI logic

            // Draw patrol range (horizontal line)
            let range_start = (patrol_start - self.camera_x) as i32;
            let range_end = ((patrol_start + patrol_range) - self.camera_x) as i32;
            let enemy_y = enemy.2 as i32;

            // Patrol range - cyan semi-transparent
            let patrol_color = 0x00ffffff44;
            rect!(x = range_start, y = enemy_y - 12, w = (range_end - range_start) as u32, h = 2, color = patrol_color);

            // Draw patrol endpoints (small markers)
            rect!(x = range_start, y = enemy_y - 16, w = 2, h = 8, color = 0x00ffffff88);
            rect!(x = range_end, y = enemy_y - 16, w = 2, h = 8, color = 0x00ffffff88);

            // For kickmouse (type 2), draw attack range
            if enemy_type == 2 {
                let enemy_screen_x = (enemy.1 - self.camera_x) as i32;

                // Attack range: 60px horizontally, 20px vertically (changed from 10px to 60px based on code)
                let attack_range_h = 60;  // Actually 10 in the code, showing both
                let attack_range_v = 20;

                // Draw actual attack trigger zone (small 10px range) - red
                rect!(
                    x = enemy_screen_x - 10,
                    y = enemy_y - attack_range_v,
                    w = 20,
                    h = (attack_range_v * 2) as u32,
                    color = 0xff000044
                );

                // Draw larger detection zone (60px) - orange
                rect!(
                    x = enemy_screen_x - attack_range_h,
                    y = enemy_y - attack_range_v,
                    w = (attack_range_h * 2) as u32,
                    h = (attack_range_v * 2) as u32,
                    color = 0xff880022
                );

                // Label
                text!("K", x = enemy_screen_x - 3, y = enemy_y - 25, color = 0xff0000ff);
            } else {
                // Label for regular enemy
                let enemy_screen_x = (enemy.1 - self.camera_x) as i32;
                text!("E", x = enemy_screen_x - 3, y = enemy_y - 25, color = 0x00ffffff);
            }

            // Draw enemy number
            let enemy_screen_x = (enemy.1 - self.camera_x) as i32;
            text!("{}", i + 1; x = enemy_screen_x + 10, y = enemy_y - 25, color = 0xffffffff);
        }
    }

    fn draw_enemies(&self) {
        for enemy in self.enemies.iter() {
            // Skip if enemy type is 0 (not initialized)
            if enemy.0 == 0 {
                continue;
            }
            
            let screen_x = (enemy.1 - self.camera_x) as i32;
            let screen_y = enemy.2 as i32;
            let respawn_timer = enemy.11;
            
            // If enemy is respawning and in cloud animation phase (last 120 frames)
            if respawn_timer > 0 && respawn_timer <= 120 {
                // Calculate cloud animation frame on-the-fly (8 frames total)
                // Change frame every 15 ticks (120 / 8 = 15)
                let frames_into_cloud = 120 - respawn_timer;
                let cloud_frame = ((frames_into_cloud / 15) % 8) as u8;
                
                // Draw cloud animation during respawn
                let cloud_sprite = match cloud_frame {
                    0 => "cloud/cloud1",
                    1 => "cloud/cloud2",
                    2 => "cloud/cloud3",
                    3 => "cloud/cloud4",
                    4 => "cloud/cloud5",
                    5 => "cloud/cloud6",
                    6 => "cloud/cloud7",
                    _ => "cloud/cloud8",
                };
                sprite!(cloud_sprite, x = screen_x - 16, y = screen_y - 16);
                continue;
            }
            
            // If enemy is dead but not yet in cloud phase, don't draw anything
            if respawn_timer > 120 {
                continue;
            }
            
            // Draw normal enemy if active and alive
            if enemy.8 {
                let enemy_type = enemy.0;
                let anim_frame = enemy.6;
                let direction = enemy.5;
                let is_attacking = enemy.10;

                // Determine sprite name based on enemy type and animation frame
                let sprite_name = match enemy_type {
                    1 => {
                        // Enemy1: 8 walk frames (mouse1.png - mouse8.png)
                        let frame = (anim_frame % 8) + 1;
                        match frame {
                            1 => "enemy/enemy1/mouse1",
                            2 => "enemy/enemy1/mouse2",
                            3 => "enemy/enemy1/mouse3",
                            4 => "enemy/enemy1/mouse4",
                            5 => "enemy/enemy1/mouse5",
                            6 => "enemy/enemy1/mouse6",
                            7 => "enemy/enemy1/mouse7",
                            8 => "enemy/enemy1/mouse8",
                            _ => "enemy/enemy1/mouse1",
                        }
                    },
                    2 => {
                        // Kickmouse: 8 walk frames (1.png - 8.png) or 3 attack frames
                        if is_attacking {
                            // Attack animation: attack/1.png - attack/3.png
                            let frame = anim_frame + 1;  // 0-2 becomes 1-3
                            match frame {
                                1 => "enemy/kickmouse/attack/1",
                                2 => "enemy/kickmouse/attack/2",
                                3 => "enemy/kickmouse/attack/3",
                                _ => "enemy/kickmouse/attack/1",
                            }
                        } else {
                            // Walk animation
                            let frame = (anim_frame % 8) + 1;
                            match frame {
                                1 => "enemy/kickmouse/1",
                                2 => "enemy/kickmouse/2",
                                3 => "enemy/kickmouse/3",
                                4 => "enemy/kickmouse/4",
                                5 => "enemy/kickmouse/5",
                                6 => "enemy/kickmouse/6",
                                7 => "enemy/kickmouse/7",
                                8 => "enemy/kickmouse/8",
                                _ => "enemy/kickmouse/1",
                            }
                        }
                    },
                    3 => {
                        // Penguin: 8 walk frames (penguin1.png - penguin8.png) or 3 attack frames
                        if is_attacking {
                            // Attack animation: attack/penguinattack1.png - penguinattack3.png
                            let frame = anim_frame + 1;  // 0-2 becomes 1-3
                            match frame {
                                1 => "enemy/penguin/attack/penguinattack1",
                                2 => "enemy/penguin/attack/penguinattack2",
                                3 => "enemy/penguin/attack/penguinattack3",
                                _ => "enemy/penguin/attack/penguinattack1",
                            }
                        } else {
                            // Walk animation
                            let frame = (anim_frame % 8) + 1;
                            match frame {
                                1 => "enemy/penguin/penguin1",
                                2 => "enemy/penguin/penguin2",
                                3 => "enemy/penguin/penguin3",
                                4 => "enemy/penguin/penguin4",
                                5 => "enemy/penguin/penguin5",
                                6 => "enemy/penguin/penguin6",
                                7 => "enemy/penguin/penguin7",
                                8 => "enemy/penguin/penguin8",
                                _ => "enemy/penguin/penguin1",
                            }
                        }
                    },
                    _ => "enemy/enemy1/mouse1",
                };

                // Draw enemy sprite with flipping based on direction
                // Direction: 0 = moving left, 1 = moving right
                // Sprites naturally face right, so flip when moving left
                sprite!(sprite_name, x = screen_x - 16, y = screen_y - 16, flip_x = direction == 0);
            }
        }
    }

    fn draw_snowballs(&self) {
        // Draw penguin snowball projectiles
        for snowball in self.snowballs.iter() {
            if snowball.0 {
                let screen_x = (snowball.1 - self.camera_x) as i32;
                let screen_y = snowball.2 as i32;
                // Draw snowball sprite (sprite is centered, adjust for 16x16 sprite)
                sprite!("enemy/penguin/snowball", x = screen_x - 8, y = screen_y - 8);
            }
        }
    }

    fn draw_projectiles(&self) {
        for proj in self.projectiles.iter() {
            if proj.0 {
                let screen_x = (proj.1 - self.camera_x) as i32;
                let screen_y = proj.2 as i32;
                let color = if proj.4 { 0xffffffff } else { 0xff0000ff };
                circ!(x = screen_x, y = screen_y, d = 8, color = color);
            }
        }
    }

    fn update_keys_animation(&mut self) {
        for i in 0..self.keys.len() {
            if self.keys[i].2 {
                // Advance animation timer; cycle frames 1-6 at 8 ticks per frame
                self.keys[i].4 += 1;
                if self.keys[i].4 >= 8 {
                    self.keys[i].4 = 0;
                    self.keys[i].3 = (self.keys[i].3 + 1) % 6;
                }
            }
        }

        if self.key_pickup_flash > 0 {
            self.key_pickup_flash -= 1;
        }
    }

    fn draw_keys(&self) {
        for key in self.keys.iter() {
            if key.2 {
                let screen_x = (key.0 - self.camera_x) as i32;
                let screen_y = key.1 as i32;
                let frame = (key.3 % 6) + 1;
                let sprite_name = match frame {
                    1 => "key/1",
                    2 => "key/2",
                    3 => "key/3",
                    4 => "key/4",
                    5 => "key/5",
                    _ => "key/6",
                };
                sprite!(sprite_name, x = screen_x-10, y = screen_y+3);
            }
        }
    }

    fn draw_kids(&self) {
        for kid in self.kids.iter() {
            // Format: (x, y, active, collected, anim_frame, anim_timer)
            if kid.2 && !kid.3 {  // active and not collected
                let screen_x = (kid.0 - self.camera_x) as i32;
                let screen_y = kid.1 as i32;

                // Kid has 3-frame animation (sprites: 1.png, 2.png, 3.png)
                let frame = (kid.4 % 3) + 1;
                let sprite_name = match frame {
                    1 => "kid/1",
                    2 => "kid/2",
                    _ => "kid/3",
                };

                // Draw kid sprite (32x32) centered
                sprite!(sprite_name, x = screen_x - 8, y = screen_y );
            }
        }
    }

    fn draw_life(&self) {
        if self.life_active && !self.life_collected {
            let screen_x = (self.life_position.0 - self.camera_x) as i32;
            let screen_y = self.life_position.1 as i32;
            
            // Draw life powerup sprite (32x32)
            sprite!("life", x = screen_x -8 , y = screen_y);
        }
    }

    fn draw_powerup1(&self) {
        if self.powerup1_active && !self.powerup1_collected {
            let screen_x = (self.powerup1_position.0 - self.camera_x) as i32;
            let screen_y = self.powerup1_position.1 as i32;
            
            // Draw powerup1 sprite
            sprite!("powerUp1", x = screen_x - 8, y = screen_y );
        }
    }

    fn check_powerup1_collection(&mut self) {
        if self.powerup1_active && !self.powerup1_collected {
            let dx = (self.player_x - self.powerup1_position.0).abs();
            let dy = (self.player_y - self.powerup1_position.1).abs();
            
            if dx < 20.0 && dy < 20.0 {
                self.powerup1_collected = true;
                self.powerup1_active = false;
                self.has_firepower = true;  // Grant firepower ability
                self.score += 750;
                log!("PowerUp1 collected! Firepower activated!");
            }
        }
    }

    fn check_life_collection(&mut self) {
        if self.life_active && !self.life_collected {
            let dx = (self.player_x - self.life_position.0).abs();
            let dy = (self.player_y - self.life_position.1).abs();

            if dx < 20.0 && dy < 20.0 {
                self.life_collected = true;
                self.life_active = false;

                // Increase lives by 1 (from door 3 in level 3)
                self.lives += 1;

                self.score += 1000;
                log!("Life powerup collected! Lives: {}", self.lives);
            }
        }
    }

    fn check_keys(&mut self) {
        for key in self.keys.iter_mut() {
            if key.2 {
                let dx = (self.player_x - key.0).abs();
                let dy = (self.player_y - key.1).abs();
                if dx < 16.0 && dy < 16.0 {
                    key.2 = false;
                    self.keys_collected = self.keys_collected.saturating_add(1);
                    self.score += 100;
                    self.key_pickup_flash = 30;
                    audio::play("collection");
                    // SFX hook (stub) – integrate Turbo audio when available
                    log!("SFX: key_pickup");
                }
            }
        }
    }

    fn update_kids_animation(&mut self) {
        // Update flash timer
        if self.kid_pickup_flash > 0 {
            self.kid_pickup_flash -= 1;
        }

        // Update each kid's animation
        for kid in self.kids.iter_mut() {
            // Format: (x, y, active, collected, anim_frame, anim_timer, spawned_from_door_idx)
            let (_, _, active, collected, anim_frame, anim_timer, _) = kid;

            if *active && !*collected {
                // Increment animation timer
                *anim_timer += 1;

                // 3-frame idle animation (frames 0, 1, 2) at 12 frames per sprite
                if *anim_timer >= 12 {
                    *anim_timer = 0;
                    *anim_frame = (*anim_frame + 1) % 3;
                }
            }
        }
    }

    fn check_kid_collection(&mut self) {
        for (kid_idx, kid) in self.kids.iter_mut().enumerate() {
            // Format: (x, y, active, collected, anim_frame, anim_timer, spawned_from_door_idx)
            if kid.2 && !kid.3 {
                let dx = (self.player_x - kid.0).abs();
                let dy = (self.player_y - kid.1).abs();

                // 20px pickup radius (slightly larger than keys)
                if dx < 20.0 && dy < 20.0 {
                    let door_idx = kid.6;
                    kid.3 = true; // Mark as collected
                    kid.2 = false; // Mark as inactive
                    self.kids_collected += 1;
                    self.score += 500;
                    self.kid_pickup_flash = 30;

                    // Bonus: restore 1 HP when rescuing a kid
                    if self.player_hp < self.player_max_hp {
                        self.player_hp += 1;
                    }

                    log!("SFX: kid_rescued - Kids rescued: {}/{}", self.kids_collected, self.total_kids_in_level);
                    let kid_sfx = if self.frame % 2 == 0 { "meeting_kid" } else { "meeting_kid_2" };
                    audio::play(kid_sfx);
                    
                    // Check for game victory: Level 3, door 0 kid
                    if self.level == 3 && door_idx == 0 {
                        self.show_victory = true;
                        self.game_won_timer = 0;
                        log!("Game Complete! Victory!");
                    }
                }
            }
        }
    }

    fn check_level_completion(&mut self) {
        // Check if player has collected all required keys
        let required_keys = if self.level == 3 { 2 } else { 3 };
        if self.keys_collected < required_keys {
            return;
        }
        
        // Check if player has rescued all kids in the level
        if self.kids_collected < self.total_kids_in_level {
            return;
        }

        // Don't check if already completing
        if self.level_complete {
            return;
        }

        // Check if player is in the completion trigger zone
        let trigger_x = self.completion_trigger.0;
        let trigger_y = self.completion_trigger.1;
        let trigger_w = self.completion_trigger.2;
        let trigger_h = self.completion_trigger.3;

        // Skip if no trigger zone is set
        if trigger_w == 0.0 || trigger_h == 0.0 {
            return;
        }

        // Player hitbox
        let player_left = self.player_x - 7.0;
        let player_right = self.player_x + 7.0;
        let player_top = self.player_y - 28.0 +9.0;
        let player_bottom = self.player_y + 28.0 - 9.0;

        // Trigger zone bounds
        let trigger_left = trigger_x;
        let trigger_right = trigger_x + trigger_w;
        let trigger_top = trigger_y;
        let trigger_bottom = trigger_y + trigger_h;

        // AABB collision check
        if player_right > trigger_left &&
           player_left < trigger_right &&
           player_bottom > trigger_top &&
           player_top < trigger_bottom
        {
            // Player entered the completion zone!
            self.level_complete = true;
            self.level_transition_timer = 120; // 2 seconds transition
            audio::play("completion");
            log!("Level Complete! Transitioning to next level...");
        }
    }

    fn draw_gift_bombs(&self) {
        // Draw gift bomb items (to pick up)
        for item in self.gift_bomb_items.iter() {
            if item.2 {  // If active
                let screen_x = (item.0 - self.camera_x) as i32;
                let screen_y = item.1 as i32;
                // Gift bomb sprite is 64x64 pixels
                // Santa sprite is 32x32, rendered at (screen_x - 16, screen_y - 16)
                // To align gift bomb with Santa's level:
                // - Santa's feet are at screen_y + 16 (bottom of 32px sprite centered at screen_y)
                // - Gift bomb should have its bottom at the same level
                // - For 64px sprite: bottom = screen_y + 32, so top = screen_y - 32
                // - But gift visual is at bottom of sprite, so we need to adjust
                // Offset: center horizontally (-32), but align bottom with Santa's feet
                sprite!("gift bomb/idle", x = screen_x - 32, y = screen_y - 48);
            }
        }

        // Draw placed bombs (idle or exploding)
        for bomb in self.placed_bombs.iter() {
            if bomb.2 {  // If active
                let screen_x = (bomb.0 - self.camera_x) as i32;
                let screen_y = bomb.1 as i32;

                // Map animation frames to sprites:
                // Frame 0: idle
                // Frames 1-4: sprites 2-5 (first warning cycle)
                // Frames 5-8: sprites 2-5 (second warning cycle)
                // Frames 9-11: sprites 6-8 (explosion)
                let sprite_name = match bomb.3 {
                    0 => "gift bomb/idle",
                    1 | 5 => "gift bomb/2",  // First & second cycle
                    2 | 6 => "gift bomb/3",
                    3 | 7 => "gift bomb/4",
                    4 | 8 => "gift bomb/5",
                    9 => "gift bomb/6",       // Explosion
                    10 => "gift bomb/7",
                    11 => "gift bomb/8",
                    _ => "gift bomb/idle",
                };

                // 64x64 sprite - align bottom with Santa's feet level
                sprite!(sprite_name, x = screen_x - 32, y = screen_y - 48);
            }
        }
    }

    fn draw_doors(&self) {
        for door in self.doors.iter() {
            let screen_x = (door.0 - self.camera_x) as i32;
            let mut screen_y = door.1 as i32;
            
            if self.level == 2 {
            screen_y -= 18;
            }
            // Select door sprites based on current level
            let (intact_sprite, exploded_sprite) = match self.level {
                1 => ("door", "exploded_gate"),
                2 => ("door2ndlevel", "door_explodedsecondlevel"),
                _ => ("door", "exploded_gate"),
            };

            if door.2 {
                // Door is intact - draw door sprite
                // x, y are top-left corner
                sprite!(intact_sprite, x = screen_x, y = screen_y );
            } else {
                // Door is destroyed - draw exploded door sprite
                sprite!(exploded_sprite, x = screen_x, y = screen_y);
            }
        }
    }

    fn draw_hud(&self) {
        // Black HUD bar at top
        rect!(x = 0, y = 0, w = 360, h = 16, color = 0x000000ff);

        // ============================================
        // HEALTH BAR (Rectangle style)
        // ============================================
        // Background (dark red)
        rect!(x = 10, y = 4, w = 50, h = 8, color = 0x400000ff);
        
        // Foreground (bright red, proportional to HP)
        let health_width = ((self.player_hp as f32 / self.player_max_hp as f32) * 50.0) as u32;
        let health_color = if self.player_hp <= 3 {
            0xff0000ff  // Red when low
        } else if self.player_hp <= 5 {
            0xffaa00ff  // Orange when medium
        } else {
            0x00ff00ff  // Green when high
        };
        rect!(x = 10, y = 4, w = health_width, h = 8, color = health_color);
        
        // HP text
        text!("HP", x = 11, y = 4, color = 0xffffffff);

        // ============================================
        // LIVES (Heart icons)
        // ============================================
        for i in 0..self.lives {
            let x = 70 + (i as i32 * 10);
            // Draw heart shape with rectangles
            rect!(x = x, y = 5, w = 3, h = 2, color = 0xff0000ff);      // Left top
            rect!(x = x + 4, y = 5, w = 3, h = 2, color = 0xff0000ff);  // Right top
            rect!(x = x, y = 7, w = 7, h = 4, color = 0xff0000ff);      // Body
        }

        // Rest of HUD - adjusted positions to prevent overlap
        text!("SCORE:{}", self.score; x = 105, y = 4, color = 0xffffffff);
        text!("KEYS:{}", self.keys_collected; x = 170, y = 4, color = 0xffd700ff);
        text!("KIDS:{}/{}", self.kids_collected, self.total_kids_in_level; x = 215, y = 4, color = 0x00ffffff);
        text!("BOM:{}", self.gift_bombs; x = 270, y = 4, color = 0xffaa00ff);

        // Level timer (countdown) - positioned at far right
        let remaining_frames = self.level_time_limit.saturating_sub(self.level_timer);
        let remaining_seconds = remaining_frames / 60;
        let minutes = remaining_seconds / 60;
        let secs = remaining_seconds % 60;
        let timer_color = if remaining_seconds < 30 { 0xff0000ff } else { 0xffffffff };
        text!("{}:{:02}", minutes, secs; x = 320, y = 4, color = timer_color);

        // Visual feedback near player when a key is picked up
        if self.key_pickup_flash > 0 {
            let fx_x = (self.player_x - self.camera_x) as i32;
            let fx_y = self.player_y as i32 - 15;
            text!("KEY!", x = fx_x - 10, y = fx_y, color = 0xffd700ff, font = "small");
        }

        // Visual feedback near player when a kid is rescued
        if self.kid_pickup_flash > 0 {
            let fx_x = (self.player_x - self.camera_x) as i32;
            let fx_y = self.player_y as i32 - 25;
            text!("KID SAVED! +500", x = fx_x - 30, y = fx_y, color = 0x00ffffff, font = "small");
        }

        // Draw Boss Health Bar if active
        if self.boss_active && self.evil_santa_hp > 0 {
            let bar_width = 150;
            let bar_height = 8;
            let bar_x = (SCREEN_WIDTH as i32 - bar_width) / 2;
            let bar_y = 20;
            
            // Border
            rect!(x = bar_x - 2, y = bar_y - 2, w = bar_width + 4, h = bar_height + 4, color = 0x000000ff);
            // Background
            rect!(x = bar_x, y = bar_y, w = bar_width, h = bar_height, color = 0x440000ff);
            // Fill
            let fill_pct = self.evil_santa_hp as f32 / self.evil_santa_max_hp as f32;
            let fill_width = (bar_width as f32 * fill_pct) as u32;
            if fill_width > 0 {
                rect!(x = bar_x, y = bar_y, w = fill_width, h = bar_height, color = 0xff0000ff);
            }
            text!("EVIL SANTA", x = bar_x + 35, y = bar_y - 12, color = 0xff0000ff, font = "small");
        }

        // Developer mode hint overlay
        if self.dev_mode {
            // Translucent bar under HUD for dev controls
            rect!(x = 0, y = 16, w = 360, h = 12, color = 0x00000088);
            // Controls: G toggle, J=Level1, K=Level2, B=Level3, L=Next, H=Prev
            text!(
                "DEV ON: G toggle | J:1 K:2 B:3 | L:Next H:Prev",
                x = 8,
                y = 18,
                color = 0xffff00ff,
                font = "small"
            );
        }
    }
}

impl GameState {
    fn draw_controls_panel(&self) {
        // Panel dimensions
        let panel_x = 15;
        let panel_y = 25;
        let panel_w = 330;
        let panel_h = 190;
        
        // ============================================
        // BACKGROUND - Festive dark overlay with gradient effect
        // ============================================
        // Outer shadow/glow
        rect!(x = panel_x - 3, y = panel_y - 3, w = panel_w + 6, h = panel_h + 6, color = 0x00000066);
        
        // Main panel background - dark green Christmas feel
        rect!(x = panel_x, y = panel_y, w = panel_w, h = panel_h, color = 0x0a1a12ee);
        rect!(x = panel_x + 2, y = panel_y + 2, w = panel_w - 4, h = panel_h - 4, color = 0x0d2818dd);
        
        // ============================================
        // DECORATIVE BORDERS - Gold frame
        // ============================================
        rect!(x = panel_x, y = panel_y, w = panel_w, h = 3, color = 0xffd700ff);
        rect!(x = panel_x, y = panel_y + panel_h - 3, w = panel_w, h = 3, color = 0xffd700ff);
        rect!(x = panel_x, y = panel_y, w = 3, h = panel_h, color = 0xffd700ff);
        rect!(x = panel_x + panel_w - 3, y = panel_y, w = 3, h = panel_h, color = 0xffd700ff);
        
        // Corner decorations (Christmas ornament style)
        circ!(x = panel_x, y = panel_y, d = 10, color = 0xff3333ff);
        circ!(x = panel_x + panel_w, y = panel_y, d = 10, color = 0x00cc00ff);
        circ!(x = panel_x, y = panel_y + panel_h, d = 10, color = 0x00cc00ff);
        circ!(x = panel_x + panel_w, y = panel_y + panel_h, d = 10, color = 0xff3333ff);
        
        // Inner gold dots on corners
        circ!(x = panel_x, y = panel_y, d = 4, color = 0xffd700ff);
        circ!(x = panel_x + panel_w, y = panel_y, d = 4, color = 0xffd700ff);
        circ!(x = panel_x, y = panel_y + panel_h, d = 4, color = 0xffd700ff);
        circ!(x = panel_x + panel_w, y = panel_y + panel_h, d = 4, color = 0xffd700ff);
        
        // ============================================
        // TITLE SECTION
        // ============================================
        rect!(x = panel_x + 60, y = panel_y + 8, w = 210, h = 22, color = 0x8b0000cc);
        rect!(x = panel_x + 62, y = panel_y + 10, w = 206, h = 18, color = 0xcc2222dd);
        
        // Title text with shadow
        text!("GAME CONTROLS", x = panel_x + 109, y = panel_y + 13, color = 0x00000088);
        text!("GAME CONTROLS", x = panel_x + 107, y = panel_y + 12, color = 0xffffffff);
        
        text!("Press S to close", x = panel_x + 125, y = panel_y + 33, color = 0xffd70099, font = "small");
        rect!(x = panel_x + 20, y = panel_y + 45, w = panel_w - 40, h = 1, color = 0xffd70066);
        
        // ============================================
        // CONTROLS GRID - Two columns with styled cards
        // ============================================
        let col1_x = panel_x + 15;
        let col2_x = panel_x + 170;
        let row_h = 22;
        let start_y = panel_y + 55;
        
        // Column 1: MOVE
        rect!(x = col1_x, y = start_y, w = 145, h = row_h - 2, color = 0x1a472aaa);
        rect!(x = col1_x, y = start_y, w = 3, h = row_h - 2, color = 0x00ff00aa);
        text!("MOVE", x = col1_x + 8, y = start_y + 5, color = 0x00ff00ff, font = "small");
        text!("< > Arrows", x = col1_x + 70, y = start_y + 5, color = 0xffffffff, font = "small");
        
        // Column 1: JUMP
        rect!(x = col1_x, y = start_y + row_h, w = 145, h = row_h - 2, color = 0x1a472aaa);
        rect!(x = col1_x, y = start_y + row_h, w = 3, h = row_h - 2, color = 0x00ff00aa);
        text!("JUMP", x = col1_x + 8, y = start_y + row_h + 5, color = 0x00ff00ff, font = "small");
        text!("X Key", x = col1_x + 90, y = start_y + row_h + 5, color = 0xffffffff, font = "small");
        
        // Column 1: ATTACK
        rect!(x = col1_x, y = start_y + row_h * 2, w = 145, h = row_h - 2, color = 0x2a1a1aaa);
        rect!(x = col1_x, y = start_y + row_h * 2, w = 3, h = row_h - 2, color = 0xff6b6baa);
        text!("ATTACK", x = col1_x + 8, y = start_y + row_h * 2 + 5, color = 0xff6b6bff, font = "small");
        text!("Z Key", x = col1_x + 90, y = start_y + row_h * 2 + 5, color = 0xffffffff, font = "small");
        
        // Column 1: BOMB
        rect!(x = col1_x, y = start_y + row_h * 3, w = 145, h = row_h - 2, color = 0x2a1a0aaa);
        rect!(x = col1_x, y = start_y + row_h * 3, w = 3, h = row_h - 2, color = 0xffaa00aa);
        text!("BOMB", x = col1_x + 8, y = start_y + row_h * 3 + 5, color = 0xffaa00ff, font = "small");
        text!("C Key", x = col1_x + 90, y = start_y + row_h * 3 + 5, color = 0xffffffff, font = "small");
        
        // Column 2: CROUCH
        rect!(x = col2_x, y = start_y, w = 145, h = row_h - 2, color = 0x1a472aaa);
        rect!(x = col2_x, y = start_y, w = 3, h = row_h - 2, color = 0x00ff00aa);
        text!("CROUCH", x = col2_x + 8, y = start_y + 5, color = 0x00ff00ff, font = "small");
        text!("Down", x = col2_x + 100, y = start_y + 5, color = 0xffffffff, font = "small");
        
        // Column 2: CLIMB
        rect!(x = col2_x, y = start_y + row_h, w = 145, h = row_h - 2, color = 0x1a472aaa);
        rect!(x = col2_x, y = start_y + row_h, w = 3, h = row_h - 2, color = 0x00ff00aa);
        text!("CLIMB", x = col2_x + 8, y = start_y + row_h + 5, color = 0x00ff00ff, font = "small");
        text!("Up/Down", x = col2_x + 85, y = start_y + row_h + 5, color = 0xffffffff, font = "small");
        
        // Column 2: HELP
        rect!(x = col2_x, y = start_y + row_h * 2, w = 145, h = row_h - 2, color = 0x1a1a2aaa);
        rect!(x = col2_x, y = start_y + row_h * 2, w = 3, h = row_h - 2, color = 0x00aaffaa);
        text!("HELP", x = col2_x + 8, y = start_y + row_h * 2 + 5, color = 0x00aaffff, font = "small");
        text!("S Key", x = col2_x + 100, y = start_y + row_h * 2 + 5, color = 0xffffffff, font = "small");
        
        // Column 2: RESTART
        rect!(x = col2_x, y = start_y + row_h * 3, w = 145, h = row_h - 2, color = 0x1a1a2aaa);
        rect!(x = col2_x, y = start_y + row_h * 3, w = 3, h = row_h - 2, color = 0x8888ffaa);
        text!("RESTART", x = col2_x + 8, y = start_y + row_h * 3 + 5, color = 0x8888ffff, font = "small");
        text!("Enter", x = col2_x + 95, y = start_y + row_h * 3 + 5, color = 0xffffffff, font = "small");
        
        // ============================================
        // FOOTER - Animated tips
        // ============================================
        rect!(x = panel_x + 20, y = panel_y + 155, w = panel_w - 40, h = 1, color = 0xffd70044);
        
        let tip_phase = (self.frame / 120) % 3;
        let tip_text = match tip_phase {
            0 => "Rescue kids and collect keys to progress!",
            1 => "Defeat enemies to get gift bombs!",
            _ => "Destroy doors to free the children!",
        };
        text!(tip_text, x = panel_x + 40, y = panel_y + 165, color = 0xff6b6bff, font = "small");
        
        // Snowflake decorations
        let snow_offset = (self.frame / 8) % 20;
        circ!(x = panel_x + 25, y = panel_y + 170 + (snow_offset as i32 % 10), d = 3, color = 0xffffff66);
        circ!(x = panel_x + panel_w - 25, y = panel_y + 175 - (snow_offset as i32 % 10), d = 3, color = 0xffffff66);
    }
}
        