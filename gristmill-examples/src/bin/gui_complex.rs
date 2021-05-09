use std::sync::Arc;

use gristmill::asset::{load_asset, image::{Image, NineSliceImage}};
use gristmill::game::{Game, Window, run_game};
use gristmill_gui::{*, quad::Quad, text::{Text, Align}, button::ButtonClass, container::*, layout::*, layout_builder::*};
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

    gui.add_widget(container, Layout::offset_parent(Rect::from_size(image_size)), Quad::new_texture(player_image));
    let mut layout = Layout::with_base_size(Size::new(128, 20));
    layout.set_anchor(Side::Top, Anchor::parent(0));
    layout.set_anchor(Side::Left, Anchor::previous_sibling_opposite(PADDING));
    layout.set_anchor(Side::Right, Anchor::parent(0));
    let mut text = Text::new("Player Name".to_string());
    text.set_font(font::Font::default(), 20.0);
    gui.add_widget(container, layout, text);
    let mut layout = Layout::with_base_size(Size::new(128, 16));
    layout.set_anchor(Side::Top, Anchor::previous_sibling_opposite(0));
    layout.set_anchor(Side::Left, Anchor::previous_sibling(0));
    layout.set_anchor(Side::Right, Anchor::parent(0));
    gui.add_widget(container, layout, Text::new("second line...".to_string()));
}

fn gui_stat_row(gui: &mut Gui, container: GuiNode, stat: String, buttons: Option<(&ButtonClass, &ButtonClass)>) {
    let mut text = Text::new(stat);
    text.set_alignment(Align::Start, Align::Middle);
    gui.add_widget(container, Layout::default(), text);
    let mut text = Text::new("0".to_string());
    text.set_alignment(Align::Middle, Align::Middle);
    gui.add_widget(container, Layout::default(), text);
    if let Some((add_button, sub_button)) = buttons {
        add_button.instance(gui, container, Layout::default(), None);
        sub_button.instance(gui, container, Layout::default(), None);
    }
    else {
        gui.add(container, Layout::default());
        gui.add(container, Layout::default());
    }
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
        let sub_image: Image = load_asset("images/Subtract").unwrap();
        let sub_texture = gui_subpass.load_image(&mut gui_subpass_setup, &sub_image);

        let mut gui = Gui::new();
        let mut base_button = ButtonClass::new();
        base_button.set_texture(button_texture);
        let base_button = Arc::new(base_button);

        let layout = Layout::center_parent(Size::new(384, 256));
        let player_frame = gui.add_widget(gui.root(), layout, Quad::new_texture(frame_texture));
        let player_frame_layout = BoxLayout::new(player_frame.into(), BoxDirection::Vertical, Padding::new(PADDING));

        gui_top(&mut gui, &player_frame_layout, player_texture);

        player_frame_layout.add_widget(&mut gui, BoxSize::Exact(1), Quad::new_color(gristmill::color::black()));
        
        let bottom = player_frame_layout.add(&mut gui, BoxSize::Remaining);
        let bottom_layout = SplitLayout::new(bottom, BoxDirection::Horizontal, Padding::new_inside(PADDING * 2));
        let left_container = bottom_layout.add(&mut gui);
        let right_container = bottom_layout.add(&mut gui);
        bottom_layout.add_center_widget(&mut gui, 1, Quad::new_color(gristmill::color::black()));
        
        gui.set_container(left_container, TableContainer::new(&[0, 24, 16, 16], 16, Padding::new_inside(PADDING), Some(1)));
        let mut add_button = ButtonClass::new_inherit(base_button.clone());
        add_button.set_icon(add_texture.clone());
        let mut sub_button = ButtonClass::new_inherit(base_button.clone());
        sub_button.set_icon(sub_texture.clone());
        gui_stat_row(&mut gui, left_container, "Remaining".to_string(), None);
        gui_stat_row(&mut gui, left_container, "Strength".to_string(), Some((&add_button, &sub_button)));
        gui_stat_row(&mut gui, left_container, "Dexterity".to_string(), Some((&add_button, &sub_button)));
        gui_stat_row(&mut gui, left_container, "Intelligence".to_string(), Some((&add_button, &sub_button)));

        gui.set_container(right_container, FlowContainer::new(Padding::new_inside(PADDING)));
        for _i in 0..10 {
            gui.add_widget(right_container, Layout::with_base_size(perk_image.size()), Quad::new_texture(perk_texture.clone()));
        }
        add_button.instance(&mut gui, right_container, Layout::with_base_size(add_image.size()), None);
        
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
