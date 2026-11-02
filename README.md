# Block Info for Horizon

A native Horizon mod (Minecraft 26.3, Horizon Nightly 2026-11-01, `c9d474d`) that
shows a small box about whatever the crosshair is on, like Jade or WAILA, in the
same dark panel style as Waypoints and Voice Chat:

```
[icon]  Wheat Crops
        Growth: 57%
        [#########.......]
        Minecraft
```

- **I** shows or hides it. Set `mode = hold` to show it only while the key is held.
- **O** opens the settings screen: every setting as a button or a `- value +` stepper, every line on or off, and a live preview of the box. Both keys can be rebound in Options > Mods.
- **Blocks**: name, icon, tool needed (with its tier: *Iron Pickaxe*), whether your held item harvests it, hardness, block states, coordinates, registry id, and the mod it comes from.
- **Crop growth**: how far a crop has grown, as a percentage and a bar, or *Mature*. Covers wheat, carrots, potatoes, beetroots, nether wart, sweet berries, cocoa, melon and pumpkin stems, torchflower and pitcher crops, saplings, and other mods' crops (anything in `#minecraft:crops` with an `age`).
- **Mobs**: name, spawn-egg icon, health.
- **Options > Mods > Block Info**: a button per setting (position, scale, background, every line on/off, reload, reset).
- **`/blockinfo`** `[on|off|toggle]`, `menu`, `reload`, `reset`, `set <key> <value>` (for example `/blockinfo set color_tool #00FFAA`).

## Config

Everything lives in `config/block_info.cfg` in the game folder. It is written with
its defaults and a comment per setting the first time the mod runs. Edit it, then
`/blockinfo reload`. The settings page and `/blockinfo set` save to the same file.

| Key | Values |
| --- | --- |
| `shown`, `mode` | `true`/`false`; `toggle` or `hold` |
| `position` | `top_center`, `top_left`, `top_right`, `bottom_left`, `bottom_right`, `crosshair` |
| `offset_x`, `offset_y` | GUI pixels added to the position |
| `scale` | 0.25 to 4 |
| `background` | `panel` (the Horizon mods' screens), `tooltip` (vanilla's frame), `flat`, `none` |
| `padding`, `line_gap`, `min_width` | GUI pixels |
| `show_icon`, `show_name`, `show_id`, `show_tool`, `show_tool_tier`, `show_harvestable`, `show_growth`, `show_hardness`, `show_states`, `show_coords`, `show_mod`, `show_entities`, `show_health` | which lines show |
| `text_shadow`, `hide_when_debug` | text shadow; hide while F3 is open |
| `color_background`, `color_border_top`, `color_border_bottom`, `color_name`, `color_label`, `color_tool`, `color_value`, `color_yes`, `color_no`, `color_info`, `color_mod`, `color_health`, `color_growth` | `#RRGGBB` or `#AARRGGBB` |
| `hidden` | ids never shown, comma separated; `mymod:*` hides a whole mod |

The corner positions stack with other mods' HUD boxes and step aside for F3's columns.

## Build and install

Uses the Horizon launcher's own Rust and Horizon source (nothing else to install):

```sh
L=~/.local/share/horizon-launcher
export PATH=$L/rust/1.99.1-x86_64-unknown-linux-gnu/bin:$PATH CARGO_HOME=$L/cargo
cargo test --offline                      # the mod's logic
python3 tools/package.py --install        # target/block_info-1.0.0.hnmod, copied into the launcher's native mods
```

Then build the game with its mods in the launcher. The mod is pinned to Horizon
`c9d474d` because it reads the game's own crosshair target, block facts, tags and the
held item, which the stable hook points don't offer. A newer Nightly may need the pin
in `compatibility.json` and `mixins/block_info/horizon-mixin.toml` moved.

## Layout

- `mixins/block_info/runtime.rs`: the config, the layout and the command, with unit tests (also the module the game gets)
- `mixins/block_info/*.rs`: one small hook adapter per hook point
- `mixins/block_info/horizon-mixin.toml`: where each hook goes
- `tools/package.py`: makes the `.hnmod`
