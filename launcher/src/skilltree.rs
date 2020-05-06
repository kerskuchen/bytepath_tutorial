use ct_lib::audio::*;
use ct_lib::draw::*;
use ct_lib::game::*;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Skilltree Scene

#[derive(Clone)]
pub struct SceneSkilltree {}

impl SceneSkilltree {
    pub fn new() -> SceneSkilltree {
        SceneSkilltree {}
    }
}

impl Scene for SceneSkilltree {
    fn update_and_draw(
        &mut self,
        _draw: &mut Drawstate,
        _audio: &mut Audiostate,
        _assets: &mut GameAssets,
        _input: &GameInput,
        _globals: &mut Globals,
        _out_game_events: &mut Vec<GameEvent>,
    ) {
    }
}
