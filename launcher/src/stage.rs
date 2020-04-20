use ct_lib::audio::*;
use ct_lib::draw::*;
use ct_lib::game::*;
use ct_lib::math::*;
use ct_lib::random::*;

use std::collections::HashMap;

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

type Entity = u64;
type Storage<T> = HashMap<Entity, T>;

#[derive(Debug, Copy, Clone)]
enum ShipType {
    Fighter,
    Sorcerer,
    Rogue,
}

#[derive(Debug, Copy, Clone)]
struct Player {
    pub timer_tick: TriggerRepeating,
    pub timer_trail_particles: TriggerRepeating,

    pub ship_type: ShipType,

    pub pos: Vec2,
    pub speed: f32,
    pub speed_max: f32,
    pub speed_base_max: f32,
    pub acc: f32,

    pub angle: f32,
    pub angle_speed: f32,

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

impl Player {
    fn new(canvas_width: f32, canvas_height: f32) -> Player {
        Player {
            attack_timer: TriggerRepeating::new(0.24),
            timer_tick: TriggerRepeating::new(5.0),
            timer_trail_particles: TriggerRepeating::new(0.01),

            ship_type: ShipType::Sorcerer,

            pos: Vec2::new(canvas_width / 2.0, canvas_height / 2.0),
            speed: 0.0,
            speed_max: 100.0,
            speed_base_max: 100.0,
            acc: 100.0,

            angle: -90.0,
            angle_speed: 1.66 * 180.0,

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
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Ammo {
    pub move_target: Entity,

    pub width: f32,
    pub height: f32,

    pub pos: Vec2,
    pub vel: Vec2,

    pub look_angle: f32,
    pub look_angle_speed: f32,

    pub color: Color,
}

#[derive(Debug, Copy, Clone)]
struct SlowMotion {
    pub timer_tween: TimerSimple,
    pub deltatime_speed_factor: f32,
}

#[derive(Debug, Copy, Clone)]
struct Screenflash {
    pub color: Color,
    pub framecount_duration: usize,
}

#[derive(Debug, Copy, Clone)]
struct TickEffect {
    pub parent: Entity,
    pub timer_tween: TimerSimple,
    pub pos: Vec2,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Copy, Clone)]
struct Muzzleflash {
    pub parent: Entity,
    pub timer_tween: TimerSimple,
    pub pos: Vec2,
    pub angle: f32,
    pub size: f32,
}
impl Muzzleflash {
    pub fn new(parent: Entity) -> Muzzleflash {
        Muzzleflash {
            parent,
            timer_tween: TimerSimple::new_started(0.1),
            pos: Vec2::zero(),
            size: 8.0,
            angle: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Projectile {
    pub pos: Vec2,
    pub dir: Vec2,
    pub size: f32,
    pub speed: f32,
}
impl Projectile {
    pub fn new(pos: Vec2, dir: Vec2) -> Projectile {
        Projectile {
            pos,
            dir,
            size: 2.5,
            speed: 200.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct HitEffect {
    pub timer_stages: TimerSimple,
    pub pos: Vec2,
    pub size: f32,
    pub angle: f32,
    pub color_first_stage: Color,
    pub color_second_stage: Color,
}
impl HitEffect {
    pub fn new(
        pos: Vec2,
        size: f32,
        angle: f32,
        color_first_stage: Color,
        color_second_stage: Color,
    ) -> HitEffect {
        HitEffect {
            timer_stages: TimerSimple::new_started(0.25),
            pos,
            size,
            angle,
            color_first_stage,
            color_second_stage,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct ExplodeParticle {
    pub timer_tween: TimerSimple,
    pub pos: Vec2,
    pub dir: Vec2,
    pub speed: f32,
    pub thickness: f32,
    pub length: f32,
    pub color: Color,
}

#[derive(Debug, Copy, Clone)]
struct TrailParticle {
    pub timer_tween: TimerSimple,
    pub pos: Vec2,
    pub size: f32,
    pub color: Color,
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
// Stage Scene

#[derive(Clone, Default)]
pub struct SceneStage {
    entities: EntityAllocator,

    players: Storage<Player>,
    slowmotions: Storage<SlowMotion>,
    screenflashes: Storage<Screenflash>,
    tickeffects: Storage<TickEffect>,
    muzzleflashes: Storage<Muzzleflash>,
    projectiles: Storage<Projectile>,
    projectile_hits: Storage<HitEffect>,
    explode_particles: Storage<ExplodeParticle>,
    trail_particles: Storage<TrailParticle>,
    ammo_collectibles: Storage<Ammo>,
}

#[derive(Clone, Default)]
struct EntityAllocator {
    next_entity_index: Entity,
}

impl EntityAllocator {
    fn create(&mut self) -> Entity {
        self.next_entity_index += 1;
        self.next_entity_index
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
        let mut stage = SceneStage::default();

        let player_entity = stage.entities.create();
        stage.players.insert(
            player_entity,
            Player::new(globals.canvas_width, globals.canvas_height),
        );

        stage
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

        let mut entities_to_delete: Vec<Entity> = Vec::new();

        //------------------------------------------------------------------------------------------
        // SPAWN AMMO
        if input.keyboard.recently_pressed(Scancode::A) {
            let player_entity = self.players.keys().next().unwrap_or(&0);

            let ammo_entity = self.entities.create();
            self.ammo_collectibles.insert(
                ammo_entity,
                Ammo {
                    move_target: *player_entity,

                    width: 8.0,
                    height: 8.0,

                    pos: globals.random.vec2_in_rect(Rect::from_width_height(
                        globals.canvas_width,
                        globals.canvas_height,
                    )),
                    vel: globals.random.vec2_in_unit_disk()
                        * globals.random.f32_in_range_closed(10.0, 20.0),

                    look_angle: globals.random.f32_in_range_closed(0.0, 360.0),
                    look_angle_speed: globals.random.f32_in_range_closed(-360.0, 360.0),

                    color: COLOR_AMMO,
                },
            );
        }

        //------------------------------------------------------------------------------------------
        // UPDATE SLOWMOTION

        for (slowmotion_entity, slowmotion) in self.slowmotions.iter_mut() {
            slowmotion.timer_tween.update(deltatime);
            let percentage = slowmotion.timer_tween.completion_ratio();
            globals.deltatime_speed_factor =
                lerp(slowmotion.deltatime_speed_factor, 1.0, percentage);

            if slowmotion.timer_tween.is_finished() {
                entities_to_delete.push(*slowmotion_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE SCREENFLASH
        for (flash_entity, flash) in self.screenflashes.iter_mut() {
            if flash.framecount_duration > 0 {
                flash.framecount_duration -= 1;
                draw.draw_rect(
                    Rect::from_width_height(globals.canvas_width, globals.canvas_height),
                    DEPTH_SCREENFLASH,
                    flash.color,
                    ADDITIVITY_NONE,
                );
            } else {
                entities_to_delete.push(*flash_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE PLAYER
        for (player_entity, player) in self.players.iter_mut() {
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
            if input.keyboard.is_down(Scancode::Left) {
                player.angle += player.angle_speed * deltatime;
            }
            if input.keyboard.is_down(Scancode::Right) {
                player.angle -= player.angle_speed * deltatime;
            }
            if player.angle > 360.0 {
                player.angle -= 360.0;
            }
            if player.angle < -360.0 {
                player.angle += 360.0;
            }
            player.speed = f32::min(player.speed + player.acc * deltatime, player.speed_max);
            let player_dir = Vec2::from_angle_flipped_y(deg_to_rad(player.angle));
            let player_speed = player.speed;
            player.pos += player_speed * player_dir * deltatime;

            // SHOOTING
            if player.attack_timer.update(player.attack_speed * deltatime) {
                let muzzleflash_entity = self.entities.create();
                self.muzzleflashes
                    .insert(muzzleflash_entity, Muzzleflash::new(*player_entity));

                let start_pos = player.pos + player.width * player_dir;

                let projectile_entity = self.entities.create();
                self.projectiles
                    .insert(projectile_entity, Projectile::new(start_pos, player_dir));
            }

            // DRAWING
            let player_pos = player.pos;
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
                    let trail_entity = self.entities.create();
                    self.trail_particles.insert(
                        trail_entity,
                        TrailParticle {
                            timer_tween: TimerSimple::new_started(
                                globals.random.f32_in_range_closed(0.15, 0.25),
                            ),
                            pos: point,
                            size: globals.random.f32_in_range_closed(2.0, 4.0),
                            color: if boost {
                                COLOR_BOOST
                            } else {
                                COLOR_SKILL_POINT
                            },
                        },
                    );
                }
            }

            // TICK EFFECT
            if player.timer_tick.update(deltatime) {
                let tickeffect_entity = self.entities.create();
                self.tickeffects.insert(
                    tickeffect_entity,
                    TickEffect {
                        parent: *player_entity,
                        timer_tween: TimerSimple::new_started(0.13),
                        pos: player.pos,
                        width: 32.0,
                        height: 48.0,
                    },
                );
            }

            // EXPLODE
            let canvas_rect = Rect::from_width_height(globals.canvas_width, globals.canvas_height);
            if !canvas_rect.contains_point(player.pos) {
                entities_to_delete.push(*player_entity);

                let screen_shake = ModulatorScreenShake::new(&mut globals.random, 6.0, 0.2, 80.0);
                globals.camera.add_shake(screen_shake);

                let slowmotion_entity = self.entities.create();
                self.slowmotions.insert(
                    slowmotion_entity,
                    SlowMotion {
                        timer_tween: TimerSimple::new_started(1.0),
                        deltatime_speed_factor: 0.15,
                    },
                );

                let screenflash_entity = self.entities.create();
                self.screenflashes.insert(
                    screenflash_entity,
                    Screenflash {
                        color: COLOR_DEFAULT,
                        framecount_duration: 4,
                    },
                );

                for _ in 0..globals.random.gen_range(15, 25) {
                    let explosion_particle_entity = self.entities.create();
                    self.explode_particles.insert(
                        explosion_particle_entity,
                        ExplodeParticle {
                            timer_tween: TimerSimple::new_started(
                                globals.random.f32_in_range_closed(0.3, 0.5),
                            ),
                            pos: player.pos,
                            dir: globals.random.vec2_in_unit_disk(),
                            speed: globals.random.f32_in_range_closed(120.0, 300.0),
                            thickness: globals.random.f32_in_range_closed(1.0, 2.0),
                            length: globals.random.f32_in_range_closed(3.0, 15.0),
                            color: COLOR_DEFAULT,
                        },
                    );
                }
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE MUZZLEFLASH
        for (effect_entity, effect) in self.muzzleflashes.iter_mut() {
            if let Some(player) = self.players.get(&effect.parent) {
                let player_dir = Vec2::from_angle_flipped_y(deg_to_rad(player.angle));
                effect.pos = player.pos + player.width * player_dir;
                effect.angle = player.angle + 45.0;
            }

            effect.timer_tween.update(deltatime);
            let percentage = easing::cubic_inout(effect.timer_tween.completion_ratio());
            let width = lerp(effect.size, 0.0, percentage);

            draw.draw_rect_transformed(
                Vec2::filled(width),
                Vec2::filled(width) / 2.0,
                effect.pos,
                Vec2::ones(),
                Vec2::from_angle_flipped_y(deg_to_rad(effect.angle)),
                DEPTH_EFFECTS,
                Color::white(),
                ADDITIVITY_NONE,
            );

            if effect.timer_tween.is_finished() {
                entities_to_delete.push(*effect_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE EXPLODE PARTICLES
        for (particle_entity, particle) in self.explode_particles.iter_mut() {
            particle.timer_tween.update(deltatime);
            let percentage = particle.timer_tween.completion_ratio();
            let length = lerp(particle.length, 0.0, percentage);
            let speed = lerp(particle.speed, 0.0, percentage);
            particle.pos += speed * deltatime * particle.dir;

            draw.draw_line_with_thickness(
                particle.pos.pixel_snapped(),
                particle.pos.pixel_snapped() + length * particle.dir,
                particle.thickness,
                false,
                DEPTH_EFFECTS,
                particle.color,
                ADDITIVITY_NONE,
            );

            if particle.timer_tween.is_finished() {
                entities_to_delete.push(*particle_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE TICKEFFECT
        for (effect_entity, effect) in self.tickeffects.iter_mut() {
            effect.timer_tween.update(deltatime);
            let percentage = easing::cubic_inout(effect.timer_tween.completion_ratio());
            let width = effect.width; //lerp(effect.width, effect.height, percentage);
            let height = lerp(effect.height, 0.0, percentage);
            let offset_y = lerp(0.0, -effect.height / 2.0, percentage);

            if let Some(player) = self.players.get(&effect.parent) {
                effect.pos = player.pos;
            }

            draw.draw_rect(
                Rect::from_pos_width_height(effect.pos, width, height)
                    .centered()
                    .translated_by(Vec2::filled_y(offset_y)),
                DEPTH_EFFECTS,
                COLOR_DEFAULT,
                ADDITIVITY_NONE,
            );

            if effect.timer_tween.is_finished() {
                entities_to_delete.push(*effect_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE TRAILPARTICLES
        for (particle_entity, particle) in self.trail_particles.iter_mut() {
            particle.timer_tween.update(deltatime);
            let percentage = particle.timer_tween.completion_ratio();
            let size = lerp(particle.size, 0.0, percentage);

            draw.draw_circle_filled(
                particle.pos,
                size,
                DEPTH_EFFECTS,
                particle.color,
                ADDITIVITY_NONE,
            );

            if particle.timer_tween.is_finished() {
                entities_to_delete.push(*particle_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE PROJECTILES
        for (projectile_entity, projectile) in self.projectiles.iter_mut() {
            projectile.pos += projectile.speed * projectile.dir * deltatime;

            draw.draw_circle_bresenham(
                projectile.pos,
                projectile.size,
                DEPTH_PROJECTILE,
                Color::white(),
                ADDITIVITY_NONE,
            );

            let canvas_rect = Rect::from_width_height(globals.canvas_width, globals.canvas_height);
            if !canvas_rect.contains_point(projectile.pos) {
                entities_to_delete.push(*projectile_entity);

                let hit_entity = self.entities.create();
                self.projectile_hits.insert(
                    hit_entity,
                    HitEffect::new(
                        projectile.pos.clamped_to_rect(canvas_rect),
                        3.0 * projectile.size,
                        0.0,
                        COLOR_DEFAULT,
                        COLOR_HP,
                    ),
                );
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE HIT EFFECTS
        for (effect_entity, effect) in self.projectile_hits.iter_mut() {
            effect.timer_stages.update(deltatime);
            let color = if effect.timer_stages.time_cur < 0.1 {
                effect.color_first_stage
            } else {
                effect.color_second_stage
            };

            draw.draw_rect_transformed(
                Vec2::filled(effect.size),
                Vec2::filled(effect.size / 2.0),
                effect.pos,
                Vec2::ones(),
                Vec2::from_angle_flipped_y(deg_to_rad(effect.angle)),
                DEPTH_EFFECTS,
                color,
                ADDITIVITY_NONE,
            );

            if effect.timer_stages.is_finished() {
                entities_to_delete.push(*effect_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE AMMO

        for (ammo_entity, ammo) in self.ammo_collectibles.iter_mut() {
            if let Some(player) = self.players.get(&ammo.move_target) {
                let dir_current = ammo.vel.normalized();
                let dir_target = (player.pos - ammo.pos).normalized();
                let dir_final = Vec2::lerp(dir_current, dir_target, 0.1).normalized();
                ammo.vel = ammo.vel.magnitude() * dir_final;
            }

            ammo.pos += ammo.vel * deltatime;
            ammo.look_angle += ammo.look_angle_speed * deltatime;
            let ammo_dir_look = Vec2::from_angle_flipped_y(deg_to_rad(ammo.look_angle));

            // DRAWING
            let ammo_pos = ammo.pos;
            let rect = Rect::from_width_height(ammo.width, ammo.height).centered();
            let transform_into_ammo_coords = move |point: Vec2| {
                point.transformed(Vec2::zero(), ammo_pos, Vec2::ones(), ammo_dir_look)
            };

            let linestrip: Vec<Vec2> = rect
                .linestrip()
                .iter()
                .map(|&point| transform_into_ammo_coords(point))
                .collect();
            draw.draw_linestrip_bresenham(
                &linestrip,
                DEPTH_COLLECTIBLES,
                COLOR_AMMO,
                ADDITIVITY_NONE,
            );

            let canvas_rect = Rect::from_width_height(globals.canvas_width, globals.canvas_height)
                .extended_uniformly_by(ammo.width);
            if !canvas_rect.contains_point(ammo.pos) {
                entities_to_delete.push(*ammo_entity);
            }

            if let Some(player) = self.players.get(&ammo.move_target) {
                if Vec2::distance_squared(player.pos, ammo.pos) < squared(ammo.width) {
                    entities_to_delete.push(*ammo_entity);

                    let hit_entity = self.entities.create();
                    self.projectile_hits.insert(
                        hit_entity,
                        HitEffect::new(ammo.pos, ammo.width, 45.0, COLOR_DEFAULT, COLOR_AMMO),
                    );
                    for _ in 0..globals.random.gen_range(4, 8) {
                        let explosion_particle_entity = self.entities.create();
                        self.explode_particles.insert(
                            explosion_particle_entity,
                            ExplodeParticle {
                                timer_tween: TimerSimple::new_started(
                                    globals.random.f32_in_range_closed(0.3, 0.5),
                                ),
                                pos: ammo.pos,
                                dir: globals.random.vec2_in_unit_disk(),
                                speed: globals.random.f32_in_range_closed(50.0, 100.0),
                                thickness: globals.random.f32_in_range_closed(1.0, 2.0),
                                length: globals.random.f32_in_range_closed(3.0, 8.0),
                                color: COLOR_AMMO,
                            },
                        );
                    }
                }
            }
        }

        //------------------------------------------------------------------------------------------
        // DELETE MARKED ENTITIES
        for entity in entities_to_delete {
            self.players.remove(&entity);
            self.slowmotions.remove(&entity);
            self.screenflashes.remove(&entity);
            self.tickeffects.remove(&entity);
            self.muzzleflashes.remove(&entity);
            self.projectiles.remove(&entity);
            self.projectile_hits.remove(&entity);
            self.explode_particles.remove(&entity);
            self.trail_particles.remove(&entity);
            self.ammo_collectibles.remove(&entity);
        }
    }
}
