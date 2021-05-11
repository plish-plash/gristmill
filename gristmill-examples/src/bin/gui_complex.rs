use std::sync::Arc;

use gristmill::asset::{Asset, Resources, image::{Image, NineSliceImage}, resource::AssetItem};
use gristmill::game::{Game, Window, run_game};
use gristmill_gui::{*, quad::Quad, text::{Text, Align}, button::ButtonClass, event::{GuiActionEvent, GuiActionEventRef}, container::*, layout::*, layout_builder::*, listener};
use gristmill::renderer::{RenderLoader, LoadRef, RenderContext, pass::{RenderPass, RenderPass3D2D}, loader::AssetListLoader};
use gristmill::color::Color;
use gristmill::geometry2d::*;
use gristmill::input::{InputSystem, InputActions, CursorAction, ActionState};

use gristmill_examples::basic_geo_renderer::BasicGeoRenderer;
use gristmill_gui::renderer::{GuiRenderer, GuiRendererLoad};

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
    scene: Scene,
    input: GuiGameInput,
    player: Player,
    player_window: PlayerWindow,
}

struct PlayerStats {
    unspent: u32,
    strength: u32,
    dexterity: u32,
    intelligence: u32,
}

impl PlayerStats {
    fn get_mut(&mut self, index: usize) -> &mut u32 {
        match index {
            0 => &mut self.strength,
            1 => &mut self.dexterity,
            2 => &mut self.intelligence,
            _ => panic!("invalid index"),
        }
    }
}

struct Player {
    name: String,
    level: u32,
    stats: PlayerStats,
    //perks: Vec<PlayerPerk>,
}

impl Player {
    fn new() -> Player {
        Player {
            name: "Test Name".to_string(),
            level: 45,
            stats: PlayerStats { unspent: 3, strength: 1, dexterity: 1, intelligence: 1 },
        }
    }
}

struct GuiAssetLoader<'a> {
    gui_loader: LoadRef<'a, GuiRenderer>,
    texture_list: GuiTextureList,
}

impl<'a> AssetListLoader<'a> for GuiAssetLoader<'a> {
    type RenderPass = <GuiGame as Game>::RenderPass;
    type Output = GuiTextureList;
    fn name() -> &'static str { "gui" }
    fn new(loader: &'a mut RenderLoader, render_pass: &'a mut Self::RenderPass) -> Self {
        GuiAssetLoader {
            gui_loader: render_pass.scene_render1(loader),
            texture_list: GuiTextureList::new(),
        }
    }
    fn load(&mut self, item: &AssetItem) {
        match &item.asset_type as &str {
            "" => {
                if let Some(image) = Image::try_read(&item.asset_path) {
                    let texture = self.gui_loader.load_image(&image);
                    self.texture_list.insert(item.name.to_owned(), texture);
                }
            },
            "nine_slice" => {
                if let Some(image) = NineSliceImage::try_read(&item.asset_path) {
                    let texture = self.gui_loader.load_nine_slice_image(&image);
                    self.texture_list.insert(item.name.to_owned(), texture);
                }
            },
            _ => log::warn!("Invalid asset type \"{}\"", item.asset_type)
        }
        
    }
    fn finish(self) -> Self::Output {
        self.texture_list
    }
}

struct PlayerWindow {
    root: GuiNode,
    name_text: WidgetNode<Text>,
    level_text: WidgetNode<Text>,
    stat_unspent: GuiValue<u32>,
    stats: [GuiValue<u32>; 3],
}

impl PlayerWindow {
    // TODO inflate from file
    const PADDING: i32 = 8;
    fn build_top(gui: &mut Gui, parent: &BoxLayout, player_image: GuiTexture) -> (WidgetNode<Text>, WidgetNode<Text>) {
        let image_size = player_image.size().unwrap();
        let container = parent.add(gui, BoxSize::Exact(image_size.height));

        gui.add_widget(container, Layout::offset_parent(Rect::from_size(image_size)), Quad::new_texture(player_image));
        let mut layout = Layout::new_size(Size::new(128, 20));
        layout.set_anchor(Side::Top, Anchor::parent(0));
        layout.set_anchor(Side::Left, Anchor::previous_sibling_opposite(PlayerWindow::PADDING));
        layout.set_anchor(Side::Right, Anchor::parent(0));
        let mut text = Text::new_empty();
        text.set_font(font::Font::default(), 20.0);
        let name_text = gui.add_widget(container, layout, text);
        let mut layout = Layout::new_size(Size::new(128, 16));
        layout.set_anchor(Side::Top, Anchor::previous_sibling_opposite(0));
        layout.set_anchor(Side::Left, Anchor::previous_sibling(0));
        layout.set_anchor(Side::Right, Anchor::parent(0));
        let level_text = gui.add_widget(container, layout, Text::new_empty());
        (name_text, level_text)
    }
    fn build_stat_row(gui: &mut Gui, container: GuiNode, stat: String, buttons: Option<(usize, &mut GuiValue<u32>, &ButtonClass, &ButtonClass)>) -> GuiValue<u32> {
        let mut text = Text::new(stat);
        text.set_alignment(Align::Start, Align::Middle);
        gui.add_widget(container, Layout::default(), text);
        let mut text = Text::new("0".to_string());
        text.set_alignment(Align::Middle, Align::Middle);
        let value_text = gui.add_widget(container, Layout::default(), text);
        
        let mut stat_value = GuiValue::new();
        stat_value.add_listener(listener::ConvertString(listener::SetText(value_text)));

        if let Some((index, stat_unspent, add_button, sub_button)) = buttons {
            let add = add_button.instance_builder()
                .with_press_event(GuiActionEvent::NamedIndex("stat_add".to_string(), index))
                .build(gui, container);
            stat_unspent.add_listener(listener::Compare(listener::Comparison::NotEqual, 0, listener::EnableButton(add)));
            let sub = sub_button.instance_builder()
                .with_press_event(GuiActionEvent::NamedIndex("stat_sub".to_string(), index))
                .build(gui, container);
            stat_value.add_listener(listener::Compare(listener::Comparison::NotEqual, 0, listener::EnableButton(sub)));
        }
        else {
            gui.add(container, Layout::default());
            gui.add(container, Layout::default());
        }
    
        stat_value
    }
    fn build(gui: &mut Gui, textures: &GuiTextureList) -> PlayerWindow {
        let mut base_button = ButtonClass::new();
        base_button.set_texture(textures["button"].clone());
        let base_button = Arc::new(base_button);

        let layout = Layout::center_parent(Size::new(384, 256));
        let root = gui.add_widget(gui.root(), layout, Quad::new_texture(textures["frame"].clone())).into();
        gui.set_event_handler(root);
        let root_layout = BoxLayout::new(root, BoxDirection::Vertical, Padding::new(PlayerWindow::PADDING));

        let (name_text, level_text) = PlayerWindow::build_top(gui, &root_layout, textures["player"].clone());

        root_layout.add_widget(gui, BoxSize::Exact(1), Quad::new_color(gristmill::color::black()));
        
        let bottom = root_layout.add(gui, BoxSize::Remaining);
        let bottom_layout = SplitLayout::new(bottom, BoxDirection::Horizontal, Padding::new_inside(PlayerWindow::PADDING * 2));
        let left_container = bottom_layout.add(gui);
        let right_container = bottom_layout.add(gui);
        bottom_layout.add_center_widget(gui, 1, Quad::new_color(gristmill::color::black()));
        
        gui.set_container(left_container, TableContainer::new(&[0, 24, 16, 16], 16, Padding::new_inside(PlayerWindow::PADDING), Some(1)));
        let mut add_button = ButtonClass::new_inherit(base_button.clone());
        add_button.set_icon(textures["add"].clone());
        let mut sub_button = ButtonClass::new_inherit(base_button.clone());
        sub_button.set_icon(textures["sub"].clone());
        let mut stat_unspent = PlayerWindow::build_stat_row(gui, left_container, "Remaining".to_string(), None);
        let stats = [
            PlayerWindow::build_stat_row(gui, left_container, "Strength".to_string(), Some((0, &mut stat_unspent, &add_button, &sub_button))),
            PlayerWindow::build_stat_row(gui, left_container, "Dexterity".to_string(), Some((1, &mut stat_unspent, &add_button, &sub_button))),
            PlayerWindow::build_stat_row(gui, left_container, "Intelligence".to_string(), Some((2, &mut stat_unspent, &add_button, &sub_button))),
        ];

        let perk_texture = &textures["perk"];
        let perk_texture_size = perk_texture.size().unwrap();
        gui.set_container(right_container, FlowContainer::new(Padding::new_inside(PlayerWindow::PADDING)));
        for _i in 0..10 {
            gui.add_widget(right_container, Layout::new_size(perk_texture_size), Quad::new_texture(perk_texture.clone()));
        }
        add_button.instance_builder()
            .with_layout(Layout::new_size(perk_texture_size))
            .build(gui, right_container);

        PlayerWindow { root, name_text, level_text, stat_unspent, stats }
    }

    fn show(&mut self, gui: &mut Gui, player: &Player) {
        gui.get_mut(self.name_text).unwrap().set_text(player.name.clone());
        gui.get_mut(self.level_text).unwrap().set_text(format!("Level: {}", player.level));
        self.update_stats(gui, player);
    }
    fn update(&mut self, gui: &mut Gui, player: &mut Player) {
        let mut stats_changed = false;
        gui.get_events(self.root).unwrap().dispatch_queue(|event| {
            match event.as_ref() {
                GuiActionEventRef::NamedIndex("stat_add", index) => {
                    if player.stats.unspent > 0 {
                        player.stats.unspent -= 1;
                        *player.stats.get_mut(index) += 1;
                        stats_changed = true;
                    }
                },
                GuiActionEventRef::NamedIndex("stat_sub", index) => {
                    let stat = player.stats.get_mut(index);
                    if *stat > 0 {
                        *stat -= 1;
                        player.stats.unspent += 1;
                        stats_changed = true;
                    }
                },
                _ => (),
            }
        });
        if stats_changed {
            self.update_stats(gui, player);
        }
    }
    fn update_stats(&mut self, gui: &mut Gui, player: &Player) {
        self.stat_unspent.set(gui, player.stats.unspent);
        self.stats[0].set(gui, player.stats.strength);
        self.stats[1].set(gui, player.stats.dexterity);
        self.stats[2].set(gui, player.stats.intelligence);
    }
}

impl Game for GuiGame {
    type RenderPass = RenderPass3D2D<BasicGeoRenderer, GuiRenderer>;
    fn load(mut resources: Resources, loader: &mut RenderLoader) -> (Self, Self::RenderPass) {
        let mut render_pass = Self::RenderPass::with_clear_color(loader, Color::new(0.0, 0.8, 0.8, 1.0));
        let textures = loader.load_assets::<GuiAssetLoader>(&mut render_pass, resources.get("gui_textures"));

        let mut gui = Gui::new();
        let mut player_window = PlayerWindow::build(&mut gui, &textures);

        let player = Player::new();
        player_window.show(&mut gui, &player);
        
        (GuiGame {
            scene: ((), gui),
            input: GuiGameInput::default(),
            player,
            player_window,
        }, render_pass)
    }

    fn update(&mut self, _window: &Window, input_system: &mut InputSystem, _delta: f64) -> bool {
        input_system.dispatch_queue(&mut self.input);
        let gui = &mut self.scene.1;
        gui.process_input(&self.input);
        self.player_window.update(gui, &mut self.player);
        true
    }

    fn render(&mut self, _loader: &mut RenderLoader, context: &mut RenderContext, render_pass: &mut Self::RenderPass) {
        render_pass.render(context, &mut self.scene);
    }
}

fn main() {
    let mut resources = Resources::new();
    gristmill_gui::font::load_fonts(&mut resources);
    run_game::<GuiGame>(resources);
}
