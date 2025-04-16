use std::path::Path;

use gristmill::{
    color::Color,
    math::{Pos2, Rect, Vec2},
    particles,
    scene2d::{Instance, ScrollCamera, UvRect},
    DrawMetrics,
};
use gristmill_miniquad::{
    Context, InputEvent, Pipeline2D, Renderer2D, Texture, WindowConfig, WindowSetup,
};

struct GameAssets {
    particle: Texture,
}

impl GameAssets {
    fn load(context: &mut Context) -> Self {
        GameAssets {
            particle: Texture::load(context, Path::new("particle.png")).unwrap(),
        }
    }
}

struct ParticleData {
    velocity: Vec2,
    lifetime: f32,
}

#[derive(Default)]
struct ParticleState {
    position: Pos2,
    alive: f32,
}

struct ParticleSolver {
    particle_size: Vec2,
}

impl particles::ParticleSolver for ParticleSolver {
    type Data = ParticleData;
    type State = ParticleState;
    type Instance = Instance;
    fn update(&self, data: &ParticleData, state: &mut ParticleState, dt: f32) -> bool {
        state.position += data.velocity * dt;
        state.alive += dt;
        state.alive < data.lifetime
    }
    fn draw(&self, data: &ParticleData, state: &ParticleState) -> Instance {
        Instance {
            rect: Rect::from_center_size(state.position, self.particle_size),
            uv: UvRect::default(),
            color: Color::new_rgba(1.0, 1.0, 1.0, 1.0 - (state.alive / data.lifetime)),
        }
    }
}

type ParticleSystem = particles::ParticleSystem<ParticleSolver, Pipeline2D>;

struct Game {
    renderer: Renderer2D,
    camera: ScrollCamera,
    particles: ParticleSystem,
    spawn_timer: f32,
    _assets: GameAssets,
}

impl gristmill_miniquad::Game for Game {
    fn init(mut context: Context, screen_size: Vec2) -> Self {
        let assets = GameAssets::load(&mut context);
        let renderer = Renderer2D::new(context);
        Game {
            renderer,
            camera: ScrollCamera {
                screen_size,
                center: Pos2::ZERO,
                scale: 1.0,
            },
            particles: ParticleSystem::new(
                ParticleSolver {
                    particle_size: assets.particle.size().to_vec2(),
                },
                assets.particle.material(),
            ),
            spawn_timer: 0.0,
            _assets: assets,
        }
    }

    fn input(&mut self, _event: InputEvent) {}

    fn update(&mut self, dt: f32) {
        self.spawn_timer += dt * 4.0;
        self.particles.spawn(
            ParticleData {
                velocity: Vec2::angled(self.spawn_timer) * 64.0,
                lifetime: 2.0,
            },
            ParticleState::default(),
        );
        self.particles.update(dt);
    }

    fn resize(&mut self, screen_size: Vec2) {
        self.camera.screen_size = screen_size;
    }

    fn draw(&mut self) -> DrawMetrics {
        self.renderer.begin_render(Color::BLACK);
        let mut batcher = self.renderer.bind_pipeline();
        batcher.set_camera(self.camera.transform());
        self.particles.draw(&mut batcher);
        std::mem::drop(batcher);
        self.renderer.end_render()
    }
}

fn main() {
    gristmill::asset::set_base_path("examples/assets").unwrap();
    gristmill_miniquad::start::<Game>(
        WindowSetup::from_title("Particles Example".to_string()),
        WindowConfig::default(),
    );
}
