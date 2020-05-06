use ct_lib::audio::*;
use ct_lib::draw::*;
use ct_lib::game::*;
use ct_lib::math::*;
use ct_lib::random::*;
use ct_platform;

use console::SceneConsole;
use skilltree::SceneSkilltree;
use stage::SceneStage;

mod main_launcher_info;

mod console;
mod skilltree;
mod stage;

const CANVAS_WIDTH: f32 = 480.0;
const CANVAS_HEIGHT: f32 = 270.0;

const WINDOW_CONFIG: WindowConfig = WindowConfig {
    has_canvas: true,
    canvas_width: CANVAS_WIDTH as u32,
    canvas_height: CANVAS_HEIGHT as u32,
    canvas_color_letterbox: Color::black(),

    windowed_mode_allow: true,
    windowed_mode_allow_resizing: true,

    grab_input: false,

    color_clear: Color::black(),
};

#[derive(Clone)]
pub struct GameState {
    globals: Globals,
    debug_deltatime_factor: f32,

    scene_current: String,

    scene_debug: SceneDebug,
    scene_stage: SceneStage,
    scene_console: SceneConsole,
    scene_skilltree: SceneSkilltree,
}

impl GameStateInterface for GameState {
    fn get_game_config() -> GameInfo {
        GameInfo {
            game_window_title: main_launcher_info::LAUNCHER_WINDOW_TITLE.to_owned(),
            game_save_folder_name: main_launcher_info::LAUNCHER_SAVE_FOLDER_NAME.to_owned(),
            game_company_name: main_launcher_info::LAUNCHER_COMPANY_NAME.to_owned(),
        }
    }
    fn get_window_config() -> WindowConfig {
        WINDOW_CONFIG
    }
    fn new(
        draw: &mut Drawstate,
        audio: &mut Audiostate,
        assets: &mut GameAssets,
        input: &GameInput,
    ) -> GameState {
        let random = Random::new_from_seed((input.deltatime * 1000000.0) as u64);

        let camera = GameCamera::new(Vec2::zero(), CANVAS_WIDTH, CANVAS_HEIGHT);

        let cursors = Cursors::new(
            &camera.cam,
            &input.mouse,
            &input.touch,
            input.screen_framebuffer_width,
            input.screen_framebuffer_height,
            CANVAS_WIDTH as u32,
            CANVAS_HEIGHT as u32,
        );

        let font_default = draw.get_font("default_tiny_bordered").clone();
        let font_default_no_border = draw.get_font("default_tiny").clone();

        let mut globals = Globals {
            random,
            camera,
            cursors,

            deltatime_speed_factor: 1.0,
            deltatime: input.deltatime,
            is_paused: false,

            canvas_width: CANVAS_WIDTH,
            canvas_height: CANVAS_HEIGHT,

            font_default,
            font_default_no_border,
        };

        let scene_debug = SceneDebug::new(draw, audio, assets, input, "Grand9K_Pixel_bordered");
        let scene_stage = SceneStage::new(draw, audio, assets, input, &mut globals);
        let scene_console = SceneConsole::new();
        let scene_skilltree = SceneSkilltree::new();

        GameState {
            globals,

            debug_deltatime_factor: 1.0,

            scene_current: "stage".to_string(),

            scene_debug,
            scene_stage,
            scene_console,
            scene_skilltree,
        }
    }

    fn update(
        &mut self,
        draw: &mut Drawstate,
        audio: &mut Audiostate,
        assets: &mut GameAssets,
        input: &GameInput,
    ) {
        if input.keyboard.recently_pressed(Scancode::F5) {
            *self = GameState::new(draw, audio, assets, input);
        }

        self.globals.cursors = Cursors::new(
            &self.globals.camera.cam,
            &input.mouse,
            &input.touch,
            input.screen_framebuffer_width,
            input.screen_framebuffer_height,
            CANVAS_WIDTH as u32,
            CANVAS_HEIGHT as u32,
        );

        // DEBUG GAMESPEED MANIPULATION
        //
        if !is_effectively_zero(self.debug_deltatime_factor - 1.0) {
            draw.debug_log(format!("Timefactor: {:.1}", self.debug_deltatime_factor));
        }
        if input.keyboard.recently_pressed(Scancode::KpPlus) {
            self.debug_deltatime_factor += 0.1;
        }
        if input.keyboard.recently_pressed(Scancode::KpMinus) {
            self.debug_deltatime_factor -= 0.1;
            if self.debug_deltatime_factor < 0.1 {
                self.debug_deltatime_factor = 0.1;
            }
        }
        if input.keyboard.recently_pressed(Scancode::Space) {
            self.globals.is_paused = !self.globals.is_paused;
        }
        let mut deltatime = input.target_deltatime * self.debug_deltatime_factor;
        if self.globals.is_paused {
            if input.keyboard.recently_pressed_or_repeated(Scancode::N) {
                deltatime = input.target_deltatime * self.debug_deltatime_factor;
            } else {
                deltatime = 0.0;
            }
        }
        self.globals.deltatime = deltatime * self.globals.deltatime_speed_factor;

        let mouse_coords = self.globals.cursors.mouse_coords;
        game_handle_mouse_camera_zooming_panning(
            &mut self.globals.camera,
            &input.mouse,
            &mouse_coords,
        );

        let mut game_events = Vec::new();
        match self.scene_current.as_str() {
            "stage" => self.scene_stage.update_and_draw(
                draw,
                audio,
                assets,
                input,
                &mut self.globals,
                &mut game_events,
            ),
            "console" => self.scene_console.update_and_draw(
                draw,
                audio,
                assets,
                input,
                &mut self.globals,
                &mut game_events,
            ),
            "skilltree" => self.scene_skilltree.update_and_draw(
                draw,
                audio,
                assets,
                input,
                &mut self.globals,
                &mut game_events,
            ),
            "debug" => self.scene_debug.update_and_draw(
                draw,
                audio,
                assets,
                input,
                &mut self.globals,
                &mut game_events,
            ),
            _ => panic!("Unknown scene '{}'", self.scene_current),
        }

        let deltatime = self.globals.deltatime;
        self.globals.camera.update(deltatime);
        draw.set_shaderparams_simple(Color::white(), self.globals.camera.proj_view_matrix());

        for event in game_events {
            match event {
                GameEvent::SwitchToScene { scene_name } => {
                    match scene_name.as_str() {
                        "stage" => {
                            self.scene_stage =
                                SceneStage::new(draw, audio, assets, input, &mut self.globals);
                        }
                        "console" => {
                            self.scene_console = SceneConsole::new();
                        }
                        "skilltree" => {
                            self.scene_skilltree = SceneSkilltree::new();
                        }
                        _ => panic!("Unknown scene '{}'", scene_name),
                    };
                    self.scene_current = scene_name;
                }
            }
        }
    }
}

fn main() {
    ct_platform::run_main::<GameState>();
}
