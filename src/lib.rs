use turbo::*;

#[turbo::game]
struct GameState {
    // Add fields here
}
impl GameState {
    pub fn new() -> Self {
        // initialize your game state
        Self { }
    }
    pub fn update(&mut self) {
        // This is where your main game loop code goes
        // The stuff in this block will run ~60x per sec
        text!("Hello, world!!!");
    }
}