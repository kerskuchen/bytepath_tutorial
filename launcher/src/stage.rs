use ct_lib::audio::*;
use ct_lib::draw::*;
use ct_lib::game::*;
use ct_lib::math::*;
use ct_lib::random::*;

use hecs::*;

const DEPTH_BACKGROUND: Depth = 0.0;
const DEPTH_PLAYER: Depth = 10.0;
const DEPTH_PROJECTILE: Depth = 20.0;
const DEPTH_EFFECTS: Depth = 30.0;
const DEPTH_SCREENFLASH: Depth = 60.0;
const DEPTH_COLLECTIBLES: Depth = 15.0;

// TODO: When f32 gets const functions we can just use from_rgb_bytes instead of this monstrosity
const COLOR_DEFAULT: Color = Color::from_rgb(222.0 / 255.0, 222.0 / 255.0, 222.0 / 255.0);
const COLOR_BACKGROUND: Color = Color::from_rgb(16.0 / 255.0, 16.0 / 255.0, 16.0 / 255.0);
const COLOR_AMMO: Color = Color::from_rgb(123.0 / 255.0, 200.0 / 255.0, 164.0 / 255.0);
const COLOR_BOOST: Color = Color::from_rgb(76.0 / 255.0, 195.0 / 255.0, 217.0 / 255.0);
const COLOR_HP: Color = Color::from_rgb(241.0 / 255.0, 103.0 / 255.0, 69.0 / 255.0);
const COLOR_SKILL_POINT: Color = Color::from_rgb(255.0 / 255.0, 198.0 / 255.0, 93.0 / 255.0);

////////////////////////////////////////////////////////////////////////////////////////////////////
// Components

struct Archetypes {}

impl Archetypes {
    fn new_player(canvas_width: f32, canvas_height: f32) -> (Transform, Motion, Player) {
        (
            Transform {
                pos: Vec2::new(canvas_width / 2.0, canvas_height / 2.0),
                dir_angle: -90.0,
            },
            Motion {
                vel: Vec2::zero(),
                acc: Vec2::zero(),
                dir_angle_vel: 0.0,
                dir_angle_acc: 0.0,
            },
            Player {
                attack_timer: TriggerRepeating::new(0.24),
                timer_tick: TriggerRepeating::new(5.0),
                timer_trail_particles: TriggerRepeating::new(0.01),

                ship_type: ShipType::Sorcerer,

                speed: 0.0,
                speed_max: 100.0,
                speed_base_max: 100.0,
                acc: 100.0,

                turn_speed: 1.66 * 180.0,

                width: 12.0,
                height: 12.0,

                attack_speed: 2.0,

                hp: 100.0,
                hp_max: 100.0,

                ammo: 100.0,
                ammo_max: 100.0,

                boost: 100.0,
                boost_max: 100.0,
                boost_cooldown: 2.0,
                boost_allowed: true,
                boost_timer: TimerSimple::new_stopped(1.0),
            },
        )
    }

    fn new_screenflash(
        canvas_width: f32,
        canvas_height: f32,
    ) -> (Screenflash, Transform, Drawable) {
        (
            Screenflash {
                framecount_duration: 4,
            },
            Transform {
                pos: Vec2::zero(),
                dir_angle: 0.0,
            },
            Drawable {
                mesh: MeshType::Rectangle {
                    width: canvas_width,
                    height: canvas_height,
                    filled: true,
                    centered: false,
                },
                pos_offset: Vec2::zero(),
                scale: Vec2::ones(),
                color: COLOR_DEFAULT,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_SCREENFLASH,
                add_jitter: false,
            },
        )
    }

    pub fn new_muzzleflash(
        parent: Entity,
        pos_offset: Vec2,
        dir_angle_offset: f32,
    ) -> (Transform, Muzzleflash, SnapToParent, Drawable) {
        let initial_size = 8.0;
        (
            Transform {
                pos: Vec2::zero(),
                dir_angle: 0.0,
            },
            Muzzleflash {
                timer_tween: TimerSimple::new_started(0.1),
                size: initial_size,
            },
            SnapToParent {
                parent,
                pos_snap: true,
                pos_offset,
                dir_angle_snap: true,
                dir_angle_offset,
            },
            Drawable {
                mesh: MeshType::RectangleTransformed {
                    width: initial_size,
                    height: initial_size,
                    filled: true,
                    centered: true,
                },
                pos_offset: Vec2::zero(),
                scale: Vec2::ones(),
                color: COLOR_DEFAULT,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_EFFECTS,
                add_jitter: false,
            },
        )
    }

    pub fn new_projectile(pos: Vec2, dir: Vec2) -> (Transform, Motion, Projectile) {
        (
            Transform {
                pos,
                dir_angle: rad_to_deg(dir.to_angle_flipped_y()),
            },
            Motion {
                vel: 200.0 * dir,
                acc: Vec2::zero(),
                dir_angle_vel: 0.0,
                dir_angle_acc: 0.0,
            },
            Projectile { size: 2.5 },
        )
    }

    pub fn new_ammo(
        pos: Vec2,
        vel: Vec2,
        dir_angle: f32,
        dir_angle_vel: f32,
        player_entity: Entity,
    ) -> (Transform, Motion, Ammo, Drawable) {
        let size = 8.0;
        (
            Transform { pos, dir_angle },
            Motion {
                vel,
                acc: Vec2::zero(),
                dir_angle_vel,
                dir_angle_acc: 0.0,
            },
            Ammo {
                move_target: player_entity,

                width: size,
                height: size,

                color: COLOR_AMMO,
            },
            Drawable {
                mesh: MeshType::RectangleTransformed {
                    width: size,
                    height: size,
                    filled: false,
                    centered: true,
                },
                pos_offset: Vec2::zero(),
                scale: Vec2::ones(),
                color: COLOR_AMMO,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_COLLECTIBLES,
                add_jitter: false,
            },
        )
    }

    pub fn new_hit_effect(
        pos: Vec2,
        size: f32,
        dir_angle: f32,
        color_first_stage: Color,
        color_second_stage: Color,
    ) -> (Transform, HitEffect) {
        (
            Transform { pos, dir_angle },
            HitEffect {
                timer_stages: TimerSimple::new_started(0.25),
                size,
                color_first_stage,
                color_second_stage,
            },
        )
    }

    pub fn new_explode_particle(
        pos: Vec2,
        dir_angle: f32,
        speed: f32,
        thickness: f32,
        length: f32,
        lifetime: f32,
        color: Color,
    ) -> (Transform, Motion, ExplodeParticle) {
        (
            Transform { pos, dir_angle },
            Motion {
                vel: speed * Vec2::from_angle_flipped_y(deg_to_rad(dir_angle)),
                acc: Vec2::zero(),
                dir_angle_vel: 0.0,
                dir_angle_acc: 0.0,
            },
            ExplodeParticle {
                timer_tween: TimerSimple::new_started(lifetime),
                thickness,
                length,
                speed,
                color,
            },
        )
    }

    fn new_tick_effect(player_entity: Entity) -> (Transform, SnapToParent, TickEffect) {
        (
            Transform {
                pos: Vec2::zero(),
                dir_angle: 0.0,
            },
            SnapToParent {
                parent: player_entity,
                pos_snap: true,
                pos_offset: Vec2::zero(),
                dir_angle_snap: false,
                dir_angle_offset: 0.0,
            },
            TickEffect {
                timer_tween: TimerSimple::new_started(0.13),
                width: 32.0,
                height: 48.0,
            },
        )
    }
}

#[derive(Debug, Copy, Clone)]
enum ShipType {
    Fighter,
    Sorcerer,
    Rogue,
}

#[derive(Debug, Copy, Clone)]
struct Transform {
    pub pos: Vec2,
    /// Given in degrees [-360, 360] counterclockwise
    pub dir_angle: f32,
}

#[derive(Debug, Copy, Clone)]
struct Motion {
    pub vel: Vec2,
    pub acc: Vec2,

    /// Given in degrees [-360, 360] counterclockwise
    pub dir_angle_vel: f32,
    /// Given in degrees [-360, 360] counterclockwise
    pub dir_angle_acc: f32,
}

#[derive(Debug, Copy, Clone)]
struct SnapToParent {
    pub parent: Entity,

    pub pos_snap: bool,
    pub pos_offset: Vec2,

    pub dir_angle_snap: bool,
    pub dir_angle_offset: f32,
}

#[derive(Debug, Copy, Clone)]
struct Player {
    pub timer_tick: TriggerRepeating,
    pub timer_trail_particles: TriggerRepeating,

    pub ship_type: ShipType,

    pub speed: f32,
    pub speed_max: f32,
    pub speed_base_max: f32,
    pub acc: f32,

    pub turn_speed: f32,

    pub width: f32,
    pub height: f32,

    pub attack_timer: TriggerRepeating,
    pub attack_speed: f32,

    pub hp: f32,
    pub hp_max: f32,

    pub ammo: f32,
    pub ammo_max: f32,

    pub boost: f32,
    pub boost_max: f32,
    pub boost_cooldown: f32,
    pub boost_allowed: bool,
    pub boost_timer: TimerSimple,
}

#[derive(Debug, Copy, Clone)]
struct Ammo {
    pub move_target: Entity,

    pub width: f32,
    pub height: f32,

    pub color: Color,
}

#[derive(Debug, Copy, Clone)]
struct SlowMotion {
    pub timer_tween: TimerSimple,
    pub deltatime_speed_factor: f32,
}

#[derive(Debug, Copy, Clone)]
struct Screenflash {
    pub framecount_duration: usize,
}

#[derive(Debug, Copy, Clone)]
struct TickEffect {
    pub timer_tween: TimerSimple,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Copy, Clone)]
struct Muzzleflash {
    pub timer_tween: TimerSimple,
    pub size: f32,
}

#[derive(Debug, Copy, Clone)]
struct Projectile {
    pub size: f32,
}

#[derive(Debug, Copy, Clone)]
struct HitEffect {
    pub timer_stages: TimerSimple,
    pub size: f32,
    pub color_first_stage: Color,
    pub color_second_stage: Color,
}

#[derive(Debug, Copy, Clone)]
struct ExplodeParticle {
    pub timer_tween: TimerSimple,
    pub thickness: f32,
    pub length: f32,
    pub speed: f32,
    pub color: Color,
}

#[derive(Debug, Copy, Clone)]
struct TrailParticle {
    pub timer_tween: TimerSimple,
    pub size: f32,
    pub color: Color,
}

#[derive(Debug, Clone)]
enum MeshType {
    Circle {
        radius: f32,
        filled: bool,
    },
    Rectangle {
        width: f32,
        height: f32,
        filled: bool,
        centered: bool,
    },
    RectangleTransformed {
        width: f32,
        height: f32,
        filled: bool,
        centered: bool,
    },
    Lines(Vec<Vec<(i32, i32)>>),
}

#[derive(Debug, Clone)]
struct Drawable {
    mesh: MeshType,
    pos_offset: Vec2,
    scale: Vec2,
    color: Color,
    additivity: Additivity,
    depth: Depth,
    add_jitter: bool,
}

fn get_draw_lines_for_ship(ship_type: ShipType) -> Vec<Vec<(i32, i32)>> {
    match ship_type {
        ShipType::Fighter => {
            let hull = vec![
                (4, 0),
                (2, -2),
                (-2, -2),
                (-3, -1),
                (-4, 0),
                (-3, 1),
                (-2, 2),
                (2, 2),
                (4, 0),
            ];
            let wing_left = vec![(2, -2), (0, -4), (-6, -4), (-3, -1)];
            let wing_right = vec![(2, 2), (0, 4), (-6, 4), (-3, 1)];
            vec![hull, wing_left, wing_right]
        }
        ShipType::Rogue => {
            let hull = vec![(4, 0), (-2, -2), (-4, 0), (-2, 2), (4, 0)];
            let wing_left = vec![(2, -1), (1, -3), (-7, -7), (-2, -2)];
            let wing_right = vec![(2, 1), (1, 3), (-7, 7), (-2, 2)];
            vec![hull, wing_left, wing_right]
        }
        ShipType::Sorcerer => {
            let hull = vec![(5, 0), (2, -2), (-4, 0), (2, 2), (5, 0)];
            let wing_left = vec![(2, -2), (5, -7), (-1, -4), (-4, 0)];
            let wing_right = vec![(2, 2), (5, 7), (-1, 4), (-4, 0)];
            vec![hull, wing_left, wing_right]
        }
    }
}

fn get_exhaust_points_for_ship(ship_type: ShipType) -> Vec<(i32, i32)> {
    match ship_type {
        ShipType::Fighter => vec![(-3, -1), (-3, 1)],
        ShipType::Rogue => vec![(-3, -2), (-3, 2)],
        ShipType::Sorcerer => vec![(-4, 0)],
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// World command buffer

struct WorldCommandBuffer {
    commands: Vec<Box<dyn FnOnce(&mut World) + Send + Sync>>,
}

impl WorldCommandBuffer {
    fn new() -> WorldCommandBuffer {
        WorldCommandBuffer {
            commands: Vec::new(),
        }
    }

    fn add_entity<ComponentsBundleType>(&mut self, components: ComponentsBundleType)
    where
        ComponentsBundleType: hecs::DynamicBundle + Send + Sync + 'static,
    {
        self.commands.push(Box::new(move |world| {
            world.spawn(components);
        }));
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.commands.push(Box::new(move |world| {
            world.despawn(entity).ok();
        }));
    }

    fn execute(&mut self, world: &mut World) {
        for command in self.commands.drain(..) {
            command(world);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Stage Scene

pub struct SceneStage {
    world: World,
    commands: WorldCommandBuffer,
    player: Option<Entity>,
}

impl Clone for SceneStage {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl SceneStage {
    pub fn new(
        _draw: &mut Drawstate,
        _audio: &mut Audiostate,
        _assets: &mut GameAssets,
        _input: &GameInput,
        globals: &mut Globals,
    ) -> SceneStage {
        let mut world = World::new();

        let player = world.spawn(Archetypes::new_player(
            globals.canvas_width,
            globals.canvas_height,
        ));

        SceneStage {
            world,
            player: Some(player),
            commands: WorldCommandBuffer::new(),
        }
    }
}

impl Scene for SceneStage {
    fn update_and_draw(
        &mut self,
        draw: &mut Drawstate,
        _audio: &mut Audiostate,
        _assets: &mut GameAssets,
        input: &GameInput,
        globals: &mut Globals,
    ) {
        draw.set_clear_color_and_depth(COLOR_BACKGROUND, DEPTH_BACKGROUND);

        if input.keyboard.recently_pressed(Scancode::S) {
            let screen_shake = ModulatorScreenShake::new(&mut globals.random, 4.0, 1.0, 60.0);
            globals.camera.add_shake(screen_shake);
        }

        let deltatime = globals.deltatime;

        //------------------------------------------------------------------------------------------
        // SPAWN AMMO
        if input.keyboard.recently_pressed(Scancode::A) {
            if let Some(player_entity) = self.player {
                self.world.spawn(Archetypes::new_ammo(
                    globals.random.vec2_in_rect(Rect::from_width_height(
                        globals.canvas_width,
                        globals.canvas_height,
                    )),
                    globals.random.vec2_in_unit_disk()
                        * globals.random.f32_in_range_closed(10.0, 20.0),
                    globals.random.f32_in_range_closed(0.0, 360.0),
                    globals.random.f32_in_range_closed(-360.0, 360.0),
                    player_entity,
                ));
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE SLOWMOTION

        for (slowmotion_entity, slowmotion) in &mut self.world.query::<&mut SlowMotion>() {
            slowmotion.timer_tween.update(deltatime);
            let percentage = slowmotion.timer_tween.completion_ratio();
            globals.deltatime_speed_factor =
                lerp(slowmotion.deltatime_speed_factor, 1.0, percentage);

            if slowmotion.timer_tween.is_finished() {
                self.commands.remove_entity(slowmotion_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE SCREENFLASH
        for (flash_entity, flash) in &mut self.world.query::<&mut Screenflash>() {
            if flash.framecount_duration > 0 {
                flash.framecount_duration -= 1;
            } else {
                self.commands.remove_entity(flash_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE PLAYER
        for (player_entity, (player_xform, player_motion, player)) in
            &mut self.world.query::<(&Transform, &mut Motion, &mut Player)>()
        {
            // BOOST
            let mut boost = false;
            player.speed_max = player.speed_base_max;
            if player.boost_allowed {
                if input.keyboard.is_down(Scancode::Up) {
                    player.speed_max = 1.5 * player.speed_base_max;
                    boost = true;
                }
                if input.keyboard.is_down(Scancode::Down) {
                    player.speed_max = 0.5 * player.speed_base_max;
                    boost = true;
                }
            } else {
                player.boost_timer.update(deltatime);
                if player.boost_timer.is_finished() {
                    player.boost_allowed = true;
                }
            }
            if boost {
                player.boost = f32::max(player.boost - 50.0 * deltatime, 0.0);
            } else {
                player.boost = f32::min(player.boost + 10.0 * deltatime, player.boost_max);
            }
            if player.boost == 0.0 {
                player.boost_allowed = false;
                player.boost_timer = TimerSimple::new_started(player.boost_cooldown);
            }

            // MOVEMENT
            player_motion.dir_angle_vel = 0.0;
            if input.keyboard.is_down(Scancode::Left) {
                player_motion.dir_angle_vel = player.turn_speed;
            }
            if input.keyboard.is_down(Scancode::Right) {
                player_motion.dir_angle_vel = -player.turn_speed;
            }
            player.speed = f32::min(player.speed + player.acc * deltatime, player.speed_max);
            let player_dir = Vec2::from_angle_flipped_y(deg_to_rad(player_xform.dir_angle));
            let player_speed = player.speed;

            player_motion.vel = player_speed * player_dir;

            // SHOOTING
            if player.attack_timer.update(player.attack_speed * deltatime) {
                let start_pos = player_xform.pos + player.width * player_dir;

                self.commands.add_entity(Archetypes::new_muzzleflash(
                    player_entity,
                    Vec2::new(player.width, 0.0),
                    45.0,
                ));
                self.commands
                    .add_entity(Archetypes::new_projectile(start_pos, player_dir));
            }

            // DRAWING
            let player_pos = player_xform.pos;
            let scale = Vec2::filled(player.width) / 4.0;
            let transform_into_player_coords = move |point: (i32, i32)| {
                Vec2::from(point).transformed(Vec2::zero(), player_pos, scale, player_dir)
            };
            let linestrips = get_draw_lines_for_ship(player.ship_type);
            for linestrip in &linestrips {
                let linestrip: Vec<Vec2> = linestrip
                    .iter()
                    .map(|&point| {
                        globals.random.vec2_in_unit_rect() + transform_into_player_coords(point)
                    })
                    .collect();
                draw.draw_linestrip_bresenham(
                    &linestrip,
                    DEPTH_PLAYER,
                    COLOR_DEFAULT,
                    ADDITIVITY_NONE,
                );
            }

            // EXHAUST PARTICLES
            if player.timer_trail_particles.update(deltatime) {
                let exhaust_points: Vec<Vec2> = get_exhaust_points_for_ship(player.ship_type)
                    .iter()
                    .map(|&point| transform_into_player_coords(point))
                    .collect();

                for &point in &exhaust_points {
                    self.commands.add_entity((
                        Transform {
                            pos: point,
                            dir_angle: 0.0,
                        },
                        TrailParticle {
                            timer_tween: TimerSimple::new_started(
                                globals.random.f32_in_range_closed(0.15, 0.25),
                            ),
                            size: globals.random.f32_in_range_closed(2.0, 4.0),
                            color: if boost {
                                COLOR_BOOST
                            } else {
                                COLOR_SKILL_POINT
                            },
                        },
                    ));
                }
            }

            // TICK EFFECT
            if player.timer_tick.update(deltatime) {
                self.commands
                    .add_entity(Archetypes::new_tick_effect(player_entity));
            }

            // EXPLODE
            let canvas_rect = Rect::from_width_height(globals.canvas_width, globals.canvas_height);
            if !canvas_rect.contains_point(player_xform.pos) {
                self.commands.remove_entity(player_entity);

                let screen_shake = ModulatorScreenShake::new(&mut globals.random, 6.0, 0.2, 80.0);
                globals.camera.add_shake(screen_shake);

                self.commands.add_entity((SlowMotion {
                    timer_tween: TimerSimple::new_started(1.0),
                    deltatime_speed_factor: 0.15,
                },));

                self.commands.add_entity(Archetypes::new_screenflash(
                    globals.canvas_width,
                    globals.canvas_height,
                ));

                for _ in 0..globals.random.gen_range(15, 25) {
                    self.commands.add_entity(Archetypes::new_explode_particle(
                        player_xform.pos,
                        rad_to_deg(globals.random.vec2_in_unit_disk().to_angle_flipped_y()),
                        globals.random.f32_in_range_closed(120.0, 300.0),
                        globals.random.f32_in_range_closed(1.0, 2.0),
                        globals.random.f32_in_range_closed(3.0, 15.0),
                        globals.random.f32_in_range_closed(0.3, 0.5),
                        COLOR_DEFAULT,
                    ));
                }
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE MUZZLEFLASH
        for (effect_entity, (effect, drawable)) in
            &mut self.world.query::<(&mut Muzzleflash, &mut Drawable)>()
        {
            effect.timer_tween.update(deltatime);
            if effect.timer_tween.is_finished() {
                self.commands.remove_entity(effect_entity);
            }

            let percentage = easing::cubic_inout(effect.timer_tween.completion_ratio());
            let width = lerp(effect.size, 0.0, percentage);
            drawable.mesh = MeshType::RectangleTransformed {
                width: width,
                height: width,
                filled: true,
                centered: true,
            };
        }

        //------------------------------------------------------------------------------------------
        // UPDATE EXPLODE PARTICLES
        for (particle_entity, (particle_xform, particle_motion, particle)) in &mut self
            .world
            .query::<(&Transform, &mut Motion, &mut ExplodeParticle)>()
        {
            particle.timer_tween.update(deltatime);
            let percentage = particle.timer_tween.completion_ratio();
            let length = lerp(particle.length, 0.0, percentage);
            let speed = lerp(particle.speed, 0.0, percentage);
            let dir = Vec2::from_angle_flipped_y(deg_to_rad(particle_xform.dir_angle));

            particle_motion.vel =
                speed * Vec2::from_angle_flipped_y(deg_to_rad(particle_xform.dir_angle));

            draw.draw_line_with_thickness(
                particle_xform.pos.pixel_snapped(),
                particle_xform.pos.pixel_snapped() + length * dir,
                particle.thickness,
                false,
                DEPTH_EFFECTS,
                particle.color,
                ADDITIVITY_NONE,
            );

            if particle.timer_tween.is_finished() {
                self.commands.remove_entity(particle_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE TICKEFFECT
        for (effect_entity, (effect_xform, effect)) in
            &mut self.world.query::<(&mut Transform, &mut TickEffect)>()
        {
            effect.timer_tween.update(deltatime);
            let percentage = easing::cubic_inout(effect.timer_tween.completion_ratio());
            let width = effect.width; //lerp(effect.width, effect.height, percentage);
            let height = lerp(effect.height, 0.0, percentage);
            let offset_y = lerp(0.0, -effect.height / 2.0, percentage);

            draw.draw_rect(
                Rect::from_pos_width_height(effect_xform.pos, width, height)
                    .centered()
                    .translated_by(Vec2::filled_y(offset_y)),
                DEPTH_EFFECTS,
                COLOR_DEFAULT,
                ADDITIVITY_NONE,
            );

            if effect.timer_tween.is_finished() {
                self.commands.remove_entity(effect_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE TRAILPARTICLES
        for (particle_entity, (particle_xform, particle)) in
            &mut self.world.query::<(&Transform, &mut TrailParticle)>()
        {
            particle.timer_tween.update(deltatime);
            let percentage = particle.timer_tween.completion_ratio();
            let size = lerp(particle.size, 0.0, percentage);

            draw.draw_circle_filled(
                particle_xform.pos,
                size,
                DEPTH_EFFECTS,
                particle.color,
                ADDITIVITY_NONE,
            );

            if particle.timer_tween.is_finished() {
                self.commands.remove_entity(particle_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE PROJECTILES
        for (projectile_entity, (projectile_xform, projectile)) in
            &mut self.world.query::<(&Transform, &mut Projectile)>()
        {
            draw.draw_circle_bresenham(
                projectile_xform.pos,
                projectile.size,
                DEPTH_PROJECTILE,
                Color::white(),
                ADDITIVITY_NONE,
            );

            let canvas_rect = Rect::from_width_height(globals.canvas_width, globals.canvas_height);
            if !canvas_rect.contains_point(projectile_xform.pos) {
                self.commands.remove_entity(projectile_entity);

                self.commands.add_entity(Archetypes::new_hit_effect(
                    projectile_xform.pos.clamped_to_rect(canvas_rect),
                    3.0 * projectile.size,
                    0.0,
                    COLOR_DEFAULT,
                    COLOR_HP,
                ));
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE HIT EFFECTS
        for (effect_entity, (effect_xform, effect)) in
            &mut self.world.query::<(&Transform, &mut HitEffect)>()
        {
            effect.timer_stages.update(deltatime);
            let color = if effect.timer_stages.time_cur < 0.1 {
                effect.color_first_stage
            } else {
                effect.color_second_stage
            };

            draw.draw_rect_transformed(
                Vec2::filled(effect.size),
                Vec2::filled(effect.size / 2.0),
                effect_xform.pos,
                Vec2::ones(),
                Vec2::from_angle_flipped_y(deg_to_rad(effect_xform.dir_angle)),
                DEPTH_EFFECTS,
                color,
                ADDITIVITY_NONE,
            );

            if effect.timer_stages.is_finished() {
                self.commands.remove_entity(effect_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE AMMO

        for (ammo_entity, (ammo_xform, ammo_motion, ammo)) in
            &mut self.world.query::<(&Transform, &mut Motion, &mut Ammo)>()
        {
            if self.world.contains(ammo.move_target) {
                let player_xform = self.world.get::<Transform>(ammo.move_target).unwrap();

                let dir_current = ammo_motion.vel.normalized();
                let dir_target = (player_xform.pos - ammo_xform.pos).normalized();
                let dir_final = Vec2::lerp(dir_current, dir_target, 0.1).normalized();
                ammo_motion.vel = ammo_motion.vel.magnitude() * dir_final;
            }

            let canvas_rect = Rect::from_width_height(globals.canvas_width, globals.canvas_height)
                .extended_uniformly_by(ammo.width);
            if !canvas_rect.contains_point(ammo_xform.pos) {
                self.commands.remove_entity(ammo_entity);
            }

            if self.world.contains(ammo.move_target) {
                let player_xform = self.world.get::<Transform>(ammo.move_target).unwrap();

                if Vec2::distance_squared(player_xform.pos, ammo_xform.pos) < squared(ammo.width) {
                    self.commands.remove_entity(ammo_entity);

                    self.commands.add_entity(Archetypes::new_hit_effect(
                        ammo_xform.pos,
                        ammo.width,
                        45.0,
                        COLOR_DEFAULT,
                        COLOR_AMMO,
                    ));

                    for _ in 0..globals.random.gen_range(4, 8) {
                        self.commands.add_entity(Archetypes::new_explode_particle(
                            ammo_xform.pos,
                            rad_to_deg(globals.random.vec2_in_unit_disk().to_angle_flipped_y()),
                            globals.random.f32_in_range_closed(50.0, 100.0),
                            globals.random.f32_in_range_closed(1.0, 2.0),
                            globals.random.f32_in_range_closed(3.0, 8.0),
                            globals.random.f32_in_range_closed(0.3, 0.5),
                            COLOR_AMMO,
                        ));
                    }
                }
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE POSITIONS

        for (_entity, (xform, motion)) in &mut self.world.query::<(&mut Transform, &Motion)>() {
            xform.pos += motion.vel * deltatime;
            xform.dir_angle += motion.dir_angle_vel * deltatime;
            if xform.dir_angle > 360.0 {
                xform.dir_angle -= 360.0;
            }
            if xform.dir_angle < -360.0 {
                xform.dir_angle += 360.0;
            }
        }

        for (_entity, (xform, snap)) in &mut self.world.query::<(&mut Transform, &SnapToParent)>() {
            if self.world.contains(snap.parent) {
                let parent_xform = self.world.get::<Transform>(snap.parent).unwrap();
                if snap.pos_snap && snap.dir_angle_snap {
                    xform.pos = parent_xform.pos
                        + snap
                            .pos_offset
                            .rotated_flipped_y(deg_to_rad(parent_xform.dir_angle));
                    xform.dir_angle = parent_xform.dir_angle + snap.dir_angle_offset;
                } else if snap.pos_snap {
                    xform.pos = parent_xform.pos + snap.pos_offset;
                } else if snap.dir_angle_snap {
                    xform.dir_angle = parent_xform.dir_angle + snap.dir_angle_offset;
                }
            }
        }

        //------------------------------------------------------------------------------------------
        // DRAWING
        for (_entity, (xform, drawable)) in &mut self.world.query::<(&Transform, &Drawable)>() {
            let pos = xform.pos;
            let scale = drawable.scale;
            let dir = Vec2::from_angle_flipped_y(deg_to_rad(xform.dir_angle));
            let pivot = drawable.pos_offset;
            let depth = drawable.depth;
            let color = drawable.color;
            let additivity = drawable.additivity;
            match &drawable.mesh {
                MeshType::Circle { radius, filled } => {
                    if drawable.add_jitter {
                        todo!();
                    }
                    if *filled {
                        if scale.x != scale.y {
                            todo!();
                        }
                        draw.draw_circle_bresenham(
                            xform.pos,
                            scale.x * *radius,
                            depth,
                            color,
                            additivity,
                        );
                    } else {
                        todo!();
                    }
                }
                MeshType::Rectangle {
                    width,
                    height,
                    filled,
                    centered,
                } => {
                    if drawable.add_jitter {
                        todo!();
                    }

                    let rect = if *centered {
                        Rect::from_pos_width_height(pos, scale.x * *width, scale.y * *height)
                            .centered()
                            .translated_by(pivot)
                    } else {
                        Rect::from_pos_width_height(pos, scale.x * *width, scale.y * *height)
                            .translated_by(pivot)
                    };

                    if *filled {
                        draw.draw_rect(rect, depth, color, additivity);
                    } else {
                        draw.draw_linestrip_bresenham(
                            &rect.linestrip(),
                            DEPTH_COLLECTIBLES,
                            COLOR_AMMO,
                            ADDITIVITY_NONE,
                        );
                    }
                }
                MeshType::RectangleTransformed {
                    width,
                    height,
                    filled,
                    centered,
                } => {
                    if drawable.add_jitter {
                        todo!();
                    }

                    let center_offset = if *centered {
                        Vec2::new(*width, *height) / 2.0
                    } else {
                        Vec2::zero()
                    };

                    if *filled {
                        draw.draw_rect_transformed(
                            Vec2::new(*width, *height),
                            pivot + center_offset,
                            pos,
                            scale,
                            dir,
                            depth,
                            color,
                            additivity,
                        );
                    } else {
                        let rect = if *centered {
                            Rect::from_width_height(*width, *height).centered()
                        } else {
                            Rect::from_pos_width_height(pos, *width, *height)
                        };

                        let linestrip: Vec<Vec2> = rect
                            .linestrip()
                            .iter()
                            .map(|&point| point.transformed(pivot, pos, scale, dir))
                            .collect();
                        draw.draw_linestrip_bresenham(&linestrip, depth, color, additivity);
                    }
                }
                MeshType::Lines(linestrips) => {
                    for linestrip in linestrips {
                        let linestrip: Vec<Vec2> = if drawable.add_jitter {
                            linestrip
                                .iter()
                                .map(|&point| {
                                    globals.random.vec2_in_unit_rect()
                                        + Vec2::from(point).transformed(
                                            Vec2::zero(),
                                            pos,
                                            scale,
                                            dir,
                                        )
                                })
                                .collect()
                        } else {
                            linestrip
                                .iter()
                                .map(|&point| {
                                    Vec2::from(point).transformed(Vec2::zero(), pos, scale, dir)
                                })
                                .collect()
                        };
                        draw.draw_linestrip_bresenham(
                            &linestrip,
                            DEPTH_PLAYER,
                            COLOR_DEFAULT,
                            ADDITIVITY_NONE,
                        );
                    }
                }
            };
        }

        //------------------------------------------------------------------------------------------
        // CREATE / DELETE ENTITIES
        self.commands.execute(&mut self.world);
    }
}
