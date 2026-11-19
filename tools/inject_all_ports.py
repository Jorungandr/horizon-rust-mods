#!/usr/bin/env python3
"""Build a Windows tree of Horizon 9ee13cd0 with the ports injected.

Each port becomes a private module under game/native_hooks/, and the game's own
hook functions call into it: the chat hook for the chat lines, the mods hook for
the Options > Mods row, the settings hooks for each mod's page, the HUD hook for
the two overlays, and the connect/leave/tick hooks for Auto Reconnect.

Run with --keep-target to refresh the sources while keeping the Cargo target
directory (a full rebuild takes ~40 minutes; an incremental one ~2).
Run with --no-copy to patch a tree that is already there.
"""
import pathlib, shutil, sys

ROOT = pathlib.Path(r"G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports")
SRC = ROOT / "horizon-mods-staging/native/target/horizon/9ee13cd038559db05de6a4ced4496028ce409564"
DST = ROOT / "build-win/horizon-src"

# Each source file is a complete module: pure logic, tests, then (for the chat
# ports) the adapter functions the injection points call.
MODS = {
    "timeformatter": ROOT / "timeformatter-horizon/mixins/timeformatter/runtime.rs",
    "whenchat": ROOT / "whenchat-horizon/mixins/whenchat/runtime.rs",
    "chattimestamps": ROOT / "chattimestamps-horizon/mixins/chattimestamps/runtime.rs",
    "coords": ROOT / "ports/coords.rs",
    "durability": ROOT / "ports/durability.rs",
    "autoreconnect": ROOT / "ports/autoreconnect.rs",
    "chathighlight": ROOT / "ports/chat_highlight.rs",
    "fpshud": ROOT / "ports/fps_hud.rs",
    "itemtooltip": ROOT / "ports/item_tooltip.rs",
}

# The ports with a settings page of their own.
SETTINGS_PORTS = (
    "chattimestamps", "coords", "durability", "autoreconnect",
    "chathighlight", "fpshud", "itemtooltip",
)
# The overlay ports that draw into the HUD.
HUD_PORTS = ("coords", "durability", "fpshud")

KEEP = "--keep-target" in sys.argv
if "--no-copy" not in sys.argv:
    if DST.exists() and not KEEP:
        shutil.rmtree(DST)
    print("copying tree ...")
    shutil.copytree(
        SRC,
        DST,
        dirs_exist_ok=KEEP,
        ignore=shutil.ignore_patterns("target"),
    )
    print("copied (target kept)" if KEEP else "copied")

sub = DST / "crates/horizon-client/src/game/native_hooks"
sub.mkdir(parents=True, exist_ok=True)

TEXT = "horizon_client::text::Text"
GAME = "horizon_client::game::Game"
HUD = "horizon_client::hud::HudBatch"
CORNERS = "horizon_client::extras_hud::Corners"
CORNER = "horizon_client::extras::Corner"

TAIL = {
    "timeformatter": f'''

/// Rewrite every well-formed `<t:...>` inside each span, keeping each
/// span's own style.
///
/// A `Text` carries two representations: `spans` (what the game draws) and
/// `source`, the component it was built from, which `component()` returns.
/// Editing `spans` in place would leave `source` showing the raw tag, so the
/// line is rebuilt through `Text::styled`, which keeps spans and component
/// in step. Only lines that hold a tag are rebuilt.
pub fn apply(message: &mut {TEXT}) {{
    if !message.spans.iter().any(|s| format_literal(&s.text).is_some()) {{
        return;
    }}
    let mut rebuilt = {TEXT}::empty();
    for span in &message.spans {{
        let text = format_literal(&span.text).unwrap_or_else(|| span.text.clone());
        rebuilt.push({TEXT}::styled(text, span.style.clone()));
    }}
    *message = rebuilt;
}}
''',
    "whenchat": f'''

/// Prepend `[HH:mm:ss] ` to the line, as the mod does. The line becomes one
/// fresh literal span, which is what the original mod's plain prefix wants.
pub fn apply(game: &mut {GAME}<'_>, system: bool, overlay: bool, message: &mut {TEXT}) {{
    let _ = game;
    if overlay {{
        return;
    }}
    let original = message.plain();
    if original.is_empty() {{
        return;
    }}
    let Some(decorated) = decorate(
        Line {{
            text: &original,
            system,
            overlay,
        }},
        &local_prefix(),
    ) else {{
        return;
    }};
    *message = {TEXT}::literal(decorated);
}}
''',
    "chattimestamps": f'''

/// The stored settings, with `format` repaired.
///
/// `Game::native_setting` trims every value it reads, but the format's
/// trailing space is what separates the timestamp from the message. Without
/// this, opening the settings page and clicking anything would store the
/// format trimmed and every later read would report `Custom`, gluing the
/// timestamp to the text. A trimmed match against a preset restores the
/// preset exactly.
fn read_settings(game: &{GAME}<'_>) -> Settings {{
    let mut settings = settings(|key| game.native_setting(ID, key));
    if let Some((value, _)) = FORMAT_PRESETS
        .iter()
        .find(|(value, _)| value.trim() == settings.format.trim())
    {{
        settings.format = (*value).to_owned();
    }}
    settings
}}

/// Add the configured timestamp: as a hover tooltip on every span, or as
/// leading spans that keep the original spans (and their styling).
pub fn apply(game: &mut {GAME}<'_>, message: &mut {TEXT}) {{
    use horizon_client::text::named_color;

    fn seg_text(text: &str, color: Option<&str>) -> {TEXT} {{
        match color.and_then(named_color) {{
            Some(c) => {TEXT}::colored(text, c),
            None => {TEXT}::literal(text),
        }}
    }}

    fn build_timestamp(segments: &[Segment]) -> {TEXT} {{
        let mut out = {TEXT}::empty();
        for seg in segments {{
            out.push(seg_text(&seg.text, seg.color));
        }}
        out
    }}

    let settings = read_settings(game);
    if !settings.enable_mod {{
        return;
    }}
    let Some(segments) = timestamp_now(&settings.format) else {{
        return;
    }};
    if segments.is_empty() {{
        return;
    }}

    if settings.enable_hover {{
        let stamp = build_timestamp(&segments);
        // Rebuild rather than edit `style.hover` in place: `Text` also keeps
        // the component it was built from, and only a rebuild keeps that in
        // step with the spans (see `ports_timeformatter::apply`).
        let mut rebuilt = {TEXT}::empty();
        for span in &message.spans {{
            let mut style = span.style.clone();
            style.hover = Some(Box::new(stamp.clone()));
            rebuilt.push({TEXT}::styled(span.text.clone(), style));
        }}
        *message = rebuilt;
    }} else {{
        let original = std::mem::take(message);
        let mut wrapper = {TEXT}::empty();
        for seg in &segments {{
            wrapper.push(seg_text(&seg.text, seg.color));
        }}
        wrapper.push(original);
        *message = wrapper;
    }}
}}

/// The buttons of this mod's settings page, under its keys.
pub fn settings_page(game: &{GAME}<'_>, mod_id: &str, buttons: &mut Vec<String>) {{
    if mod_id != ID {{
        return;
    }}
    let settings = read_settings(game);
    buttons.extend(settings.labels());
}}

/// Button `button` of this mod's settings page was clicked: advance it and
/// keep the choice.
pub fn settings_click(game: &mut {GAME}<'_>, mod_id: &str, button: usize) {{
    if mod_id != ID {{
        return;
    }}
    let mut settings = read_settings(game);
    settings.click(button);
    for (key, value) in settings.stored() {{
        game.native_set_setting(ID, key, &value);
    }}
}}
''',
    "coords": f'''

fn read_settings(game: &{GAME}<'_>) -> Settings {{
    Settings::from_lookup(|key| game.native_setting(ID, key))
}}

fn corner(placement: Placement) -> {CORNER} {{
    match placement {{
        Placement::TopLeft => {CORNER}::TopLeft,
        Placement::TopRight => {CORNER}::TopRight,
        Placement::BottomLeft => {CORNER}::BottomLeft,
        Placement::BottomRight => {CORNER}::BottomRight,
    }}
}}

/// Draw the panel in its corner, once a frame while the world is drawn.
pub fn hud(game: &{GAME}<'_>, batch: &mut {HUD}, corners: &mut {CORNERS}) {{
    let settings = read_settings(game);
    if !settings.enable_mod {{
        return;
    }}
    // `native_player` is set whenever a world is loaded, while the camera
    // frame is only filled while the world is drawn, so read the player.
    let Some(player) = game.native_player() else {{
        return;
    }};
    let rows = lines(
        &settings,
        [player.x, player.y, player.z],
        player.yaw,
        player.dimension,
    );
    if rows.is_empty() {{
        return;
    }}
    const LINE: f32 = 10.0;
    const PAD: f32 = 2.0;
    let width = rows
        .iter()
        .map(|row| game.native_text_width(row))
        .fold(0.0f32, f32::max)
        + PAD * 2.0;
    let height = rows.len() as f32 * LINE + PAD * 2.0;
    let (x, y) = corners.place(corner(settings.placement), width, height);
    // The extras' translucent black box, so the text reads on any terrain.
    game.native_fill(batch, [x, y, width, height], 0x78000000);
    for (i, row) in rows.iter().enumerate() {{
        game.native_text(
            batch,
            row,
            [x + PAD, y + PAD + i as f32 * LINE],
            0xFFFFFF,
            true,
        );
    }}
    static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {{
        log::debug!(
            "coords: drew at ({{x}}, {{y}}) size {{width}}x{{height}}: {{rows:?}}"
        );
    }}
}}

pub fn settings_page(game: &{GAME}<'_>, mod_id: &str, buttons: &mut Vec<String>) {{
    if mod_id != ID {{
        return;
    }}
    let settings = read_settings(game);
    buttons.extend(settings.labels());
}}

pub fn settings_click(game: &mut {GAME}<'_>, mod_id: &str, button: usize) {{
    if mod_id != ID {{
        return;
    }}
    let mut settings = read_settings(game);
    settings.click(button);
    for (key, value) in settings.stored() {{
        game.native_set_setting(ID, key, &value);
    }}
}}
''',
    "durability": f'''

fn read_settings(game: &{GAME}<'_>) -> Settings {{
    Settings::from_lookup(|key| game.native_setting(ID, key))
}}

fn corner(placement: Placement) -> {CORNER} {{
    match placement {{
        Placement::TopLeft => {CORNER}::TopLeft,
        Placement::TopRight => {CORNER}::TopRight,
        Placement::BottomLeft => {CORNER}::BottomLeft,
        Placement::BottomRight => {CORNER}::BottomRight,
    }}
}}

/// The gear worn or held now, with each item's own maximum.
fn current_bars(game: &{GAME}<'_>) -> Vec<Bar> {{
    use horizon_client::mod_api::EquipmentSlot;

    let items = game.res.items();
    let mut out = Vec::new();
    for (slot, engine) in [
        (Slot::Head, EquipmentSlot::Head),
        (Slot::Chest, EquipmentSlot::Chest),
        (Slot::Legs, EquipmentSlot::Legs),
        (Slot::Feet, EquipmentSlot::Feet),
        (Slot::MainHand, EquipmentSlot::MainHand),
        (Slot::OffHand, EquipmentSlot::OffHand),
    ] {{
        let Some(stack) = game.native_equipment(engine) else {{
            continue;
        }};
        let Some(info) = items.items.get(&stack.item) else {{
            continue;
        }};
        out.push(Bar {{
            slot,
            item: stack.item.to_string(),
            remaining: remaining(info.max_damage, stack.damage),
            max: info.max_damage,
        }});
    }}
    out
}}

/// Draw one row per piece of gear, once a frame while the world is drawn.
pub fn hud(game: &{GAME}<'_>, batch: &mut {HUD}, corners: &mut {CORNERS}) {{
    let settings = read_settings(game);
    if !settings.enable_mod {{
        return;
    }}
    let rows = visible(current_bars(game), &settings);
    if rows.is_empty() {{
        return;
    }}
    const ROW: f32 = 10.0;
    const ICON: f32 = 10.0;
    const GAP: f32 = 2.0;
    const PAD: f32 = 2.0;
    let width = rows
        .iter()
        .map(|bar| {{
            let icon = if settings.show_icons {{ ICON + GAP }} else {{ 0.0 }};
            let text = if settings.show_percent {{
                game.native_text_width(&bar.label())
            }} else {{
                30.0
            }};
            icon + text
        }})
        .fold(0.0f32, f32::max)
        + PAD * 2.0;
    let height = rows.len() as f32 * ROW + PAD * 2.0;
    let (x, y) = corners.place(corner(settings.placement), width, height);
    game.native_fill(batch, [x, y, width, height], 0x78000000);
    static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {{
        log::debug!(
            "durability: drew at ({{x}}, {{y}}) size {{width}}x{{height}}: {{:?}}",
            rows.iter().map(|bar| (bar.slot, bar.label())).collect::<Vec<_>>()
        );
    }}
    for (i, bar) in rows.iter().enumerate() {{
        let row_y = y + PAD + i as f32 * ROW;
        let mut cursor = x + PAD;
        if settings.show_icons && game.native_item(batch, &bar.item, [cursor, row_y], ICON) {{
            cursor += ICON + GAP;
        }}
        let color = bar.color();
        if settings.show_percent {{
            game.native_text(batch, &bar.label(), [cursor, row_y], color, true);
        }} else {{
            // No number: the bar itself carries the level.
            let filled = 30.0 * bar.percent() / 100.0;
            game.native_fill(batch, [cursor, row_y + 2.0, filled, 6.0], 0xFF00_0000 | color);
            game.native_fill(batch, [cursor, row_y + 2.0, 30.0, 6.0], 0x4000_0000);
        }}
    }}
}}

pub fn settings_page(game: &{GAME}<'_>, mod_id: &str, buttons: &mut Vec<String>) {{
    if mod_id != ID {{
        return;
    }}
    let settings = read_settings(game);
    buttons.extend(settings.labels());
}}

pub fn settings_click(game: &mut {GAME}<'_>, mod_id: &str, button: usize) {{
    if mod_id != ID {{
        return;
    }}
    let mut settings = read_settings(game);
    settings.click(button);
    for (key, value) in settings.stored() {{
        game.native_set_setting(ID, key, &value);
    }}
}}
''',
    "autoreconnect": f'''

use std::cell::{{Cell, RefCell}};
use std::time::Instant;

/// The last server this client joined, kept so a lost connection can be
/// retried without the player retyping the address.
struct LastServer {{
    address: String,
    name: String,
    cookies: Option<horizon_client::net::Cookies>,
    lan: bool,
    realm: bool,
}}

thread_local! {{
    static LAST: RefCell<Option<LastServer>> = const {{ RefCell::new(None) }};
    static RETRY: RefCell<Retry> = const {{
        RefCell::new(Retry {{
            attempts: 0,
            remaining_ms: None,
        }})
    }};
    /// When `tick` last ran, for the countdown's step. `Cell` because
    /// `Instant` is `Copy` and nothing borrows it across the update.
    static LAST_TICK: Cell<Option<Instant>> = const {{ Cell::new(None) }};
}}

fn read_settings(game: &{GAME}<'_>) -> Settings {{
    Settings::from_lookup(|key| game.native_setting(ID, key))
}}

/// Remember the server being joined, so a lost connection can be retried.
pub fn record(
    address: &str,
    name: &str,
    cookies: Option<horizon_client::net::Cookies>,
    lan: bool,
    realm: bool,
) {{
    LAST.with(|last| {{
        *last.borrow_mut() = Some(LastServer {{
            address: address.to_owned(),
            name: name.to_owned(),
            cookies,
            lan,
            realm,
        }});
    }});
}}

/// The connection left: start the countdown only for a lost one. `lost` is
/// false when the player left on purpose.
pub fn on_disconnect(game: &{GAME}<'_>, lost: bool) {{
    let settings = read_settings(game);
    RETRY.with(|retry| {{
        let started = retry.borrow_mut().lost(&settings, lost);
        if started {{
            log::info!("Auto Reconnect: connection lost, waiting {{}}s", settings.delay);
        }}
    }});
}}

/// A join succeeded: the attempt budget starts over.
pub fn joined() {{
    RETRY.with(|retry| retry.borrow_mut().joined());
}}

/// The countdown line for the disconnect screen, when one is pending.
pub fn status(game: &{GAME}<'_>) -> Option<String> {{
    let settings = read_settings(game);
    RETRY.with(|retry| countdown_text(&retry.borrow(), &settings))
}}

/// Once a tick: advance the countdown and rejoin when it runs out.
pub fn tick(game: &mut {GAME}<'_>) {{
    // A tick's own duration is what the countdown steps by, so a long frame
    // does not stretch the wait.
    let now = Instant::now();
    let dt_ms = LAST_TICK.with(|last| {{
        let previous = last.replace(Some(now));
        previous.map_or(0, |previous| {{
            now.duration_since(previous).as_millis().min(u128::from(u64::MAX)) as u64
        }})
    }});
    let fire = RETRY.with(|retry| retry.borrow_mut().advance(dt_ms));
    if !fire {{
        return;
    }}
    let settings = read_settings(game);
    if !settings.enable_mod {{
        return;
    }}
    let Some((address, name, cookies, lan, realm)) = LAST.with(|last| {{
        last.borrow().as_ref().map(|server| {{
            (
                server.address.clone(),
                server.name.clone(),
                server.cookies.clone(),
                server.lan,
                server.realm,
            )
        }})
    }}) else {{
        return;
    }};
    log::info!("Auto Reconnect: rejoining {{address}} as {{name}}");
    if let Err(error) = game.connect_server_with_options(&address, &name, cookies, lan, realm) {{
        log::warn!("Auto Reconnect: rejoin failed: {{error}}");
    }}
}}

pub fn settings_page(game: &{GAME}<'_>, mod_id: &str, buttons: &mut Vec<String>) {{
    if mod_id != ID {{
        return;
    }}
    let settings = read_settings(game);
    buttons.extend(settings.labels());
}}

pub fn settings_click(game: &mut {GAME}<'_>, mod_id: &str, button: usize) {{
    if mod_id != ID {{
        return;
    }}
    let mut settings = read_settings(game);
    settings.click(button);
    for (key, value) in settings.stored() {{
        game.native_set_setting(ID, key, &value);
    }}
}}
''',
    "chathighlight": f'''

fn read_settings(game: &{GAME}<'_>) -> Settings {{
    Settings::from_lookup(|key| game.native_setting(ID, key))
}}

/// Rebuild each span, recolouring the parts that match a keyword. As with
/// the other chat ports, spans must be rebuilt (`Text::styled`) rather than
/// edited in place so `Text`'s private `source` stays in sync.
pub fn apply(game: &{GAME}<'_>, message: &mut {TEXT}) {{
    use horizon_client::text::named_color;

    let settings = read_settings(game);
    if !settings.enable_mod {{
        return;
    }}
    let words = keywords(&settings);
    if words.is_empty() {{
        return;
    }}
    let Some(color) = named_color(settings.color_name()) else {{
        return;
    }};
    let mut rebuilt = {TEXT}::empty();
    for span in &message.spans {{
        let pieces = highlight(&span.text, &words, settings.match_case);
        if pieces.iter().all(|p| !p.hit) {{
            rebuilt.push({TEXT}::styled(span.text.clone(), span.style.clone()));
            continue;
        }}
        for piece in pieces {{
            if piece.hit {{
                let mut style = span.style.clone();
                style.color = Some(color);
                rebuilt.push({TEXT}::styled(piece.text, style));
            }} else {{
                rebuilt.push({TEXT}::styled(piece.text, span.style.clone()));
            }}
        }}
    }}
    *message = rebuilt;
}}

pub fn settings_page(game: &{GAME}<'_>, mod_id: &str, buttons: &mut Vec<String>) {{
    if mod_id != ID {{
        return;
    }}
    buttons.extend(read_settings(game).labels());
}}

pub fn settings_click(game: &mut {GAME}<'_>, mod_id: &str, button: usize) {{
    if mod_id != ID {{
        return;
    }}
    let mut settings = read_settings(game);
    settings.click(button);
    for (key, value) in settings.stored() {{
        game.native_set_setting(ID, key, &value);
    }}
}}
''',
    "fpshud": f'''

fn read_settings(game: &{GAME}<'_>) -> Settings {{
    Settings::from_lookup(|key| game.native_setting(ID, key))
}}

fn corner(placement: Placement) -> {CORNER} {{
    match placement {{
        Placement::TopLeft => {CORNER}::TopLeft,
        Placement::TopRight => {CORNER}::TopRight,
        Placement::BottomLeft => {CORNER}::BottomLeft,
        Placement::BottomRight => {CORNER}::BottomRight,
    }}
}}

/// Draw the readings in the chosen corner using the engine's own frame meter.
pub fn hud(game: &{GAME}<'_>, batch: &mut {HUD}, corners: &mut {CORNERS}) {{
    let settings = read_settings(game);
    if !settings.enable_mod {{
        return;
    }}
    let readings = game.native_frame_rate();
    let rows = lines(&settings, readings);
    const LINE: f32 = 10.0;
    const PAD: f32 = 2.0;
    let width = rows
        .iter()
        .map(|row| game.native_text_width(row))
        .fold(0.0f32, f32::max)
        + PAD * 2.0;
    let height = rows.len() as f32 * LINE + PAD * 2.0;
    let (x, y) = corners.place(corner(settings.placement), width, height);
    game.native_fill(batch, [x, y, width, height], 0x78000000);
    for (i, row) in rows.iter().enumerate() {{
        let color = if i == 0 {{ fps_color(readings.0) }} else {{ 0xFFFFFF }};
        game.native_text(batch, row, [x + PAD, y + PAD + i as f32 * LINE], color, true);
    }}
}}

pub fn settings_page(game: &{GAME}<'_>, mod_id: &str, buttons: &mut Vec<String>) {{
    if mod_id != ID {{
        return;
    }}
    buttons.extend(read_settings(game).labels());
}}

pub fn settings_click(game: &mut {GAME}<'_>, mod_id: &str, button: usize) {{
    if mod_id != ID {{
        return;
    }}
    let mut settings = read_settings(game);
    settings.click(button);
    for (key, value) in settings.stored() {{
        game.native_set_setting(ID, key, &value);
    }}
}}
''',
    "itemtooltip": f'''

fn read_settings(game: &{GAME}<'_>) -> Settings {{
    Settings::from_lookup(|key| game.native_setting(ID, key))
}}

/// Append the extra lines to an item's tooltip.
pub fn apply(
    game: &{GAME}<'_>,
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<({TEXT}, u32)>,
) {{
    let settings = read_settings(game);
    if !settings.enable_mod {{
        return;
    }}
    let Some(info) = game.res.items().items.get(&stack.item) else {{
        return;
    }};
    let data = ItemData {{
        id: &stack.item,
        count: stack.count,
        max_stack: info.max_stack,
        damage: stack.damage,
        max_damage: info.max_damage,
    }};
    for line in extra_lines(&settings, &data) {{
        lines.push(({TEXT}::literal(line.text), line.color));
    }}
}}

pub fn settings_page(game: &{GAME}<'_>, mod_id: &str, buttons: &mut Vec<String>) {{
    if mod_id != ID {{
        return;
    }}
    buttons.extend(read_settings(game).labels());
}}

pub fn settings_click(game: &mut {GAME}<'_>, mod_id: &str, button: usize) {{
    if mod_id != ID {{
        return;
    }}
    let mut settings = read_settings(game);
    settings.click(button);
    for (key, value) in settings.stored() {{
        game.native_set_setting(ID, key, &value);
    }}
}}
''',
}

for name, path in MODS.items():
    source = path.read_text(encoding="utf-8")
    if name in TAIL:
        source += TAIL[name]
    (sub / f"ports_{name}.rs").write_text(source, encoding="utf-8")
    print("wrote", f"ports_{name}.rs")

hooks = DST / "crates/horizon-client/src/game/native_hooks.rs"
text = hooks.read_text(encoding="utf-8")
assert "ports_timeformatter" not in text, "tree already patched; use --no-copy to skip recopy"

# `pub(super)` so game/remote.rs can call the reconnect port.
decls = "".join(
    f'#[path = "native_hooks/ports_{n}.rs"]\npub(super) mod ports_{n};\n' for n in MODS
)
anchor = "use super::native_data_maps::NativeDataMapType;"
assert text.count(anchor) == 1
text = text.replace(anchor, decls + "\n" + anchor)

chat_call = '''if !overlay {
            ports_whenchat::apply(&mut *self, system, overlay, message);
            ports_chattimestamps::apply(&mut *self, message);
            ports_timeformatter::apply(message);
            ports_chathighlight::apply(&*self, message);
        }
        '''
anchor = "mod_api::dispatch_client_chat_received("
assert text.count(anchor) == 1
text = text.replace(anchor, chat_call + anchor)

rows = ""
for name, title, desc in (
    ("whenchat", "WhenChat 0.1.0", "Prepends [HH:mm:ss] to every incoming chat message."),
    ("chattimestamps", "Plague Chat Timestamps 0.1.0", "Customizable chat timestamps: time/date formatting, colours and hover mode."),
    ("timeformatter", "Time Formatter 0.1.0", "Formats Discord-style <t:...> timestamps in chat into your local time."),
    ("coords", "Coordinates HUD 0.1.0", "Shows your position, facing and dimension in a screen corner."),
    ("durability", "Durability HUD 0.1.0", "Shows how much durability your armour and held items have left."),
    ("autoreconnect", "Auto Reconnect 0.1.0", "Rejoins the last server after the connection is lost."),
    ("chathighlight", "Chat Highlight 0.1.0", "Colours configured keywords in incoming chat."),
    ("fpshud", "FPS HUD 0.1.0", "Shows frames per second, slowest frame and 1% low."),
    ("itemtooltip", "Item Tooltip+ 0.1.0", "Adds item id, durability and stack size to tooltips."),
):
    rows += (
        f'        mods.push(NativeMod {{\n'
        f'            id: "{name}".into(),\n'
        f'            title: "{title}".into(),\n'
        f'            description: "{desc}".into(),\n'
        f'            keys: Vec::new(),\n'
        f'        }});\n'
    )
anchor = "mod_api::dispatch_mods(self, mod_api::mods(), mods);"
assert text.count(anchor) == 1
text = text.replace(anchor, rows + "        " + anchor)

# Settings pages: one call per port, each guarded by its own module.
settings_calls = "".join(
    f"        ports_{n}::settings_page(&*self, mod_id, buttons);\n" for n in SETTINGS_PORTS
)
anchor = "mod_api::dispatch_settings(self, mod_api::mods(), mod_id, buttons);"
assert text.count(anchor) == 1
text = text.replace(anchor, settings_calls + "        " + anchor)

settings_click_calls = "".join(
    f"        ports_{n}::settings_click(&mut *self, mod_id, button);\n" for n in SETTINGS_PORTS
)
anchor = "mod_api::dispatch_settings_click(self, mod_api::mods(), mod_id, button);"
assert text.count(anchor) == 1
text = text.replace(anchor, settings_click_calls + "        " + anchor)

# The HUD overlays, drawn before the registered mods' own panels.
hud_calls = "".join(f"        ports_{n}::hud(&*self, batch, corners);\n" for n in HUD_PORTS)
anchor = "mod_api::dispatch_hud(self, mod_api::mods(), batch, corners);"
assert text.count(anchor) == 1
text = text.replace(anchor, hud_calls + "        " + anchor)

# Item Tooltip+: append our lines to the tooltip before the registered mods
# get it, so ours sit after the vanilla lines.
tooltip_call = """        ports_itemtooltip::apply(self, stack, lines);
        """
anchor = "mod_api::dispatch_tooltip(self, mod_api::mods(), stack, lines);"
assert text.count(anchor) == 1
text = text.replace(anchor, tooltip_call + anchor)

# Auto Reconnect: the countdown advances once a tick, and a completed join
# resets the attempt budget.
anchor = "mod_api::dispatch_tick(self, mod_api::mods());"
assert text.count(anchor) == 1
text = text.replace(anchor, "        ports_autoreconnect::tick(self);\n        " + anchor)

anchor = """    pub(super) fn native_client_level_load(&mut self, dimension: &str) {"""
assert text.count(anchor) == 1
text = text.replace(
    anchor,
    anchor + "\n        ports_autoreconnect::joined();",
)

hooks.write_text(text, encoding="utf-8")
print("patched native_hooks.rs")

remote = DST / "crates/horizon-client/src/game/remote.rs"
rtext = remote.read_text(encoding="utf-8")

# `chat_received` skips its hook when no *registered* mod_api mod exists.
# Our ports are compiled into the binary, not registered, so the guard would
# silently skip every chat line. Remove that one guard (it is unique in the
# file); dispatching to an empty mod list is still a no-op.
guard = (
    "        if horizon_client::mod_api::mods().is_empty() {\n"
    "            return true;\n"
    "        }\n"
)
assert rtext.count(guard) == 1, "chat_received guard not found exactly once"
rtext = rtext.replace(
    guard,
    "        // Ports are compiled in: run the hook even with an empty mod_api registry.\n",
)

# Remember what a join needs, so Auto Reconnect can repeat it.
anchor = """        super::world_loading::preload_server_loot(&self.options.resources);
        if self.defer_world_entry(super::PendingWorldEntry::Server {"""
assert rtext.count(anchor) == 1, "connect_server_with_options anchor not found"
rtext = rtext.replace(
    anchor,
    """        super::native_hooks::ports_autoreconnect::record(
            address,
            name,
            cookies.clone(),
            lan,
            realm,
        );
        super::world_loading::preload_server_loot(&self.options.resources);
        if self.defer_world_entry(super::PendingWorldEntry::Server {""",
)

# Start (or cancel) the countdown when a connection ends.
anchor = """        self.finish_perf();
        if reason.is_none() {
            // Leaving on purpose: nothing for auto-reconnect to retry.
            self.extras_left();
        }"""
assert rtext.count(anchor) == 1, "leave_server anchor not found"
rtext = rtext.replace(
    anchor,
    """        self.finish_perf();
        super::native_hooks::ports_autoreconnect::on_disconnect(self, reason.is_some());
        if reason.is_none() {
            // Leaving on purpose: nothing for auto-reconnect to retry.
            self.extras_left();
        }""",
)

remote.write_text(rtext, encoding="utf-8")
print("patched remote.rs (chat guard, reconnect record + disconnect)")
print("done")
