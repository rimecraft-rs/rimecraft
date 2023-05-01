use serde::{Deserialize, Serialize};

pub struct GameOptions {
    container: GameOptionsSerializeContainer,
}

impl GameOptions {}

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
struct GameOptionsSerializeContainer {
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
