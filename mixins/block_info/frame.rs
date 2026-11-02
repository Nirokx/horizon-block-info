// Once a frame: open the settings screen `/blockinfo menu` asked for once
// chat has closed, close it when the mod asked to, and forget a screen the
// game closed.
fn block_info_frame(game: &mut crate::game::Game<'_>) {
    use crate::game::block_info as m;
    if game.native_playing() && m::with(|s| s.take_open()) && game.native_mod_screen_open(m::ID) {
        m::with(|s| s.open_menu());
    }
    if m::with(|s| s.take_close()) {
        game.native_mod_screen_close(m::ID);
    }
    let ours = game.native_mod_screen() == Some(m::ID);
    m::with(|s| s.sync_open(ours));
}
