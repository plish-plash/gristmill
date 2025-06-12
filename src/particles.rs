use bytemuck::Pod;
use silica_wgpu::{Context, ResizableBuffer, Texture};

use crate::world2d::{DrawCallBuilder, Quad};

pub trait ParticleSolver {
    type Particle;
    type Primitive: Pod;
    fn update(&self, particle: &mut Self::Particle, dt: f32) -> bool;
    fn draw(&self, particle: &Self::Particle) -> Self::Primitive;
}

pub struct ParticleSystem<S: ParticleSolver> {
    particles: Vec<S::Particle>,
    solver: S,
    texture: Texture,
    primitives: Vec<S::Primitive>,
    primitive_buffer: ResizableBuffer<S::Primitive>,
    changed: bool,
}

impl<S> ParticleSystem<S>
where
    S: ParticleSolver,
{
    pub fn new(context: &Context, solver: S, texture: Texture) -> Self {
        ParticleSystem {
            particles: Vec::new(),
            solver,
            texture,
            primitives: Vec::new(),
            primitive_buffer: ResizableBuffer::new(context),
            changed: false,
        }
    }
    pub fn with_particles(
        context: &Context,
        particles: Vec<S::Particle>,
        solver: S,
        texture: Texture,
    ) -> Self {
        ParticleSystem {
            particles,
            solver,
            texture,
            primitives: Vec::new(),
            primitive_buffer: ResizableBuffer::new(context),
            changed: true,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }
    pub fn spawn(&mut self, particle: S::Particle) {
        self.particles.push(particle);
        self.changed = true;
    }
    pub fn update(&mut self, dt: f32) {
        self.particles
            .retain_mut(|particle| self.solver.update(particle, dt));
        self.changed = true;
    }
}
impl<S> ParticleSystem<S>
where
    S: ParticleSolver,
    S::Particle: Clone,
{
    pub fn spawn_many(&mut self, particle: S::Particle, count: usize) {
        self.particles
            .resize(self.particles.len() + count, particle);
        self.changed = true;
    }
}
impl<S> ParticleSystem<S>
where
    S: ParticleSolver<Primitive = Quad>,
{
    pub fn draw(&mut self, renderer: &mut DrawCallBuilder) {
        if self.changed {
            self.primitives.clear();
            for particle in self.particles.iter() {
                self.primitives.push(self.solver.draw(particle));
            }
            self.primitive_buffer
                .set_data(renderer.context, &self.primitives);
            self.changed = false;
        }
        renderer.draw_buffer(&self.texture, &self.primitive_buffer);
    }
}
