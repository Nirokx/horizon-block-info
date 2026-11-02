// `/blockinfo [on|off|toggle|menu|reload|reset|set <key> <value>]`; every other
// line goes on to the game and to other mods.
fn block_info_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    use crate::game::block_info as m;
    if *ran || !game.native_on(m::ID) {
        return;
    }
    let path = crate::game::Game::native_config_dir().ok().map(|d| d.join(m::CONFIG_FILE));
    let Some(chat) = m::with(|s| {
        let done = m::run(s.config(path), command)?;
        if done.changed {
            s.save();
        }
        if done.open {
            s.request_open();
        }
        if done.reload {
            s.reload();
        }
        let mut chat = done.chat;
        chat.extend(s.take_notes());
        Some(chat)
    }) else {
        return;
    };
    *ran = true;
    for line in chat {
        game.native_chat(&line);
    }
}
