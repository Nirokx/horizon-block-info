// While playing, the toggle key (I) shows or hides the box (in toggle mode,
// saved; in hold mode it shows while the key is down) and the menu key (O)
// opens the settings screen.
fn block_info_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::block_info as m;
    if !game.native_on(m::ID) {
        return;
    }
    let toggle = game.native_mod_key(m::ID, m::KEY_TOGGLE, m::DEFAULT_TOGGLE);
    let menu = game.native_mod_key(m::ID, m::KEY_MENU, m::DEFAULT_MENU);
    if key != toggle && key != menu {
        return;
    }
    if pressed && !game.native_playing() {
        return;
    }
    let path = crate::game::Game::native_config_dir().ok().map(|d| d.join(m::CONFIG_FILE));
    if key == menu {
        m::with(|s| {
            s.config(path);
        });
        if pressed && game.native_mod_screen_open(m::ID) {
            m::with(|s| s.open_menu());
        }
        return;
    }
    m::with(|s| {
        s.held = pressed;
        let cfg = s.config(path);
        if pressed && cfg.mode == m::Mode::Toggle {
            cfg.shown = !cfg.shown;
            s.save();
        }
    });
}
