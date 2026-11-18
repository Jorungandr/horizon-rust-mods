use horizon_client::extras::Corner;
use horizon_client::mod_api::{SettingsClickEvent, SettingsEvent, HudEvent};
use runtime::{lines, DESCRIPTION, ID, NAME, Placement, Settings, VERSION};

mod runtime;

// The shared header: a port uses the subset its hooks need.
#[allow(unused_imports)]
use horizon_client::mod_api::{
    ClientChatReceivedEvent, Flow, Hooks, Host, ModsEvent, NativeMod, Text,
};

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(Mod)
}

struct Mod;

/// The mod's row in Options > Mods.
fn push_row(event: &mut ModsEvent<'_>) {
    event.mods.push(NativeMod {
        id: ID.to_owned(),
        title: format!("{NAME} {VERSION}"),
        description: DESCRIPTION.to_owned(),
        keys: Vec::new(),
    });
}

fn read_settings(host: &dyn Host) -> Settings {
    Settings::from_lookup(|key| host.setting(ID, key))
}

fn corner(placement: Placement) -> Corner {
    match placement {
        Placement::TopLeft => Corner::TopLeft,
        Placement::TopRight => Corner::TopRight,
        Placement::BottomLeft => Corner::BottomLeft,
        Placement::BottomRight => Corner::BottomRight,
    }
}

impl Hooks for Mod {
    fn mods(&self, _host: &mut dyn Host, event: &mut ModsEvent<'_>) {
        push_row(event);
    }

    /// The panel, drawn into the HUD batch the game hands every mod.
    fn hud(&self, host: &mut dyn Host, event: &mut HudEvent<'_>) {
        let settings = read_settings(&*host);
        if !settings.enable_mod {
            return;
        }
        let Some(camera) = host.camera() else {
            return;
        };
        let rows = lines(&settings, camera.position, camera.yaw, &camera.dimension);
        if rows.is_empty() {
            return;
        }
        const LINE: f32 = 10.0;
        const PAD: f32 = 2.0;
        let width = rows
            .iter()
            .map(|row| host.text_width(row))
            .fold(0.0f32, f32::max)
            + PAD * 2.0;
        let height = rows.len() as f32 * LINE + PAD * 2.0;
        let (x, y) = event.corners.place(corner(settings.placement), width, height);
        host.fill(event.batch, [x, y, width, height], 0x8000_0000);
        for (i, row) in rows.iter().enumerate() {
            host.text(
                event.batch,
                row,
                [x + PAD, y + PAD + i as f32 * LINE],
                0xFFFFFF,
                true,
            );
        }
    }

    fn settings(&self, host: &mut dyn Host, event: &mut SettingsEvent<'_>) {
        if event.mod_id != ID {
            return;
        }
        event.buttons.extend(read_settings(&*host).labels());
    }

    fn settings_click(&self, host: &mut dyn Host, event: &mut SettingsClickEvent<'_>) {
        if event.mod_id != ID {
            return;
        }
        let mut settings = read_settings(&*host);
        settings.click(event.button);
        for (key, value) in settings.stored() {
            host.set_setting(ID, key, &value);
        }
    }
}
