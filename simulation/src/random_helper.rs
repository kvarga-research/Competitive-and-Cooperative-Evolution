use rand::{Rng, SeedableRng, rngs::StdRng};

#[derive(Clone)]
pub struct RandomHelper {
    screen_size_x: f32,
    screen_size_y: f32,
    rng: StdRng,
}

impl RandomHelper {
    pub fn new(screen_size_x: f32, screen_size_y: f32, seed: u64) -> Self {
        RandomHelper {
            screen_size_x: screen_size_x,
            screen_size_y: screen_size_y,
            rng: StdRng::seed_from_u64(seed),
        }
    }
    pub fn random_coordinate(&mut self) -> (f32, f32) {
        (
            self.rng.gen_range(40.0, self.screen_size_x - 40.0),
            self.rng.gen_range(40.0, self.screen_size_y - 40.0),
        )
    }

    pub fn random_between(&mut self, min: f32, max: f32) -> f32 {
        self.rng.gen_range(min, max)
    }
}
