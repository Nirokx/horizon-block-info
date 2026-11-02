// The box: what the crosshair is on (the nearest mob before the targeted
// block, else the block, with how far it has grown if it is a crop), laid out by the runtime and drawn where the config
// puts it, at its scale.
fn block_info_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::block_info as m;
    use horizon_client::extras::Corner;
    if !game.native_on(m::ID) || !game.native_window() || game.native_player().is_none() {
        return;
    }
    let path = crate::game::Game::native_config_dir().ok().map(|d| d.join(m::CONFIG_FILE));
    let (cfg, visible, notes) = m::with(|s| {
        let cfg = s.config(path).clone();
        (cfg, s.visible(), s.take_notes())
    });
    for note in notes {
        game.native_chat(&note);
    }
    if !visible || (cfg.hide_when_debug && game.debug) {
        return;
    }
    let game = &*game;
    // The language's name (`block.minecraft.grass_block`), else one made from the id.
    let name_of = |kind: &str, id: &str| {
        let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
        let key = format!("{kind}.{ns}.{}", path.replace('/', "."));
        horizon_client::lang::get_or(&key, &m::pretty(id))
    };
    let has_item = |item: &str| !item.is_empty() && game.res.items().get(item).is_some();
    let mob = cfg
        .entities
        .then(|| game.mob_hit())
        .flatten()
        .filter(|&(_, distance)| game.before_block(distance))
        .map(|(i, _)| &game.mobs[i]);
    let target = if let Some(mob) = mob {
        let id = mob.kind.id.to_string();
        let egg = format!("{id}_spawn_egg");
        m::Target {
            name: name_of("entity", &id),
            icon: has_item(&egg).then_some(egg),
            health: Some([mob.health, horizon_client::mobs::finalize::max_health(mob)]),
            id,
            ..Default::default()
        }
    } else {
        let Some(hit) = game.pick() else {
            return;
        };
        let state = game.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
        let facts = game.res.facts().get(state);
        let items = game.res.items();
        let id = facts.name.to_string();
        let item = facts.item.to_string();
        let states: Vec<(String, String)> = game
            .pack
            .assets
            .registry
            .state(state)
            .map(|s| s.properties.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        let crop = items.block_in_tag("minecraft:crops", &id);
        m::Target {
            name: name_of("block", &id),
            icon: has_item(&item).then_some(item),
            tool: m::tool(|tag| items.block_in_tag(tag, &id)),
            harvestable: Some(!facts.requires_tool || items.correct_for_drops(game.held(), &id)),
            hardness: Some(facts.hardness),
            growth: m::growth(&id, &states, crop),
            states,
            pos: Some(hit.pos),
            id,
            ..Default::default()
        }
    };
    if cfg.hides(&target.id) {
        return;
    }
    let measure = |t: &str| game.native_text_width(t);
    let Some(panel) = m::panel(&cfg, &target, &measure) else {
        return;
    };
    let scale = cfg.scale;
    let size = panel.size.map(|v| v * scale);
    let screen = [batch.width, batch.height];
    let at = match cfg.position.corner() {
        Some(i) => {
            let corner = Corner::ALL[i];
            if !game.native_corner_shown(corner) {
                return;
            }
            let (x, y) = corners.place(corner, size[0], size[1]);
            m::clamp([x + cfg.offset[0], y + cfg.offset[1]], size, screen)
        }
        None => m::place(&cfg, size, screen),
    };
    // Drawn unscaled from `at`, then everything since `start` scaled about it.
    let start = batch.vertices.len();
    for draw in panel.draws {
        match draw {
            m::Draw::Fill([x, y, w, h], argb) => {
                game.native_fill(batch, [at[0] + x, at[1] + y, w, h], argb)
            }
            m::Draw::Text(text, [x, y], rgb, shadow) => {
                game.native_text(batch, &text, [at[0] + x, at[1] + y], rgb, shadow)
            }
        }
    }
    if let Some((item, [x, y])) = panel.icon {
        game.native_item(batch, &item, [at[0] + x, at[1] + y], 16.0);
    }
    if scale != 1.0 {
        batch.scale_since(start, at[0], at[1], scale, scale);
    }
}
