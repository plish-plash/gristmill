use crate::{Batcher, Buffer, Pipeline};

pub trait ParticleSolver {
    type Data;
    type State;
    type Instance;
    fn update(&self, data: &Self::Data, state: &mut Self::State, dt: f32) -> bool;
    fn draw(&self, data: &Self::Data, state: &Self::State) -> Self::Instance;
}

pub struct ParticleSystem<S: ParticleSolver, P: Pipeline> {
    particles: Vec<(S::Data, S::State)>,
    solver: S,
    material: P::Material,
    instances: P::InstanceBuffer,
    changed: bool,
}

impl<S, P> ParticleSystem<S, P>
where
    S: ParticleSolver,
    P: Pipeline<Instance = S::Instance>,
{
    pub fn new(solver: S, material: P::Material) -> Self {
        ParticleSystem {
            particles: Vec::new(),
            solver,
            material,
            instances: P::InstanceBuffer::new(),
            changed: false,
        }
    }
    pub fn with_particles(
        particles: Vec<(S::Data, S::State)>,
        solver: S,
        material: P::Material,
    ) -> Self {
        ParticleSystem {
            particles,
            solver,
            material,
            instances: P::InstanceBuffer::new(),
            changed: true,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }
    pub fn spawn(&mut self, data: S::Data, state: S::State) {
        self.particles.push((data, state));
        self.changed = true;
    }
    pub fn update(&mut self, dt: f32) {
        self.particles
            .retain_mut(|(data, state)| self.solver.update(data, state, dt));
        self.changed = true;
    }
    pub fn draw(&mut self, batcher: &mut Batcher<P>) {
        if self.changed {
            self.instances.clear();
            for (data, state) in self.particles.iter() {
                self.instances.push(self.solver.draw(data, state));
            }
            self.changed = false;
        }
        batcher.flush();
        batcher.pipeline.draw(
            batcher.context,
            &batcher.camera,
            &self.material,
            &mut self.instances,
        );
    }
}
impl<S, P> ParticleSystem<S, P>
where
    S: ParticleSolver,
    P: Pipeline<Instance = S::Instance>,
    S::Data: Clone,
    S::State: Clone,
{
    pub fn spawn_many(&mut self, data: S::Data, state: S::State, count: usize) {
        self.particles
            .resize(self.particles.len() + count, (data, state));
        self.changed = true;
    }
}
