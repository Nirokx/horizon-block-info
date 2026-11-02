// Block Info: a small box naming what the crosshair is on, a block or a mob,
// with the tool it wants, how far a crop has grown, the mod it comes from and
// whatever else the config turns on. The toggle key (I) or `/blockinfo` shows
// and hides it; the settings screen (O), the settings page in Options > Mods
// and `config/block_info.cfg` change the rest.
// This file is both the mod's Cargo library (its tests) and the module the
// bundle adds to the game: no game types here, the hook adapters bring plain
// values in and take draws out.
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

/// The mod's id: its settings folder and its keys' lines.
pub const ID: &str = "block_info";
pub const NAME: &str = "Block Info";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "What the crosshair is on: name, tool, crop growth, mod and more. \
     I shows / hides it, O opens its settings; also here or in config/block_info.cfg.";
/// The keys' names and defaults.
pub const KEY_TOGGLE: &str = "toggle";
pub const DEFAULT_TOGGLE: &str = "key.keyboard.i";
pub const KEY_MENU: &str = "menu";
pub const DEFAULT_MENU: &str = "key.keyboard.o";
/// The chat command, without its slash.
pub const COMMAND: &str = "blockinfo";
/// The config file, in the game's `config/` folder.
pub const CONFIG_FILE: &str = "block_info.cfg";
/// A line of the game's font, in GUI pixels.
const FONT_HEIGHT: f32 = 8.0;
/// An item icon's side.
const ICON: f32 = 16.0;
/// Between the icon and the text.
const ICON_GAP: f32 = 4.0;
/// The scale the settings page steps through.
const SCALES: [f32; 7] = [0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];

// The look of the Horizon mods' own screens (Waypoints, Voice Chat).
const PANEL: u32 = 0xE010_1018;
const PANEL_EDGE: u32 = 0xFF50_5060;
const BUTTON: u32 = 0xFF30_3038;
const BUTTON_HOVER: u32 = 0xFF40_4058;
const BUTTON_ON: u32 = 0xFF2E_5E2E;
const BUTTON_OFF: u32 = 0xFF6A_2A2A;
const FIELD: u32 = 0xFF00_0000;
const FOCUS: u32 = 0xFFFF_FFFF;
const INK: u32 = 0xFF_FFFF;
const GREY: u32 = 0xAA_AAAA;
const DARK: u32 = 0x77_7777;
const GOLD: u32 = 0xFF_CC55;
const GREEN: u32 = 0x55_FF55;
const RED: u32 = 0xFF_5555;

/// Where the box goes on the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    TopCenter,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    /// Just under the crosshair.
    Crosshair,
}

impl Position {
    const ALL: [Position; 6] = [
        Position::TopCenter,
        Position::TopLeft,
        Position::TopRight,
        Position::BottomLeft,
        Position::BottomRight,
        Position::Crosshair,
    ];
    fn key(self) -> &'static str {
        match self {
            Position::TopCenter => "top_center",
            Position::TopLeft => "top_left",
            Position::TopRight => "top_right",
            Position::BottomLeft => "bottom_left",
            Position::BottomRight => "bottom_right",
            Position::Crosshair => "crosshair",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Position::TopCenter => "Top center",
            Position::TopLeft => "Top left",
            Position::TopRight => "Top right",
            Position::BottomLeft => "Bottom left",
            Position::BottomRight => "Bottom right",
            Position::Crosshair => "Under crosshair",
        }
    }
    /// The HUD corner it stacks in with other mods' boxes, as an index into
    /// the game's `Corner::ALL` (top left, top right, bottom left, bottom right).
    pub fn corner(self) -> Option<usize> {
        match self {
            Position::TopLeft => Some(0),
            Position::TopRight => Some(1),
            Position::BottomLeft => Some(2),
            Position::BottomRight => Some(3),
            Position::TopCenter | Position::Crosshair => None,
        }
    }
}

/// What is drawn behind the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Background {
    /// The Horizon mods' panel: dark, with a 1 px grey frame.
    Panel,
    /// Vanilla's tooltip: a dark box with a coloured frame.
    Tooltip,
    /// A plain box in the background colour.
    Flat,
    None,
}

impl Background {
    const ALL: [Background; 4] =
        [Background::Panel, Background::Tooltip, Background::Flat, Background::None];
    fn key(self) -> &'static str {
        match self {
            Background::Panel => "panel",
            Background::Tooltip => "tooltip",
            Background::Flat => "flat",
            Background::None => "none",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Background::Panel => "Panel",
            Background::Tooltip => "Tooltip",
            Background::Flat => "Flat",
            Background::None => "None",
        }
    }
}

/// What the key does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Each press shows or hides the box.
    Toggle,
    /// The box shows only while the key is held.
    Hold,
}

impl Mode {
    fn key(self) -> &'static str {
        match self {
            Mode::Toggle => "toggle",
            Mode::Hold => "hold",
        }
    }
}

/// The on/off lines of the box, in the order the settings page lists them:
/// config key, settings label.
const FLAGS: [(&str, &str); 15] = [
    ("show_icon", "Icon"),
    ("show_name", "Name"),
    ("show_id", "Registry id"),
    ("show_tool", "Tool"),
    ("show_tool_tier", "Tool tier"),
    ("show_harvestable", "Harvestable"),
    ("show_growth", "Crop growth"),
    ("show_hardness", "Hardness"),
    ("show_states", "Block states"),
    ("show_coords", "Coordinates"),
    ("show_mod", "Mod name"),
    ("show_entities", "Entities"),
    ("show_health", "Entity health"),
    ("text_shadow", "Text shadow"),
    ("hide_when_debug", "Hide with F3"),
];

/// The colours: config key, what it colours. Text ignores the alpha.
const COLORS: [(&str, &str); 13] = [
    ("color_background", "box fill (alpha matters: E0 is nearly opaque)"),
    ("color_border_top", "panel frame; tooltip frame, top"),
    ("color_border_bottom", "tooltip frame, bottom"),
    ("color_name", "the block or mob's name"),
    ("color_label", "labels (\"Tool:\", \"Hardness:\")"),
    ("color_tool", "the tool's name"),
    ("color_value", "other values"),
    ("color_yes", "harvestable"),
    ("color_no", "not harvestable"),
    ("color_info", "id, block states, coordinates"),
    ("color_mod", "the mod name"),
    ("color_health", "entity health"),
    ("color_growth", "the crop growth bar"),
];

/// Every setting. Saved whole to `config/block_info.cfg`, comments and all.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub shown: bool,
    pub mode: Mode,
    pub position: Position,
    /// Moves the box from where `position` puts it, in GUI pixels.
    pub offset: [f32; 2],
    pub scale: f32,
    pub background: Background,
    /// Inside the box, around the text.
    pub padding: f32,
    /// Between lines.
    pub line_gap: f32,
    /// The box is never narrower.
    pub min_width: f32,
    pub icon: bool,
    pub name: bool,
    pub id: bool,
    pub tool: bool,
    pub tool_tier: bool,
    pub harvestable: bool,
    pub growth: bool,
    pub hardness: bool,
    pub states: bool,
    pub coords: bool,
    pub mod_name: bool,
    pub entities: bool,
    pub health: bool,
    pub shadow: bool,
    pub hide_when_debug: bool,
    /// ARGB, in `COLORS` order.
    pub colors: [u32; 13],
    /// Ids never shown (`minecraft:grass_block`), or a whole mod (`minecraft:*`).
    pub hidden: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shown: true,
            mode: Mode::Toggle,
            position: Position::TopCenter,
            offset: [0.0, 0.0],
            scale: 1.0,
            background: Background::Panel,
            padding: 4.0,
            line_gap: 2.0,
            min_width: 0.0,
            icon: true,
            name: true,
            id: false,
            tool: true,
            tool_tier: true,
            harvestable: false,
            growth: true,
            hardness: false,
            states: false,
            coords: false,
            mod_name: true,
            entities: true,
            health: true,
            shadow: true,
            hide_when_debug: true,
            // The Waypoints and Voice Chat screens': white names, grey
            // labels, gold tool, green mod (as a waypoint's dimension).
            colors: [
                PANEL,
                PANEL_EDGE,
                PANEL_EDGE,
                0xFF00_0000 | INK,
                0xFF00_0000 | GREY,
                0xFF00_0000 | GOLD,
                0xFF00_0000 | INK,
                0xFF00_0000 | GREEN,
                0xFF00_0000 | RED,
                0xFF00_0000 | DARK,
                0xFF00_0000 | GREEN,
                0xFF00_0000 | RED,
                0xFF00_0000 | GREEN,
            ],
            hidden: Vec::new(),
        }
    }
}

impl Config {
    fn flag(&mut self, key: &str) -> Option<&mut bool> {
        Some(match key {
            "shown" => &mut self.shown,
            "show_icon" => &mut self.icon,
            "show_name" => &mut self.name,
            "show_id" => &mut self.id,
            "show_tool" => &mut self.tool,
            "show_tool_tier" => &mut self.tool_tier,
            "show_harvestable" => &mut self.harvestable,
            "show_growth" => &mut self.growth,
            "show_hardness" => &mut self.hardness,
            "show_states" => &mut self.states,
            "show_coords" => &mut self.coords,
            "show_mod" => &mut self.mod_name,
            "show_entities" => &mut self.entities,
            "show_health" => &mut self.health,
            "text_shadow" => &mut self.shadow,
            "hide_when_debug" => &mut self.hide_when_debug,
            _ => return None,
        })
    }
    fn flag_value(&self, key: &str) -> bool {
        self.clone().flag(key).is_some_and(|v| *v)
    }
    fn color(&self, key: &str) -> u32 {
        COLORS
            .iter()
            .position(|c| c.0 == key)
            .map_or(0xFFFF_FFFF, |i| self.colors[i])
    }

    /// Sets `key` from its text, as the file and `/blockinfo set` write it.
    pub fn apply(&mut self, key: &str, value: &str) -> Result<(), String> {
        let value = value.trim();
        let bad = || format!("{key}: \"{value}\" is not a valid value");
        let number = |min: f32, max: f32| -> Result<f32, String> {
            let v: f32 = value.parse().map_err(|_| bad())?;
            (v.is_finite() && (min..=max).contains(&v))
                .then_some(v)
                .ok_or_else(|| format!("{key}: {value} is outside {min}..{max}"))
        };
        if let Some(flag) = self.flag(key) {
            *flag = parse_bool(value).ok_or_else(bad)?;
            return Ok(());
        }
        if let Some(i) = COLORS.iter().position(|c| c.0 == key) {
            self.colors[i] = parse_color(value).ok_or_else(bad)?;
            return Ok(());
        }
        match key {
            "mode" => {
                self.mode = match value.to_ascii_lowercase().as_str() {
                    "toggle" => Mode::Toggle,
                    "hold" => Mode::Hold,
                    _ => return Err(bad()),
                }
            }
            "position" => {
                let v = value.to_ascii_lowercase();
                self.position = *Position::ALL.iter().find(|p| p.key() == v).ok_or_else(bad)?;
            }
            "background" => {
                let v = value.to_ascii_lowercase();
                self.background =
                    *Background::ALL.iter().find(|b| b.key() == v).ok_or_else(bad)?;
            }
            "offset_x" => self.offset[0] = number(-4096.0, 4096.0)?,
            "offset_y" => self.offset[1] = number(-4096.0, 4096.0)?,
            "scale" => self.scale = number(0.25, 4.0)?,
            "padding" => self.padding = number(0.0, 32.0)?,
            "line_gap" => self.line_gap = number(0.0, 16.0)?,
            "min_width" => self.min_width = number(0.0, 1024.0)?,
            "hidden" => {
                self.hidden = value
                    .split(',')
                    .map(|s| s.trim().to_ascii_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            _ => return Err(format!("{key}: no such setting")),
        }
        Ok(())
    }

    /// Reads a config file's text: every line it understands, and a note
    /// for each it doesn't (those keep their default).
    pub fn parse(text: &str) -> (Self, Vec<String>) {
        let mut config = Self::default();
        let mut notes = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                notes.push(format!("line {}: expected key = value", n + 1));
                continue;
            };
            // A comment after the value starts at a `#` after a space (a
            // colour's own `#` comes first).
            let value = value.trim();
            let value = value.find(" #").map_or(value, |end| &value[..end]);
            if let Err(e) = config.apply(&key.trim().to_ascii_lowercase(), value) {
                notes.push(format!("line {}: {e}", n + 1));
            }
        }
        (config, notes)
    }

    /// The config file, every setting with what it does.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let mut line = |s: String| {
            out.push_str(&s);
            out.push('\n');
        };
        line(format!("# {NAME} {VERSION}: what the crosshair is on."));
        line("# Saved by the settings page in Options > Mods; edit it by hand, then".into());
        line("# `/blockinfo reload`, or change one value with `/blockinfo set <key> <value>`.".into());
        line(String::new());
        line("# Whether the box shows (the toggle key flips this in toggle mode).".into());
        line(format!("shown = {}", self.shown));
        line("# toggle: each key press shows or hides it. hold: only while the key is held.".into());
        line(format!("mode = {}", self.mode.key()));
        line(String::new());
        line("# top_center, top_left, top_right, bottom_left, bottom_right or crosshair.".into());
        line(format!("position = {}", self.position.key()));
        line("# Moves the box from there, in GUI pixels (negative is left / up).".into());
        line(format!("offset_x = {}", self.offset[0]));
        line(format!("offset_y = {}", self.offset[1]));
        line("# 0.25 to 4.".into());
        line(format!("scale = {}", self.scale));
        line("# panel (the Horizon mods' screens), tooltip (vanilla's frame), flat or none.".into());
        line(format!("background = {}", self.background.key()));
        line(format!("padding = {}", self.padding));
        line(format!("line_gap = {}", self.line_gap));
        line(format!("min_width = {}", self.min_width));
        line(String::new());
        line("# What the box holds: true or false.".into());
        for (key, label) in FLAGS {
            line(format!("{key} = {}   # {label}", self.flag_value(key)));
        }
        line(String::new());
        line("# Colours: #RRGGBB or #AARRGGBB.".into());
        for (i, (key, what)) in COLORS.iter().enumerate() {
            line(format!("{key} = #{:08X}   # {what}", self.colors[i]));
        }
        line(String::new());
        line("# Never shown, comma separated: minecraft:grass_block, minecraft:cow, or a".into());
        line("# whole mod with mymod:*".into());
        line(format!("hidden = {}", self.hidden.join(", ")));
        out
    }

    /// Whether `id` is on the hidden list.
    pub fn hides(&self, id: &str) -> bool {
        let id = id.to_ascii_lowercase();
        self.hidden.iter().any(|h| match h.strip_suffix('*') {
            Some(prefix) => id.starts_with(prefix),
            None => *h == id,
        })
    }

    /// The settings page's buttons.
    pub fn labels(&self) -> Vec<String> {
        let mut labels = vec![
            format!("Shown: {}", on_off(self.shown)),
            format!(
                "Key: {}",
                if self.mode == Mode::Toggle { "Toggles" } else { "Hold to show" }
            ),
            format!("Position: {}", self.position.label()),
            format!("Scale: {}x", self.scale),
            format!("Background: {}", self.background.label()),
        ];
        for (key, label) in FLAGS {
            labels.push(format!("{label}: {}", on_off(self.flag_value(key))));
        }
        labels.push("Reload config file".into());
        labels.push("Reset to defaults".into());
        labels
    }

    /// A click on button `i` of `labels`.
    pub fn click(&mut self, i: usize) -> Click {
        match i {
            0 => self.shown = !self.shown,
            1 => {
                self.mode = match self.mode {
                    Mode::Toggle => Mode::Hold,
                    Mode::Hold => Mode::Toggle,
                }
            }
            2 => self.position = next(&Position::ALL, self.position),
            3 => {
                self.scale = SCALES
                    .iter()
                    .copied()
                    .find(|s| *s > self.scale + 0.01)
                    .unwrap_or(SCALES[0])
            }
            4 => self.background = next(&Background::ALL, self.background),
            i if i - 5 < FLAGS.len() => {
                if let Some(flag) = self.flag(FLAGS[i - 5].0) {
                    *flag = !*flag;
                }
            }
            i if i == 5 + FLAGS.len() => return Click::Reload,
            i if i == 6 + FLAGS.len() => {
                *self = Self::default();
            }
            _ => return Click::Nothing,
        }
        Click::Changed
    }
}

/// What a settings click asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    Changed,
    Reload,
    Nothing,
}

fn next<T: Copy + PartialEq>(all: &[T], now: T) -> T {
    let i = all.iter().position(|v| *v == now).unwrap_or(0);
    all[(i + 1) % all.len()]
}

fn on_off(on: bool) -> &'static str {
    if on { "ON" } else { "OFF" }
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.to_ascii_lowercase().as_str() {
        "true" | "on" | "yes" | "1" => Some(true),
        "false" | "off" | "no" | "0" => Some(false),
        _ => None,
    }
}

/// `#RRGGBB` (opaque), `#AARRGGBB`, or the same with `0x`.
fn parse_color(v: &str) -> Option<u32> {
    let hex = v.strip_prefix('#').or_else(|| v.strip_prefix("0x"))?;
    let n = u32::from_str_radix(hex, 16).ok()?;
    match hex.len() {
        6 => Some(0xFF00_0000 | n),
        8 => Some(n),
        _ => None,
    }
}

/// What the crosshair is on, as the hook adapter read it from the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Target {
    /// `minecraft:grass_block`, `minecraft:cow`.
    pub id: String,
    /// In the game's language: `Grass Block`.
    pub name: String,
    /// The item drawn beside it, if the game has one.
    pub icon: Option<String>,
    /// A block's tool kind and the tier it needs: `("Shovel", None)`,
    /// `("Pickaxe", Some("Iron"))`.
    pub tool: Option<(String, Option<String>)>,
    /// Whether the held item gets its drops.
    pub harvestable: Option<bool>,
    /// A block's hardness (-1 for unbreakable).
    pub hardness: Option<f32>,
    pub states: Vec<(String, String)>,
    pub pos: Option<[i32; 3]>,
    /// A mob's health and maximum.
    pub health: Option<[f32; 2]>,
    /// How far a crop or sapling has grown, 0 to 1.
    pub growth: Option<f32>,
}

/// The tool a block wants, from whether it is in a block tag: its kind, and
/// the tier it needs for its drops.
pub fn tool(in_tag: impl Fn(&str) -> bool) -> Option<(String, Option<String>)> {
    const KINDS: [(&str, &str); 6] = [
        ("minecraft:mineable/pickaxe", "Pickaxe"),
        ("minecraft:mineable/axe", "Axe"),
        ("minecraft:mineable/shovel", "Shovel"),
        ("minecraft:mineable/hoe", "Hoe"),
        ("minecraft:sword_efficient", "Sword"),
        ("minecraft:leaves", "Shears"),
    ];
    const TIERS: [(&str, &str); 3] = [
        ("minecraft:needs_diamond_tool", "Diamond"),
        ("minecraft:needs_iron_tool", "Iron"),
        ("minecraft:needs_stone_tool", "Stone"),
    ];
    let kind = KINDS.iter().find(|k| in_tag(k.0))?.1;
    let tier = TIERS.iter().find(|t| in_tag(t.0)).map(|t| t.1.to_owned());
    Some((kind.to_owned(), tier))
}

/// How far a crop has grown, 0 to 1, from its id and block states; `crop`
/// says the block is in `minecraft:crops` (a mod's crop with an `age` is read
/// as wheat's 0 to 7). None for blocks that don't grow.
pub fn growth(id: &str, states: &[(String, String)], crop: bool) -> Option<f32> {
    // The last age of each of vanilla's growing blocks.
    const MAX_AGE: [(&str, f32); 11] = [
        ("minecraft:wheat", 7.0),
        ("minecraft:carrots", 7.0),
        ("minecraft:potatoes", 7.0),
        ("minecraft:melon_stem", 7.0),
        ("minecraft:pumpkin_stem", 7.0),
        ("minecraft:beetroots", 3.0),
        ("minecraft:nether_wart", 3.0),
        ("minecraft:sweet_berry_bush", 3.0),
        ("minecraft:cocoa", 2.0),
        ("minecraft:torchflower_crop", 2.0),
        ("minecraft:pitcher_crop", 4.0),
    ];
    let state = |key: &str| -> Option<f32> {
        states.iter().find(|(k, _)| k == key)?.1.parse().ok()
    };
    if id.starts_with("minecraft:attached_") && id.ends_with("_stem") {
        return Some(1.0);
    }
    if id.ends_with("_sapling") {
        return state("stage").map(|s| s.clamp(0.0, 1.0));
    }
    let max = MAX_AGE
        .iter()
        .find(|(name, _)| *name == id)
        .map(|(_, max)| *max)
        .or(crop.then_some(7.0))?;
    state("age").map(|age| (age / max).clamp(0.0, 1.0))
}

/// `minecraft:grass_block` as `Grass Block`, when the language has no name.
pub fn pretty(id: &str) -> String {
    let path = id.rsplit(':').next().unwrap_or(id);
    let path = path.rsplit('/').next().unwrap_or(path);
    title_case(path)
}

/// The mod an id comes from: `minecraft` as `Minecraft`, `create` as `Create`.
pub fn mod_name(id: &str) -> String {
    match id.split_once(':') {
        Some((ns, _)) => title_case(ns),
        None => "Minecraft".into(),
    }
}

/// A key as the screens name it: `key.keyboard.caps.lock` as `Caps Lock`.
pub fn key_label(key: &str) -> String {
    let name = key
        .strip_prefix("key.keyboard.")
        .or_else(|| key.strip_prefix("key.mouse."))
        .unwrap_or(key);
    if name == "unknown" {
        return "not bound".into();
    }
    title_case(&name.replace('.', "_"))
}

fn title_case(s: &str) -> String {
    s.split(['_', '-'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().chain(c).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A drawing in GUI pixels: a rectangle (ARGB) or text (RGB, shadow).
#[derive(Clone, Debug, PartialEq)]
pub enum Draw {
    Fill([f32; 4], u32),
    Text(String, [f32; 2], u32, bool),
}

/// The box laid out with its top left at 0, 0, unscaled.
#[derive(Clone, Debug, PartialEq)]
pub struct Panel {
    pub size: [f32; 2],
    pub draws: Vec<Draw>,
    /// The icon's item and top left.
    pub icon: Option<(String, [f32; 2])>,
}

/// One line of the box: pieces of text, each in its colour, or a bar
/// (filled 0 to 1) as wide as the text.
enum Line {
    Text(Vec<(String, u32)>),
    Bar(f32, u32),
}

/// A bar's height and the room it takes.
const BAR: f32 = 5.0;

fn lines(cfg: &Config, t: &Target) -> Vec<Line> {
    let c = |key: &str| cfg.color(key);
    let mut out: Vec<Line> = Vec::new();
    let labelled = |out: &mut Vec<Line>, label: &str, value: String, color: u32| {
        out.push(Line::Text(vec![(format!("{label}: "), c("color_label")), (value, color)]));
    };
    if cfg.tool
        && let Some((kind, tier)) = &t.tool
    {
        let name = match tier {
            Some(tier) if cfg.tool_tier => format!("{tier} {kind}"),
            _ => kind.clone(),
        };
        labelled(&mut out, "Tool", name, c("color_tool"));
    }
    if cfg.harvestable
        && let Some(yes) = t.harvestable
    {
        let (text, color) = if yes { ("Yes", "color_yes") } else { ("No", "color_no") };
        labelled(&mut out, "Harvestable", text.into(), c(color));
    }
    if cfg.growth
        && let Some(g) = t.growth
    {
        if g >= 1.0 {
            labelled(&mut out, "Growth", "Mature".into(), c("color_yes"));
        } else {
            let percent = format!("{}%", (g * 100.0).floor());
            labelled(&mut out, "Growth", percent, c("color_value"));
        }
        out.push(Line::Bar(g, c("color_growth")));
    }
    if cfg.hardness
        && let Some(h) = t.hardness
    {
        let text = if h < 0.0 {
            "Unbreakable".into()
        } else if h == 0.0 {
            "Instant".into()
        } else {
            format!("{}", (h * 100.0).round() / 100.0)
        };
        labelled(&mut out, "Hardness", text, c("color_value"));
    }
    if cfg.health
        && let Some([hp, max]) = t.health
    {
        let half = |v: f32| (v * 2.0).ceil() / 2.0;
        labelled(&mut out, "Health", format!("{} / {}", half(hp), half(max)), c("color_health"));
    }
    if cfg.states {
        for (k, v) in &t.states {
            out.push(Line::Text(vec![(format!("{k}: {v}"), c("color_info"))]));
        }
    }
    if cfg.coords
        && let Some([x, y, z]) = t.pos
    {
        out.push(Line::Text(vec![(format!("{x} {y} {z}"), c("color_info"))]));
    }
    let mut head: Vec<Line> = Vec::new();
    if cfg.name {
        head.push(Line::Text(vec![(t.name.clone(), c("color_name"))]));
    }
    if cfg.id {
        head.push(Line::Text(vec![(t.id.clone(), c("color_info"))]));
    }
    head.append(&mut out);
    if cfg.mod_name {
        head.push(Line::Text(vec![(mod_name(&t.id), c("color_mod"))]));
    }
    head
}

/// Lays the box out: its lines, its background, its icon.
pub fn panel(cfg: &Config, t: &Target, measure: &dyn Fn(&str) -> f32) -> Option<Panel> {
    let lines = lines(cfg, t);
    let icon = t.icon.clone().filter(|_| cfg.icon);
    if lines.is_empty() && icon.is_none() {
        return None;
    }
    let pad = cfg.padding;
    let line_h = FONT_HEIGHT + cfg.line_gap;
    let height = |l: &Line| match l {
        Line::Text(_) => line_h,
        Line::Bar(..) => BAR + cfg.line_gap,
    };
    let text_h = (lines.iter().map(height).sum::<f32>() - cfg.line_gap).max(0.0);
    let text_w = lines
        .iter()
        .map(|l| match l {
            Line::Text(parts) => parts.iter().map(|(s, _)| measure(s)).sum::<f32>(),
            // A bar alone is never thinner than this.
            Line::Bar(..) => 60.0,
        })
        .fold(0.0, f32::max);
    let text_x = pad + if icon.is_some() { ICON + ICON_GAP } else { 0.0 };
    let inner_h = text_h.max(if icon.is_some() { ICON } else { 0.0 });
    let w = (text_x + text_w + pad).max(cfg.min_width).ceil();
    let h = (inner_h + 2.0 * pad).ceil();

    let mut draws = background(cfg, w, h);
    let mut y = pad + ((inner_h - text_h) / 2.0).floor();
    for line in &lines {
        match line {
            Line::Text(parts) => {
                let mut x = text_x;
                for (text, color) in parts {
                    draws.push(Draw::Text(text.clone(), [x, y], color & 0xFF_FFFF, cfg.shadow));
                    x += measure(text);
                }
            }
            // As Voice Chat's level meter: black, the filled part inside.
            Line::Bar(fraction, color) => {
                let bar_w = w - text_x - pad;
                draws.push(Draw::Fill([text_x, y, bar_w, BAR], FIELD));
                let filled = ((bar_w - 2.0) * fraction.clamp(0.0, 1.0)).round();
                draws.push(Draw::Fill([text_x + 1.0, y + 1.0, filled, BAR - 2.0], *color));
            }
        }
        y += height(line);
    }
    let icon = icon.map(|item| (item, [pad, pad + ((inner_h - ICON) / 2.0).floor()]));
    Some(Panel { size: [w, h], draws, icon })
}

fn background(cfg: &Config, w: f32, h: f32) -> Vec<Draw> {
    let fill = cfg.color("color_background");
    match cfg.background {
        Background::None => Vec::new(),
        Background::Flat => vec![Draw::Fill([0.0, 0.0, w, h], fill)],
        Background::Panel => vec![
            Draw::Fill([0.0, 0.0, w, h], cfg.color("color_border_top")),
            Draw::Fill([1.0, 1.0, w - 2.0, h - 2.0], fill),
        ],
        Background::Tooltip => {
            // TooltipRenderUtil: the fill with its corners cut, then a 1 px
            // frame inside it, its sides fading from the top colour down.
            let (top, bottom) = (cfg.color("color_border_top"), cfg.color("color_border_bottom"));
            let half = ((h - 4.0) / 2.0).max(0.0);
            vec![
                Draw::Fill([1.0, 0.0, w - 2.0, h], fill),
                Draw::Fill([0.0, 1.0, 1.0, h - 2.0], fill),
                Draw::Fill([w - 1.0, 1.0, 1.0, h - 2.0], fill),
                Draw::Fill([1.0, 1.0, w - 2.0, 1.0], top),
                Draw::Fill([1.0, h - 2.0, w - 2.0, 1.0], bottom),
                Draw::Fill([1.0, 2.0, 1.0, half], top),
                Draw::Fill([w - 2.0, 2.0, 1.0, half], top),
                Draw::Fill([1.0, 2.0 + half, 1.0, h - 4.0 - half], bottom),
                Draw::Fill([w - 2.0, 2.0 + half, 1.0, h - 4.0 - half], bottom),
            ]
        }
    }
}

/// The box's top left on a `screen`-sized HUD for the positions that are not
/// a corner, `size` already scaled; kept on the screen.
pub fn place(cfg: &Config, size: [f32; 2], screen: [f32; 2]) -> [f32; 2] {
    let [w, h] = size;
    let [sw, sh] = screen;
    let [x, y] = match cfg.position {
        Position::Crosshair => [(sw - w) / 2.0, sh / 2.0 + 10.0],
        // Every other position the adapter gets from the game's corners;
        // these are only their fallback.
        Position::TopLeft => [2.0, 2.0],
        Position::TopRight => [sw - w - 2.0, 2.0],
        Position::BottomLeft => [2.0, sh - h - 2.0],
        Position::BottomRight => [sw - w - 2.0, sh - h - 2.0],
        Position::TopCenter => [(sw - w) / 2.0, 4.0],
    };
    clamp([x + cfg.offset[0], y + cfg.offset[1]], size, screen)
}

/// `at` moved so a `size` box stays on the screen.
pub fn clamp(at: [f32; 2], size: [f32; 2], screen: [f32; 2]) -> [f32; 2] {
    [0, 1].map(|i| at[i].min(screen[i] - size[i]).max(0.0).round())
}

/// What `/blockinfo ...` (without its slash) did: lines for chat, and
/// whether the config changed (to save) or must be read again.
#[derive(Debug, Default, PartialEq)]
pub struct Ran {
    pub chat: Vec<String>,
    /// Open the settings screen.
    pub open: bool,
    pub changed: bool,
    pub reload: bool,
}

/// Runs `command` when it is the mod's; None for every other command.
pub fn run(cfg: &mut Config, command: &str) -> Option<Ran> {
    let mut words = command.split_whitespace();
    if !words.next()?.eq_ignore_ascii_case(COMMAND) {
        return None;
    }
    let mut ran = Ran::default();
    let arg = words.next().map(str::to_ascii_lowercase);
    match arg.as_deref() {
        None | Some("toggle") => {
            cfg.shown = !cfg.shown;
            ran.changed = true;
            ran.chat.push(format!("{NAME}: {}", on_off(cfg.shown)));
        }
        Some("on" | "off") => {
            cfg.shown = arg.as_deref() == Some("on");
            ran.changed = true;
            ran.chat.push(format!("{NAME}: {}", on_off(cfg.shown)));
        }
        Some("reload") => ran.reload = true,
        Some("menu" | "settings" | "config") => ran.open = true,
        Some("reset") => {
            *cfg = Config::default();
            ran.changed = true;
            ran.chat.push(format!("{NAME}: back to the defaults"));
        }
        Some("set") => {
            let key = words.next().unwrap_or_default().to_ascii_lowercase();
            let value = words.collect::<Vec<_>>().join(" ");
            match cfg.apply(&key, &value) {
                Ok(()) => {
                    ran.changed = true;
                    ran.chat.push(format!("{NAME}: {key} = {value}"));
                }
                Err(e) => ran.chat.push(format!("{NAME}: {e}")),
            }
        }
        _ => {
            ran.chat.push(format!(
                "/{COMMAND} [on|off|toggle] | menu | reload | reset | set <key> <value>"
            ));
            ran.chat.push(format!("Keys are those of config/{CONFIG_FILE}."));
        }
    }
    Some(ran)
}

/// The mod's state on the window's thread.
#[derive(Default)]
pub struct State {
    config: Option<Config>,
    path: Option<PathBuf>,
    /// The key is down (for hold mode).
    pub held: bool,
    /// Lines for chat the next time a hook can write there.
    notes: Vec<String>,
    /// The settings screen is open, asked to open or to close.
    pub menu_open: bool,
    open_requested: bool,
    close_requested: bool,
    /// Reset was clicked once and asks to be clicked again.
    confirm_reset: bool,
    /// The screen's buttons as drawn last.
    widgets: Vec<Widget>,
}

impl State {
    /// The config, read from `path` the first time (and written there with
    /// the defaults when it is missing).
    pub fn config(&mut self, path: Option<PathBuf>) -> &mut Config {
        if self.config.is_none() {
            self.path = path;
            self.load();
        }
        self.config.get_or_insert_with(Config::default)
    }
    fn load(&mut self) {
        let text = self.path.as_ref().and_then(|p| fs::read_to_string(p).ok());
        let config = match text {
            Some(text) => {
                let (config, notes) = Config::parse(&text);
                self.notes
                    .extend(notes.into_iter().map(|n| format!("{NAME} config, {n}")));
                config
            }
            None => {
                let config = Config::default();
                self.config = Some(config.clone());
                self.save();
                config
            }
        };
        self.config = Some(config);
    }
    /// Reads the file again.
    pub fn reload(&mut self) {
        self.load();
        self.notes.push(format!("{NAME}: config reloaded"));
    }
    /// Writes the config to its file.
    pub fn save(&mut self) {
        let (Some(path), Some(config)) = (&self.path, &self.config) else {
            return;
        };
        let written = path
            .parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|()| fs::write(path, config.to_text()));
        if let Err(e) = written {
            self.notes.push(format!("{NAME}: can't save {}: {e}", path.display()));
        }
    }
    /// Whether the box shows now.
    pub fn visible(&self) -> bool {
        self.config
            .as_ref()
            .is_some_and(|c| c.shown && (c.mode == Mode::Toggle || self.held))
    }
    pub fn take_notes(&mut self) -> Vec<String> {
        std::mem::take(&mut self.notes)
    }
}

// ---- The settings screen ---------------------------------------------------

/// What a click on the settings screen does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Close,
    Shown,
    Mode,
    Position,
    Background,
    Scale(f32),
    OffsetX(f32),
    OffsetY(f32),
    Padding(f32),
    LineGap(f32),
    Flag(usize),
    Reload,
    Reset,
}

/// A clickable rectangle `[x, y, w, h]` of the screen drawn this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Widget {
    pub rect: [f32; 4],
    pub action: Action,
}

struct Out<'a> {
    draws: Vec<Draw>,
    widgets: Vec<Widget>,
    mouse: [f32; 2],
    measure: &'a dyn Fn(&str) -> f32,
}

fn inside(rect: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= rect[0] && at[0] < rect[0] + rect[2] && at[1] >= rect[1] && at[1] < rect[1] + rect[3]
}

impl Out<'_> {
    fn fill(&mut self, rect: [f32; 4], argb: u32) {
        self.draws.push(Draw::Fill(rect, argb));
    }
    fn text(&mut self, text: &str, at: [f32; 2], rgb: u32) {
        self.draws.push(Draw::Text(text.to_owned(), at, rgb, true));
    }
    fn width(&self, text: &str) -> f32 {
        (self.measure)(text)
    }
    /// A 1 pixel frame round `rect`, filled with `inner`.
    fn boxed(&mut self, rect: [f32; 4], edge: u32, inner: u32) {
        let [x, y, w, h] = rect;
        self.fill(rect, edge);
        self.fill([x + 1.0, y + 1.0, w - 2.0, h - 2.0], inner);
    }
    /// `text` cut to `max` pixels with an ellipsis.
    fn fit(&self, text: &str, max: f32) -> String {
        if self.width(text) <= max {
            return text.to_owned();
        }
        let mut cut = text.to_owned();
        while !cut.is_empty() && self.width(&format!("{cut}...")) > max {
            cut.pop();
        }
        format!("{cut}...")
    }
    /// A button: green when `state` is on, red when off, grey without one.
    fn button(&mut self, rect: [f32; 4], label: &str, action: Action, state: Option<bool>) {
        let hover = inside(rect, self.mouse);
        let fill = match state {
            _ if hover => BUTTON_HOVER,
            Some(true) => BUTTON_ON,
            Some(false) => BUTTON_OFF,
            None => BUTTON,
        };
        self.boxed(rect, if hover { FOCUS } else { PANEL_EDGE }, fill);
        let label = self.fit(label, rect[2] - 4.0);
        let w = self.width(&label);
        self.text(&label, [rect[0] + (rect[2] - w) / 2.0, rect[1] + (rect[3] - 8.0) / 2.0], INK);
        self.widgets.push(Widget { rect, action });
    }
    /// `label: [-] value [+]` across `w`.
    #[allow(clippy::too_many_arguments)]
    fn stepper(&mut self, x: f32, y: f32, w: f32, label: &str, value: &str, down: Action, up: Action) {
        self.text(label, [x, y + 2.0], GREY);
        let right = x + w;
        self.button([right - 12.0, y, 12.0, 12.0], "+", up, None);
        let vw = 44.0;
        let vx = right - 14.0 - vw;
        let tw = self.width(value);
        self.text(value, [vx + (vw - tw) / 2.0, y + 2.0], INK);
        self.button([vx - 14.0, y, 12.0, 12.0], "-", down, None);
    }
    /// The box as the HUD draws it, unscaled, with its top left at `at`;
    /// its icon goes to `icons`. Its height.
    fn preview(&mut self, panel: Panel, at: [f32; 2], icons: &mut Vec<(String, [f32; 2])>) -> f32 {
        let shift = |p: [f32; 2]| [at[0] + p[0], at[1] + p[1]];
        for draw in panel.draws {
            self.draws.push(match draw {
                Draw::Fill([x, y, w, h], argb) => Draw::Fill([at[0] + x, at[1] + y, w, h], argb),
                Draw::Text(t, p, rgb, shadow) => Draw::Text(t, shift(p), rgb, shadow),
            });
        }
        if let Some((item, p)) = panel.icon {
            icons.push((item, shift(p)));
        }
        panel.size[1]
    }
}

/// The settings screen's examples: a block with its tool, a growing crop.
fn samples() -> [Target; 2] {
    [
        Target {
            id: "minecraft:grass_block".into(),
            name: "Grass Block".into(),
            icon: Some("minecraft:grass_block".into()),
            tool: Some(("Shovel".into(), None)),
            harvestable: Some(true),
            hardness: Some(0.6),
            states: vec![("snowy".into(), "false".into())],
            pos: Some([1156, 64, 1102]),
            ..Default::default()
        },
        Target {
            id: "minecraft:wheat".into(),
            name: "Wheat Crops".into(),
            icon: Some("minecraft:wheat".into()),
            harvestable: Some(true),
            hardness: Some(0.0),
            states: vec![("age".into(), "4".into())],
            pos: Some([1158, 62, 1149]),
            growth: Some(4.0 / 7.0),
            ..Default::default()
        },
    ]
}

impl State {
    fn cfg(&mut self) -> &mut Config {
        self.config.get_or_insert_with(Config::default)
    }

    /// The screen is open now.
    pub fn open_menu(&mut self) {
        self.menu_open = true;
        self.confirm_reset = false;
    }
    /// `/blockinfo menu` asks for the screen, opened once chat has closed.
    pub fn request_open(&mut self) {
        self.open_requested = true;
    }
    pub fn take_open(&mut self) -> bool {
        std::mem::take(&mut self.open_requested)
    }
    fn close_menu(&mut self) {
        self.menu_open = false;
        self.close_requested = true;
    }
    /// Whether the game should close the screen now (once).
    pub fn take_close(&mut self) -> bool {
        std::mem::take(&mut self.close_requested)
    }
    /// The game's screen is (`ours`) or isn't the mod's: forget a screen the
    /// game closed (Escape, another screen).
    pub fn sync_open(&mut self, ours: bool) {
        if !ours {
            self.menu_open = false;
        }
    }

    /// A key over the screen: Escape or the menu key close it.
    pub fn menu_key(&mut self, key: &str, menu_key: &str) {
        if key == "key.keyboard.escape" || key == menu_key {
            self.close_menu();
        }
    }

    /// A left click at `at` on the screen drawn last.
    pub fn menu_click(&mut self, at: [f32; 2]) {
        let hit = self.widgets.iter().rev().find(|w| inside(w.rect, at)).map(|w| w.action);
        if let Some(action) = hit {
            self.act(action);
        }
    }

    /// Does what a button says, and saves.
    pub fn act(&mut self, action: Action) {
        let reset = std::mem::take(&mut self.confirm_reset);
        let step = |v: &mut f32, by: f32, min: f32, max: f32| {
            *v = ((*v + by) * 100.0).round() / 100.0;
            *v = v.clamp(min, max);
        };
        let cfg = self.cfg();
        match action {
            Action::Close => return self.close_menu(),
            Action::Reload => return self.reload(),
            Action::Reset if !reset => {
                self.confirm_reset = true;
                return;
            }
            Action::Reset => *cfg = Config::default(),
            Action::Shown => cfg.shown = !cfg.shown,
            Action::Mode => {
                cfg.click(1);
            }
            Action::Position => cfg.position = next(&Position::ALL, cfg.position),
            Action::Background => cfg.background = next(&Background::ALL, cfg.background),
            Action::Scale(by) => step(&mut cfg.scale, by, 0.25, 4.0),
            Action::OffsetX(by) => step(&mut cfg.offset[0], by, -4096.0, 4096.0),
            Action::OffsetY(by) => step(&mut cfg.offset[1], by, -4096.0, 4096.0),
            Action::Padding(by) => step(&mut cfg.padding, by, 0.0, 32.0),
            Action::LineGap(by) => step(&mut cfg.line_gap, by, 0.0, 16.0),
            Action::Flag(i) => {
                if let Some(flag) = FLAGS.get(i).and_then(|f| cfg.flag(f.0)) {
                    *flag = !*flag;
                }
            }
        }
        self.save();
    }

    /// Draws the settings screen on a `size` GUI, the pointer at `mouse`:
    /// what to draw, and the item icons of the preview (item, top left).
    /// `hotkeys` are what each key does and its name.
    pub fn menu(
        &mut self,
        size: [f32; 2],
        mouse: [f32; 2],
        measure: &dyn Fn(&str) -> f32,
        hotkeys: &[(String, String)],
    ) -> (Vec<Draw>, Vec<(String, [f32; 2])>) {
        let cfg = self.cfg().clone();
        let mut out = Out { draws: Vec::new(), widgets: Vec::new(), mouse, measure };
        let mut icons = Vec::new();
        let w = (size[0] - 16.0).min(440.0);
        let h = (size[1] - 16.0).min(260.0);
        let (x0, y0) = (((size[0] - w) / 2.0).round(), ((size[1] - h) / 2.0).round());
        out.fill([0.0, 0.0, size[0], size[1]], 0x8000_0000);
        out.boxed([x0, y0, w, h], PANEL_EDGE, PANEL);
        out.text(NAME, [x0 + 6.0, y0 + 5.0], GOLD);
        let (status, rgb) = match (cfg.shown, cfg.mode) {
            (false, _) => ("Hidden", RED),
            (true, Mode::Hold) => ("Shown while the key is held", GREEN),
            (true, Mode::Toggle) => ("Shown", GREEN),
        };
        out.text(status, [x0 + out.width(NAME) + 12.0, y0 + 5.0], rgb);
        out.button([x0 + w - 16.0, y0 + 3.0, 12.0, 12.0], "x", Action::Close, None);

        let col = ((w - 18.0) / 2.0).floor();
        let (lx, rx, top) = (x0 + 6.0, x0 + 12.0 + col, y0 + 20.0);
        out.fill([x0 + 9.0 + col, top, 1.0, h - 26.0], PANEL_EDGE);

        // Left: where and how the box shows, and the keys.
        let mut y = top;
        let shown = format!("Shown: {}", if cfg.shown { "On" } else { "Off" });
        out.button([lx, y, col, 12.0], &shown, Action::Shown, Some(cfg.shown));
        y += 14.0;
        let mode = if cfg.mode == Mode::Toggle { "Key toggles" } else { "Hold key to show" };
        out.button([lx, y, col, 12.0], &format!("Activation: {mode}"), Action::Mode, None);
        y += 14.0;
        let position = format!("Position: {}", cfg.position.label());
        out.button([lx, y, col, 12.0], &position, Action::Position, None);
        y += 14.0;
        let background = format!("Background: {}", cfg.background.label());
        out.button([lx, y, col, 12.0], &background, Action::Background, None);
        y += 16.0;
        let steppers = [
            ("Scale", format!("{}x", cfg.scale), Action::Scale(-0.25), Action::Scale(0.25)),
            ("Offset X", format!("{}", cfg.offset[0]), Action::OffsetX(-4.0), Action::OffsetX(4.0)),
            ("Offset Y", format!("{}", cfg.offset[1]), Action::OffsetY(-4.0), Action::OffsetY(4.0)),
            ("Padding", format!("{}", cfg.padding), Action::Padding(-1.0), Action::Padding(1.0)),
            ("Line spacing", format!("{}", cfg.line_gap), Action::LineGap(-1.0), Action::LineGap(1.0)),
        ];
        for (label, value, down, up) in steppers {
            out.stepper(lx, y, col, label, &value, down, up);
            y += 14.0;
        }
        y += 4.0;
        out.text("Hotkeys", [lx, y], GOLD);
        y += 11.0;
        for (what, key) in hotkeys {
            out.text(&out.fit(what, col * 0.6), [lx + 2.0, y], GREY);
            let kw = out.width(key);
            out.text(key, [lx + col - kw, y], if key == "not bound" { DARK } else { INK });
            y += 10.0;
        }
        let hint = out.fit(&format!("Change keys in Options > Mods > {NAME}"), col);
        out.text(&hint, [lx, y + 1.0], DARK);
        y += 12.0;
        out.text(&out.fit(&format!("Saved to config/{CONFIG_FILE}"), col), [lx, y], DARK);
        let by = y0 + h - 18.0;
        let half = ((col - 3.0) / 2.0).floor();
        out.button([lx, by, half, 12.0], "Reload file", Action::Reload, None);
        let reset = if self.confirm_reset { "Sure? Click again" } else { "Reset to defaults" };
        out.button([lx + half + 3.0, by, col - half - 3.0, 12.0], reset, Action::Reset, None);

        // Right: every line on or off, and the box as it looks now.
        let mut y = top;
        out.text("Lines", [rx, y + 2.0], GOLD);
        y += 13.0;
        let bw = ((col - 3.0) / 2.0).floor();
        for (i, (key, label)) in FLAGS.iter().enumerate() {
            let on = cfg.flag_value(key);
            let bx = if i % 2 == 0 { rx } else { rx + bw + 3.0 };
            out.button([bx, y, bw, 12.0], label, Action::Flag(i), Some(on));
            if i % 2 == 1 {
                y += 14.0;
            }
        }
        y += if FLAGS.len() % 2 == 1 { 18.0 } else { 4.0 };
        out.text("Preview", [rx, y], GOLD);
        y += 12.0;
        let bottom = y0 + h - 6.0;
        for sample in samples() {
            let Some(panel) = panel(&cfg, &sample, measure) else {
                continue;
            };
            if y + panel.size[1] > bottom {
                break;
            }
            y += out.preview(panel, [rx, y], &mut icons) + 4.0;
        }
        self.widgets = out.widgets;
        (out.draws, icons)
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// Runs `f` with the mod's state.
pub fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measure(s: &str) -> f32 {
        s.chars().count() as f32 * 6.0
    }

    fn grass() -> Target {
        Target {
            id: "minecraft:grass_block".into(),
            name: "Grass Block".into(),
            icon: Some("minecraft:grass_block".into()),
            tool: Some(("Shovel".into(), None)),
            harvestable: Some(true),
            hardness: Some(0.6),
            states: vec![("snowy".into(), "false".into())],
            pos: Some([1, 64, -3]),
            health: None,
            growth: None,
        }
    }

    fn texts(p: &Panel) -> Vec<String> {
        p.draws
            .iter()
            .filter_map(|d| match d {
                Draw::Text(t, ..) => Some(t.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn defaults_look_like_the_screenshot() {
        let p = panel(&Config::default(), &grass(), &measure).unwrap();
        assert_eq!(texts(&p), ["Grass Block", "Tool: ", "Shovel", "Minecraft"]);
        // The Horizon mods' panel: a grey frame round the dark fill.
        assert_eq!(p.draws[0], Draw::Fill([0.0, 0.0, p.size[0], p.size[1]], PANEL_EDGE));
        assert!(p.icon.is_some());
        // The tool's name follows its label, in yellow.
        let tool = p.draws.iter().find_map(|d| match d {
            Draw::Text(t, at, rgb, _) if t == "Shovel" => Some((at[0], *rgb)),
            _ => None,
        });
        assert_eq!(tool, Some((4.0 + 20.0 + measure("Tool: "), GOLD)));
    }

    #[test]
    fn every_line_can_be_turned_on_and_off() {
        let mut cfg = Config::default();
        for key in ["show_id", "show_harvestable", "show_hardness", "show_states", "show_coords"] {
            cfg.apply(key, "true").unwrap();
        }
        let p = panel(&cfg, &grass(), &measure).unwrap();
        let all = texts(&p).join("|");
        for part in ["minecraft:grass_block", "Yes", "0.6", "snowy: false", "1 64 -3"] {
            assert!(all.contains(part), "{part} missing from {all}");
        }
        for (key, _) in FLAGS.iter().take(11) {
            cfg.apply(key, "false").unwrap();
        }
        assert!(panel(&cfg, &grass(), &measure).is_none());
    }

    #[test]
    fn the_tier_joins_the_tool() {
        let t = tool(|tag| tag == "minecraft:mineable/pickaxe" || tag == "minecraft:needs_iron_tool");
        let target = Target { tool: t, ..grass() };
        let p = panel(&Config::default(), &target, &measure).unwrap();
        assert!(texts(&p).contains(&"Iron Pickaxe".to_string()));
        let mut cfg = Config::default();
        cfg.tool_tier = false;
        assert!(texts(&panel(&cfg, &target, &measure).unwrap()).contains(&"Pickaxe".to_string()));
        assert_eq!(tool(|_| false), None);
    }

    #[test]
    fn the_file_round_trips() {
        let mut cfg = Config::default();
        cfg.position = Position::BottomRight;
        cfg.scale = 1.5;
        cfg.colors[5] = 0x8012_3456;
        cfg.hidden = vec!["minecraft:grass_block".into(), "create:*".into()];
        cfg.id = true;
        let (back, notes) = Config::parse(&cfg.to_text());
        assert_eq!(notes, Vec::<String>::new());
        assert_eq!(back, cfg);
    }

    #[test]
    fn bad_lines_are_reported_and_skipped() {
        let (cfg, notes) = Config::parse("scale = 9\ncolor_name = #12\nwhat\nnope = 1\nshow_id = yes");
        assert_eq!(notes.len(), 4);
        assert_eq!(cfg.scale, 1.0);
        assert!(cfg.id);
    }

    #[test]
    fn hidden_ids_and_mods() {
        let mut cfg = Config::default();
        cfg.apply("hidden", "minecraft:Stone, create:*").unwrap();
        assert!(cfg.hides("minecraft:stone"));
        assert!(cfg.hides("create:cogwheel"));
        assert!(!cfg.hides("minecraft:stone_bricks"));
    }

    #[test]
    fn settings_buttons_cycle() {
        let mut cfg = Config::default();
        let n = cfg.labels().len();
        assert_eq!(cfg.click(2), Click::Changed);
        assert_eq!(cfg.position, Position::TopLeft);
        cfg.click(3);
        assert_eq!(cfg.scale, 1.25);
        cfg.scale = 2.0;
        cfg.click(3);
        assert_eq!(cfg.scale, 0.5);
        cfg.click(5);
        assert!(!cfg.icon);
        assert_eq!(cfg.click(n - 2), Click::Reload);
        assert_eq!(cfg.click(n - 1), Click::Changed);
        assert_eq!(cfg, Config::default());
        assert_eq!(cfg.click(n), Click::Nothing);
    }

    #[test]
    fn the_command() {
        let mut cfg = Config::default();
        assert_eq!(run(&mut cfg, "gamemode creative"), None);
        assert!(run(&mut cfg, "blockinfo").unwrap().changed);
        assert!(!cfg.shown);
        run(&mut cfg, "BlockInfo on");
        assert!(cfg.shown);
        let ran = run(&mut cfg, "blockinfo set color_tool #00FF00").unwrap();
        assert!(ran.changed);
        assert_eq!(cfg.colors[5], 0xFF00_FF00);
        assert!(!run(&mut cfg, "blockinfo set scale lots").unwrap().changed);
        assert!(run(&mut cfg, "blockinfo reload").unwrap().reload);
    }

    #[test]
    fn crops_show_how_far_they_have_grown() {
        let age = |a: &str| vec![("age".to_string(), a.to_string())];
        assert_eq!(growth("minecraft:wheat", &age("7"), true), Some(1.0));
        assert_eq!(growth("minecraft:beetroots", &age("0"), true), Some(0.0));
        assert_eq!(growth("minecraft:nether_wart", &age("1"), false), Some(1.0 / 3.0));
        assert_eq!(growth("minecraft:attached_melon_stem", &[], false), Some(1.0));
        assert_eq!(growth("farmersdelight:rice", &age("3"), true), Some(3.0 / 7.0));
        // A cactus has an age too, but doesn't ripen.
        assert_eq!(growth("minecraft:cactus", &age("3"), false), None);
        let stage = vec![("stage".to_string(), "1".to_string())];
        assert_eq!(growth("minecraft:oak_sapling", &stage, false), Some(1.0));

        let wheat = Target { growth: Some(4.0 / 7.0), ..grass() };
        let p = panel(&Config::default(), &wheat, &measure).unwrap();
        assert!(texts(&p).contains(&"57%".to_string()));
        // The bar: black, then 4/7 of its inside in green.
        let bars: Vec<_> = p
            .draws
            .iter()
            .filter_map(|d| match d {
                Draw::Fill(r, c) if r[3] <= BAR => Some((r[2], *c)),
                _ => None,
            })
            .collect();
        let inside = bars[0].0 - 2.0;
        assert_eq!(bars[1], ((inside * 4.0 / 7.0).round(), 0xFF00_0000 | GREEN));
        let ripe = Target { growth: Some(1.0), ..grass() };
        let p = panel(&Config::default(), &ripe, &measure).unwrap();
        assert!(texts(&p).contains(&"Mature".to_string()));
    }

    #[test]
    fn the_settings_screen_changes_and_saves() {
        let mut s = State::default();
        s.config(None);
        s.open_menu();
        let hotkeys = [("Show / hide".to_string(), key_label(DEFAULT_TOGGLE))];
        let (draws, icons) = s.menu([480.0, 270.0], [0.0, 0.0], &measure, &hotkeys);
        let all: Vec<String> = draws
            .iter()
            .filter_map(|d| match d {
                Draw::Text(t, ..) => Some(t.clone()),
                _ => None,
            })
            .collect();
        for part in ["Block Info", "Shown: On", "Lines", "Preview", "Hotkeys", "I", "57%"] {
            assert!(all.iter().any(|t| t == part), "{part} missing");
        }
        assert_eq!(icons.len(), 2);
        let centre = |s: &State, action: Action| {
            let r = s.widgets.iter().find(|w| w.action == action).unwrap().rect;
            [r[0] + r[2] / 2.0, r[1] + r[3] / 2.0]
        };
        s.menu_click(centre(&s, Action::Shown));
        assert!(!s.cfg().shown);
        s.menu_click(centre(&s, Action::Scale(0.25)));
        assert_eq!(s.cfg().scale, 1.25);
        s.menu_click(centre(&s, Action::Flag(6)));
        assert!(!s.cfg().growth);
        // Reset asks twice.
        s.act(Action::Reset);
        assert!(!s.cfg().shown);
        s.act(Action::Reset);
        assert_eq!(*s.cfg(), Config::default());
        s.menu_key("key.keyboard.escape", DEFAULT_MENU);
        assert!(s.take_close());
    }

    #[test]
    fn names_from_ids() {
        assert_eq!(key_label("key.keyboard.caps.lock"), "Caps Lock");
        assert_eq!(pretty("minecraft:grass_block"), "Grass Block");
        assert_eq!(mod_name("minecraft:grass_block"), "Minecraft");
        assert_eq!(mod_name("better_end:amber_ore"), "Better End");
    }

    #[test]
    fn placement_stays_on_screen() {
        let mut cfg = Config::default();
        assert_eq!(place(&cfg, [100.0, 30.0], [400.0, 300.0]), [150.0, 4.0]);
        cfg.offset = [1000.0, -50.0];
        assert_eq!(place(&cfg, [100.0, 30.0], [400.0, 300.0]), [300.0, 0.0]);
    }

    #[test]
    fn hold_mode_shows_only_while_held() {
        let mut s = State::default();
        s.config(None).mode = Mode::Hold;
        assert!(!s.visible());
        s.held = true;
        assert!(s.visible());
    }
}
