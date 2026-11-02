// A click on the settings screen: the button under the pointer.
fn block_info_mouse(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    use crate::game::block_info as m;
    if game.native_mod_screen() != Some(m::ID) {
        return;
    }
    *canceled = true;
    if button == 0 {
        m::with(|s| s.menu_click(screen.mouse));
    }
}
