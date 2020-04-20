use ct_lib::audio::*;
use ct_lib::draw::*;
use ct_lib::game::*;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Console Scene

#[derive(Clone)]
pub struct SceneConsole {}

impl SceneConsole {
    pub fn _new() -> SceneConsole {
        SceneConsole {}
    }
}
impl Scene for SceneConsole {
    fn update_and_draw(
        &mut self,
        _draw: &mut Drawstate,
        _audio: &mut Audiostate,
        _assets: &mut GameAssets,
        _input: &GameInput,
        _globals: &mut Globals,
    ) {
    }
}
