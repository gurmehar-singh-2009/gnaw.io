enum EntityType {
    Bush,
    Player,
    Stone,
    Tower,
    Tree,
    Zombie,
}

pub struct EntityManager {
    pub entity_types: Vec<EntityType>,
    pub positions: Vec<(f32, f32)>,
    pub velocities: Vec<(f32, f32)>,
    pub health: Vec<(f32, f32)>, // current, max
}
