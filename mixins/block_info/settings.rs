// The mod's settings page: one button per setting, as the config file has them.
fn block_info_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::block_info as m;
    if mod_id != m::ID {
        return;
    }
    let _ = game;
    let path = crate::game::Game::native_config_dir().ok().map(|d| d.join(m::CONFIG_FILE));
    buttons.extend(m::with(|s| s.config(path).labels()));
}
