// Keys over the settings screen are the screen's: Escape and the menu key
// close it.
fn block_info_keys(game: &crate::game::Game<'_>, key: &str, modifiers: u32, canceled: &mut bool) {
    use crate::game::block_info as m;
    if game.native_mod_screen() != Some(m::ID) {
        return;
    }
    *canceled = true;
    let _ = modifiers;
    let menu = game.native_mod_key(m::ID, m::KEY_MENU, m::DEFAULT_MENU);
    m::with(|s| s.menu_key(key, &menu));
}
