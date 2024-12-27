use std::path::Path;

use gristmill::{
    color::Color,
    math::{Pos2, Rect, Vec2},
    particles,
    scene2d::{Camera, Instance, UvRect},
    DrawMetrics,
};
use gristmill_miniquad::{Context, DrawParams, Game, InputEvent, Renderer2D, Scene2D, Texture};

type Layer = u32;

struct GameAssets {
    particle: Texture,
}

impl GameAssets {
    fn load(context: &mut Context) -> Self {
        GameAssets {
            particle: Texture::load(context, Path::new("examples/assets/particle.png")).unwrap(),
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
    texture: Texture,
}

impl particles::ParticleSolver for ParticleSolver {
    type Data = ParticleData;
    type State = ParticleState;
    type DrawParams = DrawParams;
    type DrawInstance = Instance;

    fn update(&self, data: &Self::Data, state: &mut Self::State, dt: f32) -> bool {
        state.position += data.velocity * dt;
        state.alive += dt;
        state.alive < data.lifetime
    }
    fn draw_params(&self) -> Self::DrawParams {
        DrawParams::texture(&self.texture)
    }
    fn draw(&self, data: &Self::Data, state: &Self::State) -> Self::DrawInstance {
        Instance {
            rect: Rect::from_center_size(state.position, self.texture.size().to_vec2()),
            uv: UvRect::default(),
            color: Color::new_rgba(1.0, 1.0, 1.0, 1.0 - (state.alive / data.lifetime)),
        }
    }
}

type ParticleSystem = particles::ParticleSystem<ParticleSolver, Layer>;

struct MyGame {
    context: Context,
    renderer: Renderer2D,
    camera: Camera,
    scene: Scene2D<Layer>,
    particles: ParticleSystem,
    spawn_timer: f32,
}

impl Game for MyGame {
    fn init(mut context: Context, screen_size: Vec2) -> Self {
        let assets = GameAssets::load(&mut context);
        let renderer = Renderer2D::new(&mut context, None);
        MyGame {
            context,
            renderer,
            camera: Camera {
                screen_size,
                center: Pos2::ZERO,
                scale: 1.0,
            },
            scene: Scene2D::new(),
            particles: ParticleSystem::new(
                ParticleSolver {
                    texture: assets.particle,
                },
                0,
            ),
            spawn_timer: 0.0,
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
        self.particles.queue_draw(&mut self.scene);

        self.renderer.begin_render(&mut self.context, Color::BLACK);
        self.renderer
            .set_camera(&mut self.context, self.camera.transform());
        self.scene.draw(&mut self.renderer, &mut self.context);
        self.renderer.end_render(&mut self.context)
    }
}

fn main() {
    gristmill_miniquad::start::<MyGame>(Default::default());
}
