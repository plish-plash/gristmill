use gristmill::asset::{load_asset, image::{Image, NineSliceImage}};
use gristmill::game::{Game, Window, run_game};
use gristmill_gui::{*, quad::Quad, text::Text, button::ButtonBuilder, container::FlowContainer, layout::*, layout_builder::*};
use gristmill::renderer::{RenderPassInfo, Renderer, RenderContext, pass::{RenderPass, GeometryGuiPass}};
use gristmill::color::Color;
use gristmill::geometry2d::*;
use gristmill::input::{InputSystem, InputActions, CursorAction, ActionState};

use gristmill_examples::basic_geo_subpass::BasicGeoSubpass;
use gristmill_gui::renderer::GuiSubpass;

// -------------------------------------------------------------------------------------------------

type Scene = ((), Gui);

#[derive(Default)]
struct GuiGameInput {
    primary: CursorAction,
}

impl InputActions for GuiGameInput {
    fn end_frame(&mut self) {
        self.primary.end_frame();
    }
    fn set_action_state(&mut self, target: &str, state: ActionState) {
        match target {
            "primary" => self.primary.set_state(state),
            _ => (),
        }
    }
}

impl GuiInputActions for GuiGameInput {
    fn primary(&self) -> &CursorAction { &self.primary }
}

struct GuiGame {
    render_pass: GeometryGuiPass<BasicGeoSubpass, GuiSubpass>,
    scene: Scene,
    input: GuiGameInput,
}

const PADDING: i32 = 8;

fn gui_top(gui: &mut Gui, parent: &BoxLayout, player_image: GuiTexture) {
    let image_size = player_image.size().unwrap();
    let container = parent.add(gui, BoxSize::Exact(image_size.height));

    gui.add_widget(container, Quad::new_texture(player_image), Layout::offset_parent(Rect::from_size(image_size)));
    let mut layout = Layout::with_base_size(Size::new(128, 20));
    layout.set_anchor(Side::Top, Anchor::parent(0));
    layout.set_anchor(Side::Left, Anchor::previous_sibling_opposite(PADDING));
    layout.set_anchor(Side::Right, Anchor::parent(0));
    let mut text = Text::new();
    text.set_font(font::Font::default(), 20.0);
    text.set_text("Player Name".to_string());
    gui.add_widget(container, text, layout);
    let mut layout = Layout::with_base_size(Size::new(128, 16));
    layout.set_anchor(Side::Top, Anchor::previous_sibling_opposite(0));
    layout.set_anchor(Side::Left, Anchor::previous_sibling(0));
    layout.set_anchor(Side::Right, Anchor::parent(0));
    gui.add_widget(container, Text::with_text("second line...".to_string()), layout);
}

impl Game for GuiGame {
    fn load(renderer: &mut Renderer) -> (Self, RenderPassInfo) {
        let mut render_pass = GeometryGuiPass::<BasicGeoSubpass, GuiSubpass>::with_clear_color(renderer, Color::new(0.0, 0.8, 0.8, 1.0));
        let mut gui_subpass_setup = renderer.subpass_setup(render_pass.info(), 1);
        let gui_subpass = render_pass.subpass1();
        let frame_image: NineSliceImage = load_asset("images/FrameSquare").unwrap();
        let frame_texture = gui_subpass.load_nine_slice_image(&mut gui_subpass_setup, &frame_image);
        let button_image: NineSliceImage = load_asset("images/FrameRounded").unwrap();
        let button_texture = gui_subpass.load_nine_slice_image(&mut gui_subpass_setup, &button_image);
        let player_image: Image = load_asset("images/Portrait").unwrap();
        let player_texture = gui_subpass.load_image(&mut gui_subpass_setup, &player_image);
        let perk_image: Image = load_asset("images/Perk1").unwrap();
        let perk_texture = gui_subpass.load_image(&mut gui_subpass_setup, &perk_image);
        let add_image: Image = load_asset("images/Add").unwrap();
        let add_texture = gui_subpass.load_image(&mut gui_subpass_setup, &add_image);

        let mut gui = Gui::new();

        let layout = Layout::center_parent(Size::new(384, 256));
        let player_frame = gui.add_widget(gui.root(), Quad::new_texture(frame_texture), layout);
        let mut player_frame_layout = BoxLayout::new(player_frame.into(), BoxDirection::Vertical, PADDING);
        player_frame_layout.set_pad_outside(true);

        gui_top(&mut gui, &player_frame_layout, player_texture);

        player_frame_layout.add_widget(&mut gui, Quad::new_color(gristmill::color::black()), BoxSize::Exact(1));
        
        let bottom = player_frame_layout.add(&mut gui, BoxSize::Remaining);
        let bottom_layout = SplitLayout::new(bottom, BoxDirection::Horizontal, PADDING * 2);
        let left_container = bottom_layout.add(&mut gui);
        let right_container = bottom_layout.add(&mut gui);
        bottom_layout.add_center_widget(&mut gui, Quad::new_color(gristmill::color::black()), 1);

        gui.set_container(right_container, FlowContainer::new(PADDING));
        for _i in 0..10 {
            gui.add_widget(right_container, Quad::new_texture(perk_texture.clone()), Layout::with_base_size(perk_image.size()));
        }
        ButtonBuilder::new()
            .with_texture(button_texture)
            .with_icon(add_texture)
            .build(&mut gui, right_container, Layout::with_base_size(add_image.size()));
        
        let render_pass_info = render_pass.info();
        (GuiGame {
            render_pass,
            scene: ((), gui),
            input: GuiGameInput::default(),
        }, render_pass_info)
    }

    fn resize(&mut self, dimensions: Size) {
        self.render_pass.set_dimensions(dimensions);
    }

    fn update(&mut self, _window: &Window, input_system: &mut InputSystem, _delta: f64) -> bool {
        input_system.dispatch_queue(&mut self.input);
        let gui = &mut self.scene.1;
        gui.process_input(&self.input, move |event| {
            match event {
                GuiActionEvent::Action(_) => (),
                _ => (),
            }
        });
        true
    }

    fn render(&mut self, _renderer: &mut Renderer, context: &mut RenderContext) {
        self.render_pass.render(context, &mut self.scene);
    }
}

fn main() {
    gristmill_gui::font::load_fonts(vec!["fonts/DejaVuSans".to_string()]); // TODO fonts should be autoloaded
    run_game::<GuiGame>();
}
