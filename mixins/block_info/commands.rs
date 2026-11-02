// `/blockinfo` completes and colours as the game's own commands do.
fn block_info_commands(commands: &mut Vec<&'static str>) {
    commands.push(crate::game::block_info::COMMAND);
}
