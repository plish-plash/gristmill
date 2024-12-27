use std::hash::Hash;

use crate::Scene;

pub trait ParticleSolver {
    type Data;
    type State;
    type DrawParams: Eq + Hash;
    type DrawInstance;
    fn update(&self, data: &Self::Data, state: &mut Self::State, dt: f32) -> bool;
    fn draw_params(&self) -> Self::DrawParams;
    fn draw(&self, data: &Self::Data, state: &Self::State) -> Self::DrawInstance;
}

pub struct ParticleSystem<S: ParticleSolver, L> {
    particles: Vec<(S::Data, S::State)>,
    solver: S,
    layer: L,
}

impl<S: ParticleSolver, L> ParticleSystem<S, L> {
    pub fn new(solver: S, layer: L) -> Self {
        ParticleSystem {
            particles: Vec::new(),
            solver,
            layer,
        }
    }
    pub fn spawn(&mut self, data: S::Data, state: S::State) {
        self.particles.push((data, state));
    }
    pub fn update(&mut self, dt: f32) {
        self.particles
            .retain_mut(|(data, state)| self.solver.update(data, state, dt));
    }
}
impl<S: ParticleSolver, L: Ord> ParticleSystem<S, L>
where
    S::Data: Clone,
    S::State: Clone,
{
    pub fn spawn_many(&mut self, data: S::Data, state: S::State, count: usize) {
        self.particles
            .resize(self.particles.len() + count, (data, state));
    }
}
impl<S: ParticleSolver, L> ParticleSystem<S, L>
where
    L: Clone + Ord,
{
    pub fn queue_draw(&self, scene: &mut Scene<L, S::DrawParams, S::DrawInstance>) {
        scene.queue_all(
            self.layer.clone(),
            self.solver.draw_params(),
            self.particles
                .iter()
                .map(|(data, state)| self.solver.draw(data, state)),
        );
    }
}
