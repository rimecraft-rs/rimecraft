use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct GameOptions {
    pub container: GameOptionsSerializeContainer,
}

impl GameOptions {
    pub fn new(path: &str) -> Self {
        let buf = Path::new(path).join("options.toml");
        if let Ok(mut file) = File::open(buf.clone()) {
            toml::from_str(&{
                let mut string = String::new();
                file.read_to_string(&mut string).unwrap();
                string
            })
            .unwrap()
        } else {
            Self::default()
        }
    }
}

impl<'a> Deserialize<'a> for GameOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        Ok(Self {
            container: GameOptionsSerializeContainer::deserialize(deserializer)?,
        })
    }
}

impl Serialize for GameOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        GameOptionsSerializeContainer::serialize(&self.container, serializer)
    }
}

#[derive(Deserialize, Serialize)]
pub struct GameOptionsSerializeContainer {
    pub monochrome_logo: bool,
    pub hide_lightning_flashes: bool,
    pub mouse_sensitivity: f32,
    pub view_distance: u32,
    pub simulation_distance: u32,
    pub entity_distance_scaling: f64,
    pub max_fps: u32,
    pub ao: bool,
    pub chat_opacity: f64,
    pub chat_line_spacing: f64,
    pub text_background_opacity: f64,
    pub panorama_speed: f64,
    pub high_constrace: bool,
    pub chat_scale: f64,
    pub chat_width: f64,
    pub chat_height_unfocused: f64,
    pub chat_height_focused: f64,
    pub chat_delay: f64,
    pub notification_display_time: f64,
    pub mipmap_levels: u32,
    pub biome_blend_radius: u32,
    pub mouse_wheel_sensitvity: f64,
    pub raw_mouse_input: bool,
    pub auto_jump: bool,
    pub operator_items_tab: bool,
    pub auto_suggestions: bool,
    pub chat_colors: bool,
    pub chat_links: bool,
    pub chat_links_prompt: bool,
    pub enable_vsync: bool,
    pub entity_shadows: bool,
    pub force_unicode_font: bool,
    pub invert_y_mouse: bool,
    pub discrete_mouse_scroll: bool,
    pub reduced_debug_info: bool,
    pub show_subtitles: bool,
    pub directional_audio: bool,
    pub background_for_chat_only: bool,
    pub touchscreen: bool,
    pub bobview: bool,
    pub sneak_toggled: bool,
    pub sprint_toogled: bool,
    pub hide_matched_names: bool,
    pub show_autosave_indicator: bool,
}

impl Default for GameOptionsSerializeContainer {
    fn default() -> Self {
        Self {
            monochrome_logo: true,
            hide_lightning_flashes: Default::default(),
            mouse_sensitivity: Default::default(),
            view_distance: Default::default(),
            simulation_distance: Default::default(),
            entity_distance_scaling: Default::default(),
            max_fps: Default::default(),
            ao: Default::default(),
            chat_opacity: Default::default(),
            chat_line_spacing: Default::default(),
            text_background_opacity: Default::default(),
            panorama_speed: Default::default(),
            high_constrace: Default::default(),
            chat_scale: Default::default(),
            chat_width: Default::default(),
            chat_height_unfocused: Default::default(),
            chat_height_focused: Default::default(),
            chat_delay: Default::default(),
            notification_display_time: Default::default(),
            mipmap_levels: Default::default(),
            biome_blend_radius: Default::default(),
            mouse_wheel_sensitvity: Default::default(),
            raw_mouse_input: Default::default(),
            auto_jump: Default::default(),
            operator_items_tab: Default::default(),
            auto_suggestions: Default::default(),
            chat_colors: Default::default(),
            chat_links: Default::default(),
            chat_links_prompt: Default::default(),
            enable_vsync: Default::default(),
            entity_shadows: Default::default(),
            force_unicode_font: Default::default(),
            invert_y_mouse: Default::default(),
            discrete_mouse_scroll: Default::default(),
            reduced_debug_info: Default::default(),
            show_subtitles: Default::default(),
            directional_audio: Default::default(),
            background_for_chat_only: Default::default(),
            touchscreen: Default::default(),
            bobview: Default::default(),
            sneak_toggled: Default::default(),
            sprint_toogled: Default::default(),
            hide_matched_names: Default::default(),
            show_autosave_indicator: Default::default(),
        }
    }
}
