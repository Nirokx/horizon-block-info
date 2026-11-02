// Draws the settings screen: the controls, the hotkeys as bound now, the
// lines on or off, and the box as it looks with them.
fn block_info_screen(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    use crate::game::block_info as m;
    if game.native_mod_screen() != Some(m::ID) {
        return;
    }
    let hotkeys = [
        ("Show / hide", m::KEY_TOGGLE, m::DEFAULT_TOGGLE),
        ("Open this menu", m::KEY_MENU, m::DEFAULT_MENU),
    ]
    .map(|(what, name, default)| (what.to_owned(), m::key_label(&game.native_mod_key(m::ID, name, default))));
    let path = crate::game::Game::native_config_dir().ok().map(|d| d.join(m::CONFIG_FILE));
    let size = [batch.width, batch.height];
    let measure = |t: &str| game.native_text_width(t);
    let (draws, icons) = m::with(|s| {
        s.config(path);
        s.menu(size, screen.mouse, &measure, &hotkeys)
    });
    for draw in draws {
        match draw {
            m::Draw::Fill(rect, argb) => game.native_fill(batch, rect, argb),
            m::Draw::Text(text, at, rgb, shadow) => game.native_text(batch, &text, at, rgb, shadow),
        }
    }
    for (item, at) in icons {
        game.native_item(batch, &item, at, 16.0);
    }
}
