use ct_lib::audio::*;
use ct_lib::draw::*;
use ct_lib::game::*;
use ct_lib::math::*;
use ct_lib::random::*;
use ct_lib::*;

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
    fn new_player(pos: Vec2, ship_type: ShipType) -> (Transform, Motion, Drawable, Player) {
        let player_size = 12.0;
        (
            Transform {
                pos,
                dir_angle: -90.0,
            },
            Motion {
                vel: Vec2::zero(),
                acc: Vec2::zero(),
                dir_angle_vel: 0.0,
                dir_angle_acc: 0.0,
            },
            Drawable {
                mesh: MeshType::Linestrip(get_draw_lines_for_ship(ship_type)),
                pos_offset: Vec2::zero(),
                scale: Vec2::filled(player_size) / 4.0,
                color: COLOR_DEFAULT,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_PLAYER,
                add_jitter: true,
            },
            Player {
                attack_timer: TriggerRepeating::new(0.24),
                timer_tick: TriggerRepeating::new(5.0),
                timer_trail_particles: TriggerRepeating::new(0.01),

                ship_type,

                speed: 0.0,
                speed_max: 100.0,
                speed_base_max: 100.0,
                acc: 100.0,

                turn_speed: 1.66 * 180.0,

                width: player_size,
                height: player_size,

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

    pub fn new_trailparticle(
        pos: Vec2,
        size: f32,
        lifetime: f32,
        color: Color,
    ) -> (Transform, TrailParticle, Drawable) {
        (
            Transform {
                pos,
                dir_angle: 0.0,
            },
            TrailParticle {
                timer_tween: TimerSimple::new_started(lifetime),
                size,
                color,
            },
            Drawable {
                mesh: MeshType::Circle {
                    radius: size,
                    filled: true,
                },
                pos_offset: Vec2::zero(),
                scale: Vec2::ones(),
                depth: DEPTH_EFFECTS,
                color,
                additivity: ADDITIVITY_NONE,
                add_jitter: false,
            },
        )
    }

    pub fn new_projectile(pos: Vec2, dir: Vec2) -> (Transform, Motion, Projectile, Drawable) {
        let projectile_size = 2.5;
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
            Projectile {
                size: projectile_size,
            },
            Drawable {
                mesh: MeshType::Circle {
                    radius: projectile_size,
                    filled: false,
                },
                pos_offset: Vec2::zero(),
                scale: Vec2::ones(),
                add_jitter: false,
                depth: DEPTH_PROJECTILE,
                color: COLOR_DEFAULT,
                additivity: ADDITIVITY_NONE,
            },
        )
    }

    pub fn new_ammo(
        pos: Vec2,
        vel: Vec2,
        dir_angle: f32,
        dir_angle_vel: f32,
        player_entity: Entity,
    ) -> (Transform, Motion, Ammo, TurnTowardsTarget, Drawable) {
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
                width: size,
                height: size,

                color: COLOR_AMMO,
            },
            TurnTowardsTarget {
                target: player_entity,
                follow_precision_percent: 0.1,
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
    ) -> (Transform, HitEffect, Drawable) {
        (
            Transform { pos, dir_angle },
            HitEffect {
                timer_stages: TimerSimple::new_started(0.25),
                size,
                color_first_stage,
                color_second_stage,
            },
            Drawable {
                mesh: MeshType::RectangleTransformed {
                    width: size,
                    height: size,
                    filled: true,
                    centered: true,
                },
                pos_offset: Vec2::zero(),
                scale: Vec2::ones(),
                color: color_first_stage,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_EFFECTS,
                add_jitter: false,
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
    ) -> (Transform, Motion, ExplodeParticle, Drawable) {
        let dir = Vec2::from_angle_flipped_y(deg_to_rad(dir_angle));
        (
            Transform { pos, dir_angle },
            Motion {
                vel: speed * dir,
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
            Drawable {
                mesh: MeshType::LineWithThickness {
                    start: pos,
                    end: pos + length * dir,
                    thickness: thickness,
                    smooth_edges: false,
                },
                pos_offset: Vec2::zero(),
                scale: Vec2::ones(),
                depth: DEPTH_EFFECTS,
                color: color,
                additivity: ADDITIVITY_NONE,
                add_jitter: false,
            },
        )
    }

    fn new_tick_effect(player_entity: Entity) -> (Transform, SnapToParent, TickEffect, Drawable) {
        let width = 32.0;
        let height = 48.0;
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
                width,
                height,
            },
            Drawable {
                mesh: MeshType::Rectangle {
                    width,
                    height,
                    filled: true,
                    centered: true,
                },
                pos_offset: Vec2::zero(),
                scale: Vec2::ones(),
                depth: DEPTH_EFFECTS,
                color: COLOR_DEFAULT,
                additivity: ADDITIVITY_NONE,
                add_jitter: false,
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
struct TurnTowardsTarget {
    pub target: Entity,
    pub follow_precision_percent: f32,
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
    LineWithThickness {
        start: Vec2,
        end: Vec2,
        thickness: f32,
        smooth_edges: bool,
    },
    Linestrip(Vec<Vec<(i32, i32)>>),
}

#[derive(Debug, Clone)]
struct Drawable {
    mesh: MeshType,
    pos_offset: Vec2,
    scale: Vec2,
    depth: Depth,
    color: Color,
    additivity: Additivity,
    add_jitter: bool,
}

fn linestrip_transform<CoordType>(
    linestrip: &[CoordType],
    pos: Vec2,
    pivot: Vec2,
    scale: Vec2,
    dir: Vec2,
    jitter: Option<&mut Random>,
) -> Vec<Vec2>
where
    CoordType: Into<Vec2> + Copy + Clone,
{
    if let Some(random) = jitter {
        linestrip
            .iter()
            .map(|&point| {
                random.vec2_in_unit_rect()
                    + Vec2::from(point.into()).transformed(pivot, pos, scale, dir)
            })
            .collect()
    } else {
        linestrip
            .iter()
            .map(|&point| Vec2::from(point.into()).transformed(pivot, pos, scale, dir))
            .collect()
    }
}

/*
fn linestrip_transform(
    linestrip: &[(i32, i32)],
    pos: Vec2,
    pivot: Vec2,
    scale: Vec2,
    dir: Vec2,
    jitter: Option<&mut Random>,
) -> Vec<Vec2> {
    if let Some(random) = jitter {
        linestrip
            .iter()
            .map(|&point| {
                random.vec2_in_unit_rect() + Vec2::from(point).transformed(pivot, pos, scale, dir)
            })
            .collect()
    } else {
        linestrip
            .iter()
            .map(|&point| Vec2::from(point).transformed(pivot, pos, scale, dir))
            .collect()
    }
}
*/

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

fn get_shoot_points_for_ship(ship_type: ShipType) -> Vec<(i32, i32)> {
    match ship_type {
        ShipType::Fighter => vec![(4, 0)],
        ShipType::Rogue => vec![(4, 0)],
        ShipType::Sorcerer => vec![(5, 0)],
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

        let player_pos = Vec2::new(globals.canvas_width, globals.canvas_height) / 2.0;
        let player = world.spawn(Archetypes::new_player(player_pos, ShipType::Sorcerer));

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
        // UPDATE POSITIONS

        // MOTION
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

        // SNAPPING
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
        // STEERING

        // TURNING TOWARDS TARGET
        for (_entity, (xform, motion, follow)) in
            &mut self
                .world
                .query::<(&Transform, &mut Motion, &TurnTowardsTarget)>()
        {
            if self.world.contains(follow.target) {
                let target_xform = self.world.get::<Transform>(follow.target).unwrap();

                let dir_current = motion.vel.normalized();
                let dir_target = (target_xform.pos - xform.pos).normalized();
                let dir_final =
                    Vec2::lerp(dir_current, dir_target, follow.follow_precision_percent)
                        .normalized();
                motion.vel = motion.vel.magnitude() * dir_final;
            }
        }

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

            // STEERING
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

            let player_pos = player_xform.pos;
            let player_scale = Vec2::filled(player.width) / 4.0;

            // SHOOTING
            if player.attack_timer.update(player.attack_speed * deltatime) {
                let shoot_points_relative: Vec<Vec2> = linestrip_transform(
                    &get_shoot_points_for_ship(player.ship_type),
                    Vec2::zero(),
                    Vec2::zero(),
                    player_scale,
                    Vec2::unit_x(),
                    None,
                );
                let shoot_points: Vec<Vec2> = linestrip_transform(
                    &get_shoot_points_for_ship(player.ship_type),
                    player_pos,
                    Vec2::zero(),
                    player_scale,
                    player_dir,
                    None,
                );

                let muzzle_pos_relative = shoot_points_relative.first().cloned().unwrap();
                let muzzle_pos = shoot_points.first().cloned().unwrap();

                self.commands.add_entity(Archetypes::new_muzzleflash(
                    player_entity,
                    muzzle_pos_relative,
                    45.0,
                ));
                self.commands
                    .add_entity(Archetypes::new_projectile(muzzle_pos, player_dir));
            }

            // EXHAUST PARTICLES
            if player.timer_trail_particles.update(deltatime) {
                let exhaust_points: Vec<Vec2> = linestrip_transform(
                    &get_exhaust_points_for_ship(player.ship_type),
                    player_pos,
                    Vec2::zero(),
                    player_scale,
                    player_dir,
                    None,
                );

                for &point in &exhaust_points {
                    let size = globals.random.f32_in_range_closed(2.0, 4.0);
                    let lifetime = globals.random.f32_in_range_closed(0.15, 0.25);
                    let color = if boost {
                        COLOR_BOOST
                    } else {
                        COLOR_SKILL_POINT
                    };

                    self.commands
                        .add_entity(Archetypes::new_trailparticle(point, size, lifetime, color));
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

        for (particle_entity, (particle_xform, particle_motion, particle, drawable)) in &mut self
            .world
            .query::<(&Transform, &mut Motion, &mut ExplodeParticle, &mut Drawable)>()
        {
            particle.timer_tween.update(deltatime);
            if particle.timer_tween.is_finished() {
                self.commands.remove_entity(particle_entity);
            }

            let percentage = particle.timer_tween.completion_ratio();
            let length = lerp(particle.length, 0.0, percentage);
            let speed = lerp(particle.speed, 0.0, percentage);
            let dir = Vec2::from_angle_flipped_y(deg_to_rad(particle_xform.dir_angle));

            particle_motion.vel =
                speed * Vec2::from_angle_flipped_y(deg_to_rad(particle_xform.dir_angle));

            drawable.mesh = MeshType::LineWithThickness {
                start: particle_xform.pos.pixel_snapped(),
                end: particle_xform.pos.pixel_snapped() + length * dir,
                thickness: particle.thickness,
                smooth_edges: false,
            };
        }

        //------------------------------------------------------------------------------------------
        // UPDATE TICKEFFECT

        for (effect_entity, (effect, drawable)) in
            &mut self.world.query::<(&mut TickEffect, &mut Drawable)>()
        {
            effect.timer_tween.update(deltatime);
            if effect.timer_tween.is_finished() {
                self.commands.remove_entity(effect_entity);
            }

            let percentage = easing::cubic_inout(effect.timer_tween.completion_ratio());
            let width = effect.width;
            let height = lerp(effect.height, 0.0, percentage);
            let offset_y = lerp(0.0, -effect.height / 2.0, percentage);
            drawable.mesh = MeshType::Rectangle {
                width: width,
                height: height,
                filled: true,
                centered: true,
            };
            drawable.pos_offset = Vec2::filled_y(offset_y);
        }

        //------------------------------------------------------------------------------------------
        // UPDATE TRAILPARTICLES

        for (particle_entity, (particle, drawable)) in
            &mut self.world.query::<(&mut TrailParticle, &mut Drawable)>()
        {
            particle.timer_tween.update(deltatime);
            if particle.timer_tween.is_finished() {
                self.commands.remove_entity(particle_entity);
            }

            let percentage = particle.timer_tween.completion_ratio();
            let radius = lerp(particle.size, 0.0, percentage);
            drawable.mesh = MeshType::Circle {
                radius,
                filled: true,
            };
        }

        //------------------------------------------------------------------------------------------
        // UPDATE PROJECTILES

        for (projectile_entity, (projectile_xform, projectile)) in
            &mut self.world.query::<(&Transform, &mut Projectile)>()
        {
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

        for (effect_entity, (effect, drawable)) in
            &mut self.world.query::<(&mut HitEffect, &mut Drawable)>()
        {
            effect.timer_stages.update(deltatime);
            if effect.timer_stages.is_finished() {
                self.commands.remove_entity(effect_entity);
            }

            let color = if effect.timer_stages.time_cur < 0.1 {
                effect.color_first_stage
            } else {
                effect.color_second_stage
            };
            drawable.color = color;
        }

        //------------------------------------------------------------------------------------------
        // UPDATE AMMO

        for (ammo_entity, (ammo_xform, ammo_motion, ammo, turn_to_target)) in &mut self
            .world
            .query::<(&Transform, &mut Motion, &mut Ammo, &TurnTowardsTarget)>()
        {
            // Check if entity needs to be removed from game
            let mut remove_self = false;
            let canvas_rect = Rect::from_width_height(globals.canvas_width, globals.canvas_height);
            if !canvas_rect.contains_point(ammo_xform.pos) {
                remove_self = true;
            }
            if self.world.contains(turn_to_target.target) {
                let player_xform = self.world.get::<Transform>(turn_to_target.target).unwrap();
                if Vec2::distance_squared(player_xform.pos, ammo_xform.pos) < squared(ammo.width) {
                    remove_self = true;
                }
            }

            if remove_self {
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
                    if scale.x != scale.y {
                        todo!();
                    }
                    if *filled {
                        draw.draw_circle_filled(
                            xform.pos,
                            scale.x * *radius,
                            depth,
                            color,
                            additivity,
                        );
                    } else {
                        draw.draw_circle_bresenham(
                            xform.pos,
                            scale.x * *radius,
                            depth,
                            color,
                            additivity,
                        );
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

                        let linestrip: Vec<Vec2> =
                            linestrip_transform(&rect.linestrip(), pos, pivot, scale, dir, None);
                        draw.draw_linestrip_bresenham(&linestrip, depth, color, additivity);
                    }
                }
                MeshType::Linestrip(linestrips) => {
                    for linestrip_raw in linestrips {
                        let jitter = if drawable.add_jitter {
                            Some(&mut globals.random)
                        } else {
                            None
                        };
                        let linestrip: Vec<Vec2> =
                            linestrip_transform(linestrip_raw, pos, pivot, scale, dir, jitter);
                        draw.draw_linestrip_bresenham(
                            &linestrip,
                            DEPTH_PLAYER,
                            COLOR_DEFAULT,
                            ADDITIVITY_NONE,
                        );
                    }
                }
                MeshType::LineWithThickness {
                    start,
                    end,
                    thickness,
                    smooth_edges,
                } => {
                    draw.draw_line_with_thickness(
                        *start,
                        *end,
                        *thickness,
                        *smooth_edges,
                        depth,
                        color,
                        additivity,
                    );
                }
            };
        }

        //------------------------------------------------------------------------------------------
        // CREATE / DELETE ENTITIES
        self.commands.execute(&mut self.world);
    }
}
