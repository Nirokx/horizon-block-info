// The mod's row in Options > Mods, with its two keys.
fn block_info_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::block_info as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: vec![
            [m::KEY_TOGGLE.into(), "Show / hide block info".into(), m::DEFAULT_TOGGLE.into()],
            [m::KEY_MENU.into(), "Block info settings".into(), m::DEFAULT_MENU.into()],
        ],
    });
}
