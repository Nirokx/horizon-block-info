// A click on the settings page: the setting steps on and the file is saved,
// or the file is read again.
fn block_info_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::block_info as m;
    if mod_id != m::ID {
        return;
    }
    let _ = game;
    let path = crate::game::Game::native_config_dir().ok().map(|d| d.join(m::CONFIG_FILE));
    m::with(|s| match s.config(path).click(button) {
        m::Click::Changed => s.save(),
        m::Click::Reload => s.reload(),
        m::Click::Nothing => {}
    });
}
