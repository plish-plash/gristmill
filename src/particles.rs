use crate::Batcher;

pub trait ParticleSolver {
    type Data;
    type State;
    type DrawParams: Eq + PartialOrd;
    type DrawInstance;
    fn update(&self, data: &Self::Data, state: &mut Self::State, dt: f32) -> bool;
    fn draw_params(&self) -> Self::DrawParams;
    fn draw(&self, data: &Self::Data, state: &Self::State) -> Self::DrawInstance;
}

pub struct ParticleSystem<S: ParticleSolver> {
    particles: Vec<(S::Data, S::State)>,
    solver: S,
}

impl<S: ParticleSolver> ParticleSystem<S> {
    pub fn new(solver: S) -> Self {
        ParticleSystem {
            particles: Vec::new(),
            solver,
        }
    }
    pub fn spawn(&mut self, data: S::Data, state: S::State) {
        self.particles.push((data, state));
    }
    pub fn update(&mut self, dt: f32) {
        self.particles
            .retain_mut(|(data, state)| self.solver.update(data, state, dt));
    }
    pub fn draw(&self, batcher: &mut Batcher<S::DrawParams, S::DrawInstance>) {
        batcher.get_batch(self.solver.draw_params()).extend(
            self.particles
                .iter()
                .map(|(data, state)| self.solver.draw(data, state)),
        );
    }
}
impl<S: ParticleSolver> ParticleSystem<S>
where
    S::Data: Clone,
    S::State: Clone,
{
    pub fn spawn_many(&mut self, data: S::Data, state: S::State, count: usize) {
        self.particles
            .resize(self.particles.len() + count, (data, state));
    }
}
