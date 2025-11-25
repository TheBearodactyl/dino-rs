#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DinoState {
    Running,
    Jumping,
    Crouching,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ObstacleType {
    SmallCactus,
    MediumCactus,
    TallCactus,
    WideCactus,
    PterodactylLow,
    PterodactylMid,
    PterodactylHigh,
}

#[derive(Clone)]
pub struct Obstacle {
    pub x: f32,
    pub obstacle_type: ObstacleType,
}

impl Obstacle {
    pub fn new(x: f32, obstacle_type: ObstacleType) -> Self {
        Self { x, obstacle_type }
    }
}

#[derive(Clone)]
pub struct Cloud {
    pub x: f32,
    pub y: usize,
}

impl Cloud {
    pub fn new(x: f32, y: usize) -> Self {
        Self { x, y }
    }
}

pub struct Dino {
    pub state: DinoState,
    pub y: f32,
    pub velocity_y: f32,
}

impl Dino {
    pub fn new() -> Self {
        Self {
            state: DinoState::Running,
            y: 0.0,
            velocity_y: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.state = DinoState::Running;
        self.y = 0.0;
        self.velocity_y = 0.0;
    }
}
