use glam::*;
use getrandom;
use oorandom::Rand32;

// Customise these
pub const FPS: u32 = 60;
const MAX_PERCIEVE: f32 = 75.0;
const MIN_DIST: f32 = 20.0;
// Max speed is pixels per second
const MAX_SPEED: f32 = 500.0;
const MIN_SPEED: f32 = 50.0;
// The amount of speed kept when a boid bounces off the wall
const BOUNCINESS: f32 = 0.5;
const WALL_PERCENT: f32 = 0.1;
const WALL_ACC: f32 = 0.64;

const MAX_PERCIEVE_SQ: f32 = MAX_PERCIEVE * MAX_PERCIEVE;
const MIN_DIST_SQ: f32 = MIN_DIST * MIN_DIST;

const MAX_SPEED_FRAME: f32 = MAX_SPEED / FPS as f32;
const MIN_SPEED_FRAME: f32 = MIN_SPEED / FPS as f32;
const MAX_SPEED_SQ: f32 = MAX_SPEED_FRAME * MAX_SPEED_FRAME;
const MIN_SPEED_SQ: f32 = MIN_SPEED_FRAME * MIN_SPEED_FRAME;

#[derive(Clone)]
pub struct Boid {
    pub pos: Vec2,
    pub vel: Vec2,
}

impl Boid {
    fn new(rng: &mut Rand32, space: &Vec2) -> Boid {
        Boid {
            pos: Vec2::new(rng.rand_float() * space.x, rng.rand_float() * space.y),
            vel: Vec2::new(rng.rand_float() * MAX_SPEED_SQ.sqrt(), rng.rand_float() * MAX_SPEED_SQ.sqrt())
        }
    }
}

pub struct BoidFlock {
    boids: [Vec<Boid>; 2],
    len: usize,
    size: Vec2,
    cur: usize,
    rng: Rand32,
    goal: Option<Vec2>,
}

impl BoidFlock {
    pub fn new(count: usize, size: Vec2) -> BoidFlock {
        assert!(MAX_SPEED_SQ > MIN_SPEED_SQ);

        let mut seed = [0; 8];
        getrandom::getrandom(&mut seed).expect("Unable to RNG");
        let mut flock = BoidFlock {
            boids: [Vec::with_capacity(count), Vec::new()],
            len: count,
            size: size,
            cur: 0,
            rng: Rand32::new(u64::from_ne_bytes(seed)),
            goal: None,
        };
        for _ in 0..count {
            flock.boids[0].push(Boid::new(&mut flock.rng, &flock.size));
        }
        flock.boids[1] = flock.boids[0].clone();
        flock
    }

    pub fn iter(&self) -> std::slice::Iter<Boid> {
        self.boids[self.cur].iter()
    }

    pub fn resize(&mut self, new: Vec2) {
        assert!(new.min_element() > 0.0, "Negative size given");
        self.size = new;
    }

    pub fn set_goal(&mut self, new: Option<Vec2>) {
        self.goal = new;
    }

    pub fn get_goal(&self) -> Option<Vec2> {
        self.goal.clone()
    }

    pub fn update(&mut self) {
        assert_eq!(self.len, self.boids[0].len());
        assert_eq!(self.len, self.boids[1].len());

        let next = 1 - self.cur;
        let cur = self.cur;
        let split = self.boids.split_at_mut(1);
        let (from_boids, to_boids) = if cur == 0 {
            (&split.0[0], &mut split.1[0])
        }
        else {
            (&split.1[0], &mut split.0[0])
        };
        
        // Processing AI for every single bird
        for i in 0..self.len {
            let mut cur_boid = from_boids[i].clone();
            // Process AI
            Self::rule_one(&mut cur_boid, &from_boids, i);
            Self::rule_two(&mut cur_boid, &from_boids, i);
            Self::rule_three(&mut cur_boid, &from_boids, i);
            Self::rule_four(&mut cur_boid, self.size);
            // Only run rule 5 if there is currently a goal
            match self.goal {
                Some(goal) => Self::rule_five(&mut cur_boid, goal),
                None => {}
            }

            cur_boid.pos += cur_boid.vel;

            to_boids[i] = cur_boid;
        }

        self.cur = next;
    }

    fn rule_one(boid: &mut Boid, all: &Vec<Boid>, cur: usize) {
        // Rule 1: Boids fly to the center of mass of neighbours
        let mut found = 0;
        let mut percieved_center = Vec2::zero();
        for other in 0..all.len() {
            if other != cur {
                if boid.pos.distance_squared(all[other].pos) < MAX_PERCIEVE_SQ {
                    found += 1;
                    percieved_center += all[other].pos;
                }
            }
        }
        if found > 0 {
            percieved_center /= found as f32;
            boid.vel += (percieved_center - boid.pos) / 200.0;
        }
    }

    fn rule_two(boid: &mut Boid, all: &Vec<Boid>, cur: usize) {
        // Rule 2: Boids try to keep a small distance away from other boids
        for other in 0..all.len() {
            if other != cur {
                if boid.pos.distance_squared(all[other].pos) < MIN_DIST_SQ {
                    boid.vel -= (all[other].pos - boid.pos) / 20.0;
                }
            }
        }
    }

    fn rule_three(boid: &mut Boid, all: &Vec<Boid>, cur: usize) {
        // Rule 3: Boids try to match their velocity with other nearby boids
        let mut found = 0;
        let mut percieved_avg = Vec2::zero();
        for other in 0..all.len() {
            if other != cur {
                if boid.pos.distance_squared(all[other].pos) < MAX_PERCIEVE_SQ {
                    found += 1;
                    percieved_avg += all[other].vel;
                }
            }
        }
        if found > 0 {
            percieved_avg *= 1.2 / found as f32;
            boid.vel += (percieved_avg - boid.vel) / 20.0;
        }
    }

    fn rule_four(boid: &mut Boid, size: Vec2) {
        // Rule 4: Boids have a limited speed, and try to stay onscreen
        // Speed limit
        let len_sq = boid.vel.length_squared();
        if len_sq > MAX_SPEED_SQ {
            boid.vel *= MAX_SPEED_FRAME / len_sq.sqrt();
        }

        // If a boids velocity will take it off screen, 'bounce' off instead
        // Calculating the next position it will take, and if that will carry it offscreen then multiply the vel by neg bounciness
        let next_pos = boid.pos + boid.vel;
        if (next_pos.x < 0.0 && boid.vel.x < 0.0) || (next_pos.x > size.x && boid.vel.x > 0.0) {
            boid.vel.x *= -BOUNCINESS;
        }
        if (next_pos.y < 0.0 && boid.vel.y < 0.0) || (next_pos.y > size.y && boid.vel.y > 0.0) {
            boid.vel.y *= -BOUNCINESS;
        }

        // Additionally, accelerate away from walls in general
        if boid.pos.x < size.x * WALL_PERCENT {
            boid.vel.x += WALL_ACC;
        }
        if boid.pos.x > size.x * (1.0 - WALL_PERCENT) {
            boid.vel.x -= WALL_ACC;
        }
        if boid.pos.y < size.y * WALL_PERCENT {
            boid.vel.y += WALL_ACC;
        }
        if boid.pos.y > size.y * (1.0 - WALL_PERCENT) {
            boid.vel.y -= WALL_ACC;
        }
    }

    fn rule_five(boid: &mut Boid, goal: Vec2) {
        // Rule 5: Boids tend towards the goal if it exists
        boid.vel += (goal - boid.pos) / 200.0;
    }
}