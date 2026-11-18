// The mod's settings page: three buttons mirroring the original ModConfig.
fn chattimestamps_page(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    use crate::game::chattimestamps as m;
    if mod_id != m::ID {
        return;
    }
    let game = &*game;
    let settings = m::settings(|key| game.native_setting(m::ID, key));
    buttons.extend(settings.labels());
}
