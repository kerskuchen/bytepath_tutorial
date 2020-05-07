use ct_lib::audio::*;
use ct_lib::draw::*;
use ct_lib::game::*;
use ct_lib::math::*;
use ct_lib::random::*;

use ct_lib::dformat;
use lazy_static::*;

use hecs::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use std::collections::HashMap;
use std::collections::HashSet;

const DEBUG_DRAW_ENABLE: bool = false;

const DEPTH_BACKGROUND: Depth = 0.0;
const DEPTH_PLAYER: Depth = 10.0;
const DEPTH_COLLECTIBLES: Depth = 15.0;
const DEPTH_PROJECTILE: Depth = 20.0;
const DEPTH_EFFECTS: Depth = 30.0;
const DEPTH_INFOTEXT: Depth = 35.0;
const DEPTH_SCREENFLASH: Depth = 60.0;
const DEPTH_GUI: Depth = 55.0;

// TODO: When f32 gets const functions we can just use from_rgb_bytes instead of this monstrosity
const COLOR_BACKGROUND: Color = Color::from_rgb(16.0 / 255.0, 16.0 / 255.0, 16.0 / 255.0);
const COLOR_DEFAULT: Color = Color::from_rgb(222.0 / 255.0, 222.0 / 255.0, 222.0 / 255.0);
const COLOR_AMMO: Color = Color::from_rgb(123.0 / 255.0, 200.0 / 255.0, 164.0 / 255.0);
const COLOR_BOOST: Color = Color::from_rgb(76.0 / 255.0, 195.0 / 255.0, 217.0 / 255.0);
const COLOR_HP: Color = Color::from_rgb(241.0 / 255.0, 103.0 / 255.0, 69.0 / 255.0);
const COLOR_SKILL_POINT: Color = Color::from_rgb(255.0 / 255.0, 198.0 / 255.0, 93.0 / 255.0);

const COLOR_NEGATIVE_DEFAULT: Color = Color::from_rgb(
    1.0 - 222.0 / 255.0,
    1.0 - 222.0 / 255.0,
    1.0 - 222.0 / 255.0,
);
const COLOR_NEGATIVE_AMMO: Color = Color::from_rgb(
    1.0 - 123.0 / 255.0,
    1.0 - 200.0 / 255.0,
    1.0 - 164.0 / 255.0,
);
const COLOR_NEGATIVE_BOOST: Color =
    Color::from_rgb(1.0 - 76.0 / 255.0, 1.0 - 195.0 / 255.0, 1.0 - 217.0 / 255.0);
const COLOR_NEGATIVE_HP: Color =
    Color::from_rgb(1.0 - 241.0 / 255.0, 1.0 - 103.0 / 255.0, 1.0 - 69.0 / 255.0);
const COLOR_NEGATIVE_SKILL_POINT: Color =
    Color::from_rgb(1.0 - 255.0 / 255.0, 1.0 - 198.0 / 255.0, 1.0 - 93.0 / 255.0);

const COLORS: [Color; 5] = [
    COLOR_DEFAULT,
    COLOR_HP,
    COLOR_AMMO,
    COLOR_BOOST,
    COLOR_SKILL_POINT,
];
const COLORS_NEGATIVE: [Color; 5] = [
    COLOR_NEGATIVE_DEFAULT,
    COLOR_NEGATIVE_HP,
    COLOR_NEGATIVE_AMMO,
    COLOR_NEGATIVE_BOOST,
    COLOR_NEGATIVE_SKILL_POINT,
];

const COLORS_ALL: [Color; 9] = [
    COLOR_DEFAULT,
    COLOR_HP,
    COLOR_AMMO,
    COLOR_BOOST,
    COLOR_SKILL_POINT,
    COLOR_NEGATIVE_HP,
    COLOR_NEGATIVE_AMMO,
    COLOR_NEGATIVE_BOOST,
    COLOR_NEGATIVE_SKILL_POINT,
];

type CollisionMask = u64;
const COLLISION_LAYER_ENEMY: u64 = 1 << 0;
const COLLISION_LAYER_PLAYER: u64 = 1 << 1;
const COLLISION_LAYER_ENEMY_PROJECTILE: u64 = 1 << 2;
const COLLISION_LAYER_PLAYER_PROJECTILE: u64 = 1 << 3;
const COLLISION_LAYER_COLLECTIBLES: u64 = 1 << 4;

const PLAYER_BASE_HP: f32 = 100.0;
const PLAYER_BASE_BOOST: f32 = 100.0;
const PLAYER_BASE_AMMO: f32 = 100.0;

const PLAYER_BASE_GAIN_HP: f32 = 25.0;
const PLAYER_BASE_GAIN_AMMO: f32 = 5.0;
const PLAYER_BASE_GAIN_BOOST: f32 = 25.0;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Attacks

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, EnumIter)]
pub enum AttackType {
    Neutral,
    Double,
    Triple,
    Rapid,
    Spread,
    Back,
    Side,
}

#[derive(Debug, Copy, Clone)]
struct Attack {
    pub typename: AttackType,
    pub name: &'static str,
    pub name_abbreviation: &'static str,
    pub reload_time: f32,
    pub ammo_consumption_on_shot: f32,
    pub color: Color,
}

lazy_static! {
    static ref ATTACKS: HashMap<AttackType, Attack> = {
        let mut attacks = HashMap::new();
        attacks.insert(
            AttackType::Neutral,
            Attack {
                typename: AttackType::Neutral,
                name: "Neutral",
                name_abbreviation: "N",
                reload_time: 0.24,
                ammo_consumption_on_shot: 0.0,
                color: COLOR_DEFAULT,
            },
        );
        attacks.insert(
            AttackType::Double,
            Attack {
                typename: AttackType::Double,
                name: "Double",
                name_abbreviation: "2",
                reload_time: 0.32,
                ammo_consumption_on_shot: 2.0,
                color: COLOR_AMMO,
            },
        );
        attacks.insert(
            AttackType::Triple,
            Attack {
                typename: AttackType::Triple,
                name: "Triple",
                name_abbreviation: "3",
                reload_time: 0.32,
                ammo_consumption_on_shot: 3.0,
                color: COLOR_BOOST,
            },
        );
        attacks.insert(
            AttackType::Rapid,
            Attack {
                typename: AttackType::Rapid,
                name: "Rapid",
                name_abbreviation: "R",
                reload_time: 0.12,
                ammo_consumption_on_shot: 1.0,
                color: COLOR_DEFAULT,
            },
        );
        attacks.insert(
            AttackType::Spread,
            Attack {
                typename: AttackType::Spread,
                name: "Spread",
                name_abbreviation: "RS",
                reload_time: 0.16,
                ammo_consumption_on_shot: 1.0,
                color: COLOR_DEFAULT,
            },
        );
        attacks.insert(
            AttackType::Back,
            Attack {
                typename: AttackType::Back,
                name: "Back",
                name_abbreviation: "Ba",
                reload_time: 0.32,
                ammo_consumption_on_shot: 2.0,
                color: COLOR_SKILL_POINT,
            },
        );
        attacks.insert(
            AttackType::Side,
            Attack {
                typename: AttackType::Side,
                name: "Side",
                name_abbreviation: "Si",
                reload_time: 0.32,
                ammo_consumption_on_shot: 3.0,
                color: COLOR_BOOST,
            },
        );

        attacks
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Shared Components

type Blinker = TimerStateSwitchBinary;

#[derive(Debug, Clone)]
struct Collider {
    radius: f32,

    layers_own: CollisionMask,
    layers_affects: CollisionMask,

    collisions: Vec<Entity>,
}

#[derive(Debug, Copy, Clone)]
struct Transform {
    pub pos: Vec2,
    /// Given in degrees [-360, 360] counterclockwise
    pub dir_angle: f32,
}

#[derive(Debug, Copy, Clone)]
struct TweenColor {
    pub timer_tween: TimerSimple,
    pub color_start: Color,
    pub color_end: Color,
    pub easing_type: EasingType,
}
impl TweenColor {
    fn new(
        color_start: Color,
        color_end: Color,
        tween_time: f32,
        easing_type: EasingType,
    ) -> TweenColor {
        TweenColor {
            timer_tween: TimerSimple::new_started(tween_time),
            color_start,
            color_end,
            easing_type,
        }
    }
    fn update(&mut self, drawable: &mut Drawable, deltatime: f32) {
        self.timer_tween.update(deltatime);
        let percentage = ease(self.timer_tween.completion_ratio(), self.easing_type);
        drawable.color = Color::lerp(self.color_start, self.color_end, percentage);
    }
}

#[derive(Debug, Copy, Clone)]
struct TweenScale {
    pub timer_tween: TimerSimple,
    pub scale_start: f32,
    pub scale_end: f32,
    pub easing_type: EasingType,
}
impl TweenScale {
    fn new(
        scale_start: f32,
        scale_end: f32,
        tween_time: f32,
        easing_type: EasingType,
    ) -> TweenScale {
        TweenScale {
            timer_tween: TimerSimple::new_started(tween_time),
            scale_start,
            scale_end,
            easing_type,
        }
    }
    fn update(&mut self, drawable: &mut Drawable, deltatime: f32) {
        self.timer_tween.update(deltatime);
        let percentage = ease(self.timer_tween.completion_ratio(), self.easing_type);
        drawable.scale = Vec2::filled(lerp(self.scale_start, self.scale_end, percentage));
    }
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
    pub remove_entity_if_lost_parent: bool,

    pub pos_snap: bool,
    pub pos_offset: Vec2,

    pub dir_angle_snap: bool,
    pub dir_angle_offset: f32,
}

#[derive(Debug, Copy, Clone)]
struct MoveTowardsTarget {
    pub target: Entity,
    pub follow_precision_percent: f32,
}

#[derive(Debug, Copy, Clone)]
struct AutoremoveTimer {
    pub timer: TimerSimple,
}
impl AutoremoveTimer {
    fn new(lifetime: f32) -> AutoremoveTimer {
        AutoremoveTimer {
            timer: TimerSimple::new_started(lifetime),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct AutoremoveTimerFrames {
    pub framecount_start: usize,
    pub framecount_left: usize,
}
impl AutoremoveTimerFrames {
    fn new(framecount: usize) -> AutoremoveTimerFrames {
        AutoremoveTimerFrames {
            framecount_start: framecount,
            framecount_left: framecount,
        }
    }
    fn update_and_check_if_finished(&mut self) -> bool {
        if self.framecount_left > 0 {
            self.framecount_left -= 1;
            false
        } else {
            true
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Primary Components

#[derive(Debug, Copy, Clone)]
pub enum Skill {
    AddPercentageHP(i32),
    AddPercentageAmmo(i32),
    AddPercentageBoost(i32),
    AddHp(i32),
    AddAmmo(i32),
    AddBoost(i32),
    AddHpGain(i32),
    AddAmmoGain(i32),
    AddBoostGain(i32),
}

impl Skill {
    pub fn name(&self) -> String {
        match self {
            Skill::AddPercentageHP(_) => "HP".to_string(),
            Skill::AddPercentageAmmo(_) => "Ammo".to_string(),
            Skill::AddPercentageBoost(_) => "Boost".to_string(),
            Skill::AddHp(_) => "Flat HP".to_string(),
            Skill::AddAmmo(_) => "Flat Ammo".to_string(),
            Skill::AddBoost(_) => "Flat Boost".to_string(),
            Skill::AddHpGain(_) => "HP Gain".to_string(),
            Skill::AddAmmoGain(_) => "Ammo Gain".to_string(),
            Skill::AddBoostGain(_) => "Boost Gain".to_string(),
        }
    }
    pub fn description(&self) -> String {
        match self {
            Skill::AddPercentageHP(value) => format!("+{}% HP", value),
            Skill::AddPercentageAmmo(value) => format!("+{}% Ammo", value),
            Skill::AddPercentageBoost(value) => format!("+{}% Boost", value),
            Skill::AddHp(value) => format!("+{} HP", value),
            Skill::AddAmmo(value) => format!("+{} Ammo", value),
            Skill::AddBoost(value) => format!("+{} Boost", value),
            Skill::AddHpGain(value) => format!("+{} HP Gain", value),
            Skill::AddAmmoGain(value) => format!("+{} Ammo Gain", value),
            Skill::AddBoostGain(value) => format!("+{} Boost Gain", value),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Player {
    pub attack: Attack,
    pub timer_trail_particles: TriggerRepeating,

    pub ship_type: ShipType,

    pub speed: f32,
    pub speed_max: f32,
    pub speed_base_max: f32,
    pub acc: f32,

    pub turn_speed: f32,

    pub size: f32,

    pub reload_timer: TriggerRepeating,

    pub hp: f32,
    pub hp_max: f32,
    pub hp_gain: f32,

    pub invincible_timer: TimerSimple,

    pub ammo: f32,
    pub ammo_max: f32,
    pub ammo_gain: f32,

    pub boost: f32,
    pub boost_max: f32,
    pub boost_gain: f32,

    pub boost_allowed: bool,
    pub boost_cooldown_time: f32,
    pub boost_cooldown_timer: TimerSimple,

    pub cycle_cooldown: f32,
    pub cycle_timer: TimerSimple,
}

impl Player {
    fn new(size: f32, ship_type: ShipType, skills: &[Skill]) -> Player {
        let (hp, hp_gain, ammo, ammo_gain, boost, boost_gain) = {
            let mut gain_hp = PLAYER_BASE_GAIN_HP;
            let mut gain_ammo = PLAYER_BASE_GAIN_AMMO;
            let mut gain_boost = PLAYER_BASE_GAIN_BOOST;
            let mut flat_hp = PLAYER_BASE_HP;
            let mut flat_ammo = PLAYER_BASE_AMMO;
            let mut flat_boost = PLAYER_BASE_BOOST;
            let mut multiplier_hp = 1.0;
            let mut multiplier_ammo = 1.0;
            let mut multiplier_boost = 1.0;
            for skill in skills {
                match skill {
                    Skill::AddPercentageHP(value) => {
                        multiplier_hp += (*value as f32) / 100.0;
                    }
                    Skill::AddPercentageAmmo(value) => {
                        multiplier_ammo += (*value as f32) / 100.0;
                    }
                    Skill::AddPercentageBoost(value) => {
                        multiplier_boost += (*value as f32) / 100.0;
                    }
                    Skill::AddHp(value) => {
                        flat_hp += *value as f32;
                    }
                    Skill::AddAmmo(value) => {
                        flat_ammo += *value as f32;
                    }
                    Skill::AddBoost(value) => {
                        flat_boost += *value as f32;
                    }
                    Skill::AddHpGain(value) => {
                        gain_hp += *value as f32;
                    }
                    Skill::AddAmmoGain(value) => {
                        gain_ammo += *value as f32;
                    }
                    Skill::AddBoostGain(value) => {
                        gain_boost += *value as f32;
                    }
                }
            }

            (
                flat_hp * multiplier_hp,
                gain_hp,
                flat_ammo * multiplier_ammo,
                gain_ammo,
                flat_boost * multiplier_boost,
                gain_boost,
            )
        };

        let attack = ATTACKS[&AttackType::Neutral];
        Player {
            attack,
            timer_trail_particles: TriggerRepeating::new(0.01),
            ship_type,
            speed: 0.0,

            speed_max: 100.0,

            speed_base_max: 100.0,
            acc: 100.0,
            turn_speed: 1.66 * 180.0,

            size,

            reload_timer: TriggerRepeating::new(attack.reload_time),

            hp,
            hp_max: hp,
            hp_gain,

            invincible_timer: TimerSimple::new_stopped(2.0),

            ammo,
            ammo_max: ammo,
            ammo_gain,

            boost,
            boost_max: boost,
            boost_gain,

            boost_allowed: true,
            boost_cooldown_time: 2.0,
            boost_cooldown_timer: TimerSimple::new_stopped(1.0),
            cycle_cooldown: 5.0,
            cycle_timer: TimerSimple::new_started(5.0),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Enemy {
    hp: f32,
    hp_max: f32,
    hitflash_timer: TimerSimple,
    radius: f32,
    can_shoot: bool,
    is_charging: bool,
    timer_shoot: TimerSimple,
    timer_charge: TriggerRepeating,
    score: usize,
}

#[derive(Debug, Copy, Clone)]
enum CollectibleType {
    Boost,
    Ammo,
    Hp,
    Skillpoint,
    Attack(AttackType),
}
impl CollectibleType {
    fn get_infotext_string(&self) -> &'static str {
        match self {
            CollectibleType::Boost => "+BOOST",
            CollectibleType::Ammo => "+AMMO",
            CollectibleType::Hp => "+HP",
            CollectibleType::Skillpoint => "+1 SP",
            CollectibleType::Attack(attacktype) => ATTACKS[attacktype].name,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Collectible {
    collectible: CollectibleType,
    color: Color,
    size: f32,
}

#[derive(Debug, Copy, Clone)]
struct TickEffect {
    pub timer_tween: TimerSimple,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Copy, Clone)]
struct Projectile {
    pub length: f32,
    pub color: Color,
    pub damage: f32,
}

#[derive(Debug, Copy, Clone)]
struct ExplodeParticle {
    pub timer_tween: TimerSimple,
    pub thickness: f32,
    pub length: f32,
    pub speed: f32,
    pub color: Color,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Drawables

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
        length: f32,
        thickness: f32,
        smooth_edges: bool,
        centered: bool,
    },
    Linestrips(Vec<Vec<(i32, i32)>>),
    Linestrip(Vec<Vec2>),
    Text {
        text: String,
        font_name: String,
        font_scale: f32,
        alignment: Option<TextAlignment>,
        color_background: Option<Color>,
    },
}
#[derive(Debug, Clone)]
struct Drawable {
    mesh: MeshType,
    pos_offset: Vec2,
    dir_angle_offset: f32,
    scale: Vec2,
    depth: Depth,
    color: Color,
    additivity: Additivity,
    add_jitter: bool,
    visible: bool,
}

#[derive(Debug, Clone)]
struct DrawableMulti {
    drawables: Vec<Drawable>,
}

fn draw_drawable(
    fonts: &HashMap<String, SpriteFont>,
    draw: &mut Drawstate,
    globals: &mut Globals,
    xform: &Transform,
    drawable: &Drawable,
) {
    if !drawable.visible {
        return;
    }

    let pos = xform.pos;
    let scale = drawable.scale;
    let dir = Vec2::from_angle_flipped_y(deg_to_rad(xform.dir_angle + drawable.dir_angle_offset));
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
                draw.draw_circle_filled(xform.pos, scale.x * *radius, depth, color, additivity);
            } else {
                draw.draw_circle_bresenham(xform.pos, scale.x * *radius, depth, color, additivity);
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

            draw.draw_rect(rect, *filled, depth, color, additivity);
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
        MeshType::Linestrips(linestrips) => {
            for linestrip_raw in linestrips {
                let jitter = if drawable.add_jitter {
                    Some(&mut globals.random)
                } else {
                    None
                };
                let linestrip: Vec<Vec2> =
                    linestrip_transform(linestrip_raw, pos, pivot, scale, dir, jitter);
                draw.draw_linestrip_bresenham(&linestrip, depth, color, additivity);
            }
        }
        MeshType::Linestrip(linestrip_raw) => {
            let jitter = if drawable.add_jitter {
                Some(&mut globals.random)
            } else {
                None
            };
            let linestrip: Vec<Vec2> =
                linestrip_transform(linestrip_raw, pos, pivot, scale, dir, jitter);
            draw.draw_linestrip_bresenham(&linestrip, depth, color, additivity);
        }
        MeshType::LineWithThickness {
            length,
            thickness,
            smooth_edges,
            centered,
        } => {
            let (start, end) = if *centered {
                (
                    xform.pos - 0.5 * *length * dir,
                    xform.pos + 0.5 * *length * dir,
                )
            } else {
                (xform.pos, xform.pos + *length * dir)
            };

            draw.draw_line_with_thickness(
                start,
                end,
                *thickness,
                *smooth_edges,
                depth,
                color,
                additivity,
            );
        }
        MeshType::Text {
            text,
            font_name,
            font_scale,
            alignment,
            color_background,
        } => {
            let font = fonts
                .get(font_name)
                .expect(&format!("Font '{}' not found in given fontmap", font_name));
            draw.draw_text(
                text,
                font,
                *font_scale,
                pos,
                Vec2::zero(),
                *alignment,
                *color_background,
                depth,
                color,
                additivity,
            );
        }
    };
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Custom Entities that are to awkward to be combined from components

#[derive(Debug, Clone)]
struct EnemyChargeParticle {
    pub muzzle_pos: Vec2,
    pub muzzle_pos_offset: Vec2,
    pub muzzle_entity: Entity,
    pub size: f32,
    pub timer: TimerSimple,
    pub start_pos: Vec2,
}
impl EnemyChargeParticle {
    fn new(
        muzzle_pos: Vec2,
        muzzle_pos_offset: Vec2,
        muzzle_entity: Entity,
        size: f32,
        lifetime: f32,
        start_pos: Vec2,
    ) -> EnemyChargeParticle {
        EnemyChargeParticle {
            muzzle_pos,
            muzzle_pos_offset,
            muzzle_entity,
            size,
            timer: TimerSimple::new_started(lifetime),
            start_pos,
        }
    }
}

#[derive(Debug, Clone)]
struct InfoText {
    pub pos: Vec2,
    pub timer: TimerSimple,
    pub blinker: Blinker,
    pub char_switcher: TriggerRepeating,
    pub color: Color,
    pub text: Vec<char>,
    pub text_color_foreground: Vec<Color>,
    pub text_color_background: Vec<Color>,
}
impl InfoText {
    fn new(pos: Vec2, text: &str, color: Color) -> InfoText {
        assert!(text.is_ascii());
        let text: Vec<char> = text.chars().collect();
        let text_color_foreground = vec![color; text.len()];
        let text_color_background = vec![Color::transparent(); text.len()];

        InfoText {
            pos,
            timer: TimerSimple::new_started(1.1),
            blinker: Blinker::new(true, 0.7, 0.05),
            char_switcher: TriggerRepeating::new_with_distinct_triggertimes(0.7, 0.035),
            color,
            text,
            text_color_foreground,
            text_color_background,
        }
    }

    fn update_and_check_if_finished(
        &mut self,
        draw: &mut Drawstate,
        random: &mut Random,
        gui_font: &SpriteFont,
        deltatime: f32,
    ) -> bool {
        self.timer.update(deltatime);

        let mut text_offset = Vec2::zero();

        // Change text characters and colors randomly
        if self.char_switcher.update_and_check(deltatime) {
            let random_ascii_chars = " 0123456789!@#$%&*()-=+[]^~/;?><.,|abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWYXZ".as_bytes();
            for ascii_char in self.text.iter_mut() {
                if random.gen_bool(0.2) {
                    *ascii_char = random.pick_from_slice(random_ascii_chars) as char;
                }
            }
            for color in self.text_color_foreground.iter_mut() {
                if random.gen_bool(0.05) {
                    *color = random.pick_from_slice(&COLORS_ALL)
                } else {
                    *color = self.color;
                }
            }
            for color in self.text_color_background.iter_mut() {
                if random.gen_bool(0.3) {
                    *color = random.pick_from_slice(&COLORS_ALL)
                } else {
                    *color = Color::transparent();
                }
            }
        }

        // Draw text
        let visible = self.blinker.update_and_check(deltatime);
        if visible {
            for (index, &character) in self.text.iter().enumerate() {
                text_offset = draw.draw_text(
                    &character.to_string(),
                    gui_font,
                    1.0,
                    self.pos,
                    text_offset,
                    None,
                    Some(self.text_color_background[index]),
                    DEPTH_INFOTEXT,
                    self.text_color_foreground[index],
                    ADDITIVITY_NONE,
                )
            }
        }

        self.timer.is_finished()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Archetypes

struct Archetypes {}

impl Archetypes {
    fn new_player(
        pos: Vec2,
        ship_type: ShipType,
        skills: &[Skill],
    ) -> (Transform, Motion, Drawable, Player, Collider) {
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
                mesh: MeshType::Linestrips(get_draw_lines_for_ship(ship_type)),
                pos_offset: Vec2::zero(),
                dir_angle_offset: 0.0,
                scale: Vec2::filled(player_size) / 4.0,
                color: COLOR_DEFAULT,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_PLAYER,
                add_jitter: true,
                visible: true,
            },
            Player::new(player_size, ship_type, skills),
            Collider {
                radius: player_size,
                layers_own: COLLISION_LAYER_PLAYER,
                layers_affects: COLLISION_LAYER_COLLECTIBLES,

                collisions: Vec::with_capacity(32),
            },
        )
    }

    fn new_screenflash(
        canvas_width: f32,
        canvas_height: f32,
        framecount: usize,
    ) -> (AutoremoveTimerFrames, Transform, Drawable) {
        (
            AutoremoveTimerFrames::new(framecount),
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
                dir_angle_offset: 0.0,
                scale: Vec2::ones(),
                color: COLOR_DEFAULT,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_SCREENFLASH,
                add_jitter: false,
                visible: true,
            },
        )
    }

    fn new_muzzleflash(
        parent: Entity,
        pos_offset: Vec2,
        dir_angle_offset: f32,
    ) -> (
        Transform,
        AutoremoveTimer,
        TweenScale,
        SnapToParent,
        Drawable,
    ) {
        let initial_size = 8.0;
        let lifetime = 0.1;
        (
            Transform {
                pos: Vec2::zero(),
                dir_angle: 0.0,
            },
            AutoremoveTimer::new(lifetime),
            TweenScale::new(initial_size, 0.0, lifetime, EasingType::CubicInOut),
            SnapToParent {
                parent,
                pos_snap: true,
                pos_offset,
                dir_angle_snap: true,
                dir_angle_offset,
                remove_entity_if_lost_parent: true,
            },
            Drawable {
                mesh: MeshType::RectangleTransformed {
                    width: 1.0,
                    height: 1.0,
                    filled: true,
                    centered: true,
                },
                pos_offset: Vec2::zero(),
                dir_angle_offset: 0.0,
                scale: Vec2::ones(),
                color: COLOR_DEFAULT,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_EFFECTS,
                add_jitter: false,
                visible: true,
            },
        )
    }

    fn new_trailparticle(
        pos: Vec2,
        size: f32,
        lifetime: f32,
        color: Color,
    ) -> (Transform, AutoremoveTimer, TweenScale, Drawable) {
        (
            Transform {
                pos,
                dir_angle: 0.0,
            },
            AutoremoveTimer::new(lifetime),
            TweenScale::new(size, 0.0, lifetime, EasingType::Linear),
            Drawable {
                mesh: MeshType::Circle {
                    radius: 1.0,
                    filled: true,
                },
                pos_offset: Vec2::zero(),
                dir_angle_offset: 0.0,
                scale: Vec2::filled(size),
                depth: DEPTH_EFFECTS,
                color,
                additivity: ADDITIVITY_NONE,
                add_jitter: false,
                visible: true,
            },
        )
    }

    fn new_projectile(
        pos: Vec2,
        dir: Vec2,
        speed: f32,
        length: f32,
        color: Color,
        damage: f32,
    ) -> (Transform, Motion, Projectile, Collider, DrawableMulti) {
        (
            Transform {
                pos,
                dir_angle: rad_to_deg(dir.to_angle_flipped_y()),
            },
            Motion {
                vel: speed * dir,
                acc: Vec2::zero(),
                dir_angle_vel: 0.0,
                dir_angle_acc: 0.0,
            },
            Projectile {
                length,
                color,
                damage,
            },
            Collider {
                radius: length,
                layers_own: COLLISION_LAYER_PLAYER_PROJECTILE,
                layers_affects: COLLISION_LAYER_ENEMY | COLLISION_LAYER_ENEMY_PROJECTILE,
                collisions: Vec::with_capacity(32),
            },
            DrawableMulti {
                drawables: vec![
                    Drawable {
                        mesh: MeshType::LineWithThickness {
                            length,
                            thickness: 0.4 * length,
                            smooth_edges: false,
                            centered: false,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: 0.0,
                        scale: Vec2::ones(),
                        add_jitter: false,
                        depth: DEPTH_PROJECTILE,
                        color,
                        additivity: ADDITIVITY_NONE,
                        visible: true,
                    },
                    Drawable {
                        mesh: MeshType::LineWithThickness {
                            length,
                            thickness: 0.4 * length,
                            smooth_edges: false,
                            centered: false,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: -180.0,
                        scale: Vec2::ones(),
                        add_jitter: false,
                        depth: DEPTH_PROJECTILE,
                        color: COLOR_DEFAULT,
                        additivity: ADDITIVITY_NONE,
                        visible: true,
                    },
                ],
            },
        )
    }

    fn new_enemy_projectile(
        pos: Vec2,
        dir: Vec2,
        speed: f32,
        length: f32,
        damage: f32,
    ) -> (Transform, Motion, Projectile, Collider, DrawableMulti) {
        (
            Transform {
                pos,
                dir_angle: rad_to_deg(dir.to_angle_flipped_y()),
            },
            Motion {
                vel: speed * dir,
                acc: Vec2::zero(),
                dir_angle_vel: 0.0,
                dir_angle_acc: 0.0,
            },
            Projectile {
                length,
                color: COLOR_HP,
                damage,
            },
            Collider {
                radius: length,
                layers_own: COLLISION_LAYER_ENEMY_PROJECTILE,
                layers_affects: COLLISION_LAYER_PLAYER | COLLISION_LAYER_PLAYER_PROJECTILE,
                collisions: Vec::with_capacity(32),
            },
            DrawableMulti {
                drawables: vec![Drawable {
                    mesh: MeshType::LineWithThickness {
                        length: 2.0 * length,
                        thickness: 0.4 * length,
                        smooth_edges: false,
                        centered: true,
                    },
                    pos_offset: Vec2::zero(),
                    dir_angle_offset: 0.0,
                    scale: Vec2::ones(),
                    add_jitter: false,
                    depth: DEPTH_PROJECTILE,
                    color: COLOR_HP,
                    additivity: ADDITIVITY_NONE,
                    visible: true,
                }],
            },
        )
    }

    fn new_ammo_collectible(
        pos: Vec2,
        vel: Vec2,
        dir_angle: f32,
        dir_angle_vel: f32,
        player_entity: Entity,
    ) -> (
        Transform,
        Motion,
        Collectible,
        MoveTowardsTarget,
        Collider,
        Drawable,
    ) {
        let size = 8.0;
        (
            Transform { pos, dir_angle },
            Motion {
                vel,
                acc: Vec2::zero(),
                dir_angle_vel,
                dir_angle_acc: 0.0,
            },
            Collectible {
                collectible: CollectibleType::Ammo,
                color: COLOR_AMMO,
                size,
            },
            MoveTowardsTarget {
                target: player_entity,
                follow_precision_percent: 0.1,
            },
            Collider {
                radius: size,
                layers_own: COLLISION_LAYER_COLLECTIBLES,
                layers_affects: 0,
                collisions: Vec::with_capacity(32),
            },
            Drawable {
                mesh: MeshType::RectangleTransformed {
                    width: size,
                    height: size,
                    filled: false,
                    centered: true,
                },
                pos_offset: Vec2::zero(),
                dir_angle_offset: 0.0,
                scale: Vec2::ones(),
                color: COLOR_AMMO,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_COLLECTIBLES,
                add_jitter: false,
                visible: true,
            },
        )
    }

    fn new_skillpoints_collectible(
        pos: Vec2,
        vel: Vec2,
        dir_angle: f32,
        dir_angle_vel: f32,
    ) -> (Transform, Motion, Collectible, Collider, DrawableMulti) {
        let size = 12.0;
        (
            Transform { pos, dir_angle },
            Motion {
                vel,
                acc: Vec2::zero(),
                dir_angle_vel,
                dir_angle_acc: 0.0,
            },
            Collectible {
                collectible: CollectibleType::Skillpoint,
                color: COLOR_SKILL_POINT,
                size,
            },
            Collider {
                radius: size,
                layers_own: COLLISION_LAYER_COLLECTIBLES,
                layers_affects: 0,
                collisions: Vec::with_capacity(32),
            },
            DrawableMulti {
                drawables: vec![
                    Drawable {
                        mesh: MeshType::RectangleTransformed {
                            width: 1.0 * size,
                            height: 1.0 * size,
                            filled: false,
                            centered: true,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: 0.0,
                        scale: Vec2::ones(),
                        color: COLOR_SKILL_POINT,
                        additivity: ADDITIVITY_NONE,
                        depth: DEPTH_COLLECTIBLES,
                        add_jitter: false,
                        visible: true,
                    },
                    Drawable {
                        mesh: MeshType::RectangleTransformed {
                            width: 0.25 * size,
                            height: 0.25 * size,
                            filled: true,
                            centered: true,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: 0.0,
                        scale: Vec2::ones(),
                        color: COLOR_SKILL_POINT,
                        additivity: ADDITIVITY_NONE,
                        depth: DEPTH_COLLECTIBLES,
                        add_jitter: false,
                        visible: true,
                    },
                ],
            },
        )
    }

    fn new_boost_collectible(
        pos: Vec2,
        vel: Vec2,
        dir_angle: f32,
        dir_angle_vel: f32,
    ) -> (Transform, Motion, Collectible, Collider, DrawableMulti) {
        let size = 12.0;
        (
            Transform { pos, dir_angle },
            Motion {
                vel,
                acc: Vec2::zero(),
                dir_angle_vel,
                dir_angle_acc: 0.0,
            },
            Collectible {
                collectible: CollectibleType::Boost,
                color: COLOR_BOOST,
                size,
            },
            Collider {
                radius: size,
                layers_own: COLLISION_LAYER_COLLECTIBLES,
                layers_affects: 0,
                collisions: Vec::with_capacity(32),
            },
            DrawableMulti {
                drawables: vec![
                    Drawable {
                        mesh: MeshType::RectangleTransformed {
                            width: 1.0 * size,
                            height: 1.0 * size,
                            filled: false,
                            centered: true,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: 0.0,
                        scale: Vec2::ones(),
                        color: COLOR_BOOST,
                        additivity: ADDITIVITY_NONE,
                        depth: DEPTH_COLLECTIBLES,
                        add_jitter: false,
                        visible: true,
                    },
                    Drawable {
                        mesh: MeshType::RectangleTransformed {
                            width: 0.25 * size,
                            height: 0.25 * size,
                            filled: true,
                            centered: true,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: 0.0,
                        scale: Vec2::ones(),
                        color: COLOR_BOOST,
                        additivity: ADDITIVITY_NONE,
                        depth: DEPTH_COLLECTIBLES,
                        add_jitter: false,
                        visible: true,
                    },
                ],
            },
        )
    }

    fn new_attack_collectible(
        pos: Vec2,
        vel: Vec2,
        attacktype: AttackType,
    ) -> (Transform, Motion, Collectible, Collider, DrawableMulti) {
        assert!(attacktype != AttackType::Neutral);
        let attack = ATTACKS[&attacktype];

        let font_name = "gui_font".to_owned();
        let label = attack.name_abbreviation.to_owned();
        let color = attack.color;
        let size = 14.0;
        (
            Transform {
                pos,
                dir_angle: 45.0,
            },
            Motion {
                vel,
                acc: Vec2::zero(),
                dir_angle_vel: 0.0,
                dir_angle_acc: 0.0,
            },
            Collectible {
                collectible: CollectibleType::Attack(attacktype),
                color,
                size,
            },
            Collider {
                radius: size,
                layers_own: COLLISION_LAYER_COLLECTIBLES,
                layers_affects: 0,
                collisions: Vec::with_capacity(32),
            },
            DrawableMulti {
                drawables: vec![
                    Drawable {
                        mesh: MeshType::RectangleTransformed {
                            width: 1.0 * size,
                            height: 1.0 * size,
                            filled: false,
                            centered: true,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: 0.0,
                        scale: Vec2::ones(),
                        color: COLOR_DEFAULT,
                        additivity: ADDITIVITY_NONE,
                        depth: DEPTH_COLLECTIBLES,
                        add_jitter: false,
                        visible: true,
                    },
                    Drawable {
                        mesh: MeshType::RectangleTransformed {
                            width: 1.3 * size,
                            height: 1.3 * size,
                            filled: false,
                            centered: true,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: 0.0,
                        scale: Vec2::ones(),
                        color,
                        additivity: ADDITIVITY_NONE,
                        depth: DEPTH_COLLECTIBLES,
                        add_jitter: false,
                        visible: true,
                    },
                    Drawable {
                        mesh: MeshType::Text {
                            text: label,
                            font_name,
                            font_scale: 1.0,
                            alignment: Some(TextAlignment {
                                x: AlignmentHorizontal::Center,
                                y: AlignmentVertical::Center,
                                origin_is_baseline: false,
                                ignore_whitespace: true,
                            }),
                            color_background: None,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: 0.0,
                        scale: Vec2::ones(),
                        color,
                        additivity: ADDITIVITY_NONE,
                        depth: DEPTH_COLLECTIBLES,
                        add_jitter: false,
                        visible: true,
                    },
                ],
            },
        )
    }

    fn new_hp_collectible(
        pos: Vec2,
        vel: Vec2,
    ) -> (Transform, Motion, Collectible, Collider, DrawableMulti) {
        let size = 10.0;
        (
            Transform {
                pos,
                dir_angle: 0.0,
            },
            Motion {
                vel,
                acc: Vec2::zero(),
                dir_angle_vel: 0.0,
                dir_angle_acc: 0.0,
            },
            Collectible {
                collectible: CollectibleType::Hp,
                color: COLOR_HP,
                size,
            },
            Collider {
                radius: size,
                layers_own: COLLISION_LAYER_COLLECTIBLES,
                layers_affects: 0,
                collisions: Vec::with_capacity(32),
            },
            DrawableMulti {
                drawables: vec![
                    Drawable {
                        mesh: MeshType::RectangleTransformed {
                            width: size,
                            height: size / 3.0,
                            filled: true,
                            centered: true,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: 0.0,
                        scale: Vec2::ones(),
                        color: COLOR_HP,
                        additivity: ADDITIVITY_NONE,
                        depth: DEPTH_COLLECTIBLES,
                        add_jitter: false,
                        visible: true,
                    },
                    Drawable {
                        mesh: MeshType::RectangleTransformed {
                            width: size / 3.0,
                            height: size,
                            filled: true,
                            centered: true,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: 0.0,
                        scale: Vec2::ones(),
                        color: COLOR_HP,
                        additivity: ADDITIVITY_NONE,
                        depth: DEPTH_COLLECTIBLES,
                        add_jitter: false,
                        visible: true,
                    },
                    Drawable {
                        mesh: MeshType::Circle {
                            radius: size,
                            filled: false,
                        },
                        pos_offset: Vec2::zero(),
                        dir_angle_offset: 0.0,
                        scale: Vec2::ones(),
                        color: COLOR_DEFAULT,
                        additivity: ADDITIVITY_NONE,
                        depth: DEPTH_COLLECTIBLES,
                        add_jitter: false,
                        visible: true,
                    },
                ],
            },
        )
    }

    fn new_hit_effect(
        pos: Vec2,
        width: f32,
        height: f32,
        dir_angle: f32,
        first_stage_color: Color,
        first_stage_duration: f32,
        second_stage_color: Color,
        second_stage_duration: f32,
        filled: bool,
    ) -> (Transform, AutoremoveTimer, TweenColor, Drawable) {
        let lifetime = first_stage_duration + second_stage_duration;
        (
            Transform { pos, dir_angle },
            AutoremoveTimer::new(lifetime),
            TweenColor::new(
                first_stage_color,
                second_stage_color,
                first_stage_duration,
                EasingType::StepEnd,
            ),
            Drawable {
                mesh: MeshType::RectangleTransformed {
                    width,
                    height,
                    filled,
                    centered: true,
                },
                pos_offset: Vec2::zero(),
                dir_angle_offset: 0.0,
                scale: Vec2::ones(),
                color: first_stage_color,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_EFFECTS,
                add_jitter: false,
                visible: true,
            },
        )
    }

    fn new_hit_effect_round(
        pos: Vec2,
        radius: f32,
        first_stage_color: Color,
        first_stage_duration: f32,
        second_stage_color: Color,
        second_stage_duration: f32,
        filled: bool,
    ) -> (Transform, AutoremoveTimer, TweenColor, Drawable) {
        let lifetime = first_stage_duration + second_stage_duration;
        (
            Transform {
                pos,
                dir_angle: 0.0,
            },
            AutoremoveTimer::new(lifetime),
            TweenColor::new(
                first_stage_color,
                second_stage_color,
                first_stage_duration,
                EasingType::StepEnd,
            ),
            Drawable {
                mesh: MeshType::Circle { radius, filled },
                pos_offset: Vec2::zero(),
                dir_angle_offset: 0.0,
                scale: Vec2::ones(),
                color: first_stage_color,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_EFFECTS,
                add_jitter: false,
                visible: true,
            },
        )
    }

    fn new_explode_particle(
        pos: Vec2,
        dir_angle: f32,
        speed: f32,
        thickness: f32,
        length: f32,
        lifetime: f32,
        color: Color,
    ) -> (
        Transform,
        Motion,
        AutoremoveTimer,
        ExplodeParticle,
        Drawable,
    ) {
        let dir = Vec2::from_angle_flipped_y(deg_to_rad(dir_angle));
        (
            Transform { pos, dir_angle },
            Motion {
                vel: speed * dir,
                acc: Vec2::zero(),
                dir_angle_vel: 0.0,
                dir_angle_acc: 0.0,
            },
            AutoremoveTimer::new(lifetime),
            ExplodeParticle {
                timer_tween: TimerSimple::new_started(lifetime),
                thickness,
                length,
                speed,
                color,
            },
            Drawable {
                mesh: MeshType::LineWithThickness {
                    length,
                    thickness: thickness,
                    smooth_edges: false,
                    centered: false,
                },
                pos_offset: Vec2::zero(),
                dir_angle_offset: 0.0,
                scale: Vec2::ones(),
                depth: DEPTH_EFFECTS,
                color: color,
                additivity: ADDITIVITY_NONE,
                add_jitter: false,
                visible: true,
            },
        )
    }

    fn new_tick_effect(
        player_entity: Entity,
    ) -> (
        Transform,
        SnapToParent,
        AutoremoveTimer,
        TickEffect,
        Drawable,
    ) {
        let width = 32.0;
        let height = 48.0;
        let lifetime = 0.13;
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
                remove_entity_if_lost_parent: true,
            },
            AutoremoveTimer::new(lifetime),
            TickEffect {
                timer_tween: TimerSimple::new_started(lifetime),
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
                dir_angle_offset: 0.0,
                scale: Vec2::ones(),
                depth: DEPTH_EFFECTS,
                color: COLOR_DEFAULT,
                additivity: ADDITIVITY_NONE,
                add_jitter: false,
                visible: true,
            },
        )
    }

    fn new_enemy_rock(
        random: &mut Random,
        pos: Vec2,
        vel: Vec2,
        radius: f32,
    ) -> (Transform, Motion, Collider, Enemy, Drawable) {
        fn create_irregular_polygon(
            random: &mut Random,
            vertex_count: usize,
            radius: f32,
        ) -> Vec<Vec2> {
            let mut result = Vec::new();

            let mut angle_current = 0.0;
            let angle_increment = deg_to_rad(360.0 / vertex_count as f32);
            for _ in 0..vertex_count {
                let distance = radius + random.f32_in_range_closed(-radius / 4.0, radius / 4.0);
                let angle = angle_current
                    + random.f32_in_range_closed(-angle_increment / 4.0, angle_increment / 4.0);
                let pos = Vec2::new(distance * f32::cos(angle), distance * f32::sin(angle));
                result.push(pos);

                angle_current += angle_increment;
            }

            // Connect the last vertex with the first
            let first = result.first().unwrap().clone();
            result.push(first);

            result
        }

        let dir_angle = random.f32_in_range_closed(0.0, 360.0);
        let dir_angle_vel = random.f32_in_range_closed(-360.0, 360.0);
        let linestrip = create_irregular_polygon(random, 8, 10.0);
        (
            Transform { pos, dir_angle },
            Motion {
                vel,
                acc: Vec2::zero(),
                dir_angle_vel,
                dir_angle_acc: 0.0,
            },
            Collider {
                radius,
                layers_own: COLLISION_LAYER_ENEMY,
                layers_affects: COLLISION_LAYER_PLAYER,
                collisions: Vec::with_capacity(32),
            },
            Enemy {
                hp: 100.0,
                hp_max: 100.0,
                hitflash_timer: TimerSimple::new_stopped(0.1),
                radius,
                can_shoot: false,
                is_charging: false,
                timer_shoot: TimerSimple::new_stopped(1.0),
                timer_charge: TriggerRepeating::new(1.0),
                score: 100,
            },
            Drawable {
                mesh: MeshType::Linestrip(linestrip),
                pos_offset: Vec2::zero(),
                dir_angle_offset: 0.0,
                scale: Vec2::ones(),
                color: COLOR_HP,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_COLLECTIBLES,
                add_jitter: false,
                visible: true,
            },
        )
    }

    fn new_enemy_shooter(pos: Vec2, vel: Vec2) -> (Transform, Motion, Collider, Enemy, Drawable) {
        let width = 12.0;
        let height = 6.0;
        let radius = f32::max(width, height);
        let dir_angle = if vel.x >= 0.0 { 0.0 } else { -180.0 };
        let linestrip = vec![
            Vec2::new(width, 0.0),
            Vec2::new(-width / 2.0, height),
            Vec2::new(-width, 0.0),
            Vec2::new(-width / 2.0, -height),
            Vec2::new(width, 0.0),
        ];
        (
            Transform { pos, dir_angle },
            Motion {
                vel,
                acc: Vec2::zero(),
                dir_angle_vel: 0.0,
                dir_angle_acc: 0.0,
            },
            Collider {
                radius,
                layers_own: COLLISION_LAYER_ENEMY,
                layers_affects: COLLISION_LAYER_PLAYER,
                collisions: Vec::with_capacity(32),
            },
            Enemy {
                hp: 100.0,
                hp_max: 100.0,
                hitflash_timer: TimerSimple::new_stopped(0.1),
                radius,
                can_shoot: true,
                is_charging: false,
                timer_shoot: TimerSimple::new_started(5.0),
                timer_charge: TriggerRepeating::new_with_distinct_triggertimes(4.0, 0.02),
                score: 150,
            },
            Drawable {
                mesh: MeshType::Linestrip(linestrip),
                pos_offset: Vec2::zero(),
                dir_angle_offset: 0.0,
                scale: Vec2::ones(),
                color: COLOR_HP,
                additivity: ADDITIVITY_NONE,
                depth: DEPTH_COLLECTIBLES,
                add_jitter: false,
                visible: true,
            },
        )
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

    fn add_component<ComponentType>(&mut self, entity: Entity, component: ComponentType)
    where
        ComponentType: Send + Sync + 'static,
    {
        self.commands.push(Box::new(move |world| {
            world
                .insert_one(entity, component)
                .expect("Could not add component to entity");
        }));
    }

    fn add_component_bundle<ComponentsBundleType>(
        &mut self,
        entity: Entity,
        components: ComponentsBundleType,
    ) where
        ComponentsBundleType: hecs::DynamicBundle + Send + Sync + 'static,
    {
        self.commands.push(Box::new(move |world| {
            world
                .insert(entity, components)
                .expect("Could not add components to entity");
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
// Ship types

#[derive(Debug, Copy, Clone)]
enum ShipType {
    Fighter,
    Sorcerer,
    Rogue,
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
// Director

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, EnumIter)]
pub enum ResourceType {
    Boost,
    Skillpoint,
    Health,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum EnemyType {
    Rock,
    Shooter,
}

pub struct Director {
    pub difficulty: usize,
    pub timer_spawn_attack: TriggerRepeating,
    pub timer_spawn_resource: TriggerRepeating,
    pub timer_round: TimerSimple,
    pub round_duration: f32,
    pub round_enemies_and_spawntimes: std::collections::VecDeque<(EnemyType, f32)>,
}
impl Director {
    pub fn new(random: &mut Random) -> Director {
        let difficulty = 1;
        let round_duration = 22.0;
        Director {
            difficulty,
            timer_spawn_attack: TriggerRepeating::new(30.0),
            timer_spawn_resource: TriggerRepeating::new(16.0),
            timer_round: TimerSimple::new_started(round_duration),
            round_duration,
            round_enemies_and_spawntimes: Director::create_enemies_and_spawntimes(
                random,
                difficulty,
                round_duration,
            ),
        }
    }

    pub fn update_and_spawn_enemies_and_collectibles(
        &mut self,
        random: &mut Random,
        deltatime: f32,
    ) -> (Option<EnemyType>, Option<ResourceType>, Option<AttackType>) {
        self.timer_round.update(deltatime);

        let attack_to_spawn = if self.timer_spawn_attack.update_and_check(deltatime) {
            let attacktypes: Vec<AttackType> = AttackType::iter().skip(1).collect();
            let attacktype = random.pick_from_slice(&attacktypes);
            Some(attacktype)
        } else {
            None
        };

        let resource_to_spawn = if self.timer_spawn_resource.update_and_check(deltatime) {
            let mut shufflebag = Shufflebag::new_with_counts(&[
                (ResourceType::Boost, 28),
                (ResourceType::Health, 14),
                (ResourceType::Skillpoint, 58),
            ]);
            let resourcetype = shufflebag.get_next(random);
            Some(resourcetype)
        } else {
            None
        };

        let enemy_to_spawn = {
            if let Some((enemy, spawn_time)) = self.round_enemies_and_spawntimes.front().cloned() {
                if spawn_time <= self.timer_round.time_cur {
                    self.round_enemies_and_spawntimes.pop_front();
                    Some(enemy)
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Update difficulty
        if self.timer_round.is_finished() {
            self.timer_round.restart();
            self.difficulty += 1;
            self.round_enemies_and_spawntimes = Director::create_enemies_and_spawntimes(
                random,
                self.difficulty,
                self.round_duration,
            );
        }

        (enemy_to_spawn, resource_to_spawn, attack_to_spawn)
    }

    fn create_enemies_and_spawntimes(
        random: &mut Random,
        difficulty: usize,
        round_duration: f32,
    ) -> std::collections::VecDeque<(EnemyType, f32)> {
        let enemy_spawn_chances = Director::get_enemy_spawn_chance(random, difficulty);

        let mut points = Director::get_spending_points_for_difficulty(difficulty);
        let mut enemybag = Shufflebag::new_with_counts(&enemy_spawn_chances);
        let mut enemies = Vec::new();

        while points > 0 {
            let enemy = enemybag.get_next(random);
            points -= Director::get_spending_cost_for_enemy(enemy);
            enemies.push(enemy);
        }

        let mut spawn_times = Vec::new();
        for _ in 0..enemies.len() {
            spawn_times.push(random.f32_in_range_open(0.0, round_duration))
        }
        spawn_times.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

        enemies.into_iter().zip(spawn_times.into_iter()).collect()
    }

    fn get_spending_points_for_difficulty(difficulty: usize) -> i32 {
        let mut points = 16;
        for n in 1..difficulty {
            points = if (n - 1) % 4 == 0 {
                points + 8
            } else if (n - 1) % 4 == 1 {
                points
            } else if (n - 1) % 4 == 2 {
                (points as f32 / 1.5) as i32
            } else {
                points * 2
            };
        }

        points
    }

    fn get_enemy_spawn_chance(random: &mut Random, difficulty: usize) -> Vec<(EnemyType, usize)> {
        match difficulty {
            0 => unimplemented!("There is no difficulty 0"),
            1 => vec![(EnemyType::Rock, 1)],
            2 => vec![(EnemyType::Rock, 8), (EnemyType::Shooter, 4)],
            3 => vec![(EnemyType::Rock, 8), (EnemyType::Shooter, 8)],
            4 => vec![(EnemyType::Rock, 4), (EnemyType::Shooter, 8)],
            _ => vec![
                (EnemyType::Rock, random.gen_range(2, 12)),
                (EnemyType::Shooter, random.gen_range(2, 12)),
            ],
        }
    }

    fn get_spending_cost_for_enemy(enemytype: EnemyType) -> i32 {
        match enemytype {
            EnemyType::Rock => 1,
            EnemyType::Shooter => 2,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Stage Scene

#[derive(Clone)]
struct SlowmotionModulator {
    factor: f32,
    timer: TimerSimple,
}
impl SlowmotionModulator {
    fn new() -> SlowmotionModulator {
        SlowmotionModulator {
            factor: 1.0,
            timer: TimerSimple::new_stopped(1.0),
        }
    }

    fn add_slowmotion(&mut self, duration: f32, factor: f32) {
        self.timer = TimerSimple::new_started(duration);
        self.factor = factor;
    }

    #[must_use]
    fn update_and_get_new_deltatime(&mut self, deltatime: f32) -> f32 {
        self.timer.update(deltatime);
        let percentage = self.timer.completion_ratio();
        let factor = lerp(self.factor, 1.0, percentage);
        factor * deltatime
    }
}

pub struct SceneStage {
    score: usize,
    skillpoint_count: usize,

    slowmotion: SlowmotionModulator,

    director: Director,
    fonts: HashMap<String, SpriteFont>,
    world: World,
    commands: WorldCommandBuffer,
    player: Entity,
}

impl Clone for SceneStage {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl SceneStage {
    pub fn new(
        draw: &mut Drawstate,
        _audio: &mut Audiostate,
        _assets: &mut GameAssets,
        _input: &GameInput,
        globals: &mut Globals,
    ) -> SceneStage {
        let mut world = World::new();

        let skills = vec![Skill::AddHp(15), Skill::AddPercentageBoost(50)];
        let player_pos = Vec2::new(globals.canvas_width, globals.canvas_height) / 2.0;
        let player = world.spawn(Archetypes::new_player(
            player_pos,
            ShipType::Sorcerer,
            &skills,
        ));

        let mut fonts = HashMap::new();
        fonts.insert("gui_font".to_owned(), draw.get_font("default_tiny").clone());

        SceneStage {
            score: 0,
            slowmotion: SlowmotionModulator::new(),

            director: Director::new(&mut globals.random),
            skillpoint_count: 0,
            fonts,
            world,
            player: player,
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
        out_game_events: &mut Vec<GameEvent>,
    ) {
        draw.set_clear_color_and_depth(COLOR_BACKGROUND, DEPTH_BACKGROUND);

        if input.keyboard.recently_pressed(Scancode::S) {
            let screen_shake = ModulatorScreenShake::new(&mut globals.random, 4.0, 1.0, 60.0);
            globals.camera.add_shake(screen_shake);
        }

        let deltatime = self
            .slowmotion
            .update_and_get_new_deltatime(globals.deltatime);

        //------------------------------------------------------------------------------------------
        // DRAW GUI

        // Score
        draw.draw_text(
            &self.score.to_string(),
            &self.fonts["gui_font"],
            1.0,
            Vec2::new(globals.canvas_width - 20.0, 10.0),
            Vec2::zero(),
            Some(TextAlignment {
                x: AlignmentHorizontal::Right,
                y: AlignmentVertical::Top,
                origin_is_baseline: false,
                ignore_whitespace: false,
            }),
            None,
            DEPTH_GUI,
            COLOR_DEFAULT,
            ADDITIVITY_NONE,
        );

        // Skillpoints
        draw.draw_text(
            &format!("{} SP", self.skillpoint_count),
            &self.fonts["gui_font"],
            1.0,
            Vec2::new(20.0, 10.0),
            Vec2::zero(),
            Some(TextAlignment {
                x: AlignmentHorizontal::Left,
                y: AlignmentVertical::Top,
                origin_is_baseline: false,
                ignore_whitespace: false,
            }),
            None,
            DEPTH_GUI,
            COLOR_SKILL_POINT,
            ADDITIVITY_NONE,
        );

        // Bars
        fn draw_bar(
            draw: &mut Drawstate,
            font: &SpriteFont,
            text_title: &str,
            text_value: &str,
            bar_color: Color,
            bar_center: Vec2,
            bar_width: f32,
            bar_height: f32,
            bar_filled_percentage: f32,
            swap_title_and_value_text_pos: bool,
        ) {
            let bar_left_top = bar_center - Vec2::new(bar_width, bar_height) / 2.0;
            draw.draw_rect(
                Rect::from_pos_width_height(
                    bar_left_top,
                    bar_width * bar_filled_percentage,
                    bar_height,
                ),
                true,
                DEPTH_GUI,
                bar_color,
                ADDITIVITY_NONE,
            );
            draw.draw_rect(
                Rect::from_pos_width_height(bar_left_top, bar_width, bar_height),
                false,
                DEPTH_GUI,
                Color::new(
                    bar_color.r - (32.0 / 255.0),
                    bar_color.g - (32.0 / 255.0),
                    bar_color.b - (32.0 / 255.0),
                    1.0,
                ),
                ADDITIVITY_NONE,
            );

            let mut pos_title = bar_center - Vec2::filled_y(8.0);
            let mut pos_value = bar_center + Vec2::filled_y(8.0);
            if swap_title_and_value_text_pos {
                std::mem::swap(&mut pos_title, &mut pos_value);
            }

            draw.draw_text(
                text_title,
                font,
                1.0,
                pos_title,
                Vec2::zero(),
                Some(TextAlignment {
                    x: AlignmentHorizontal::Center,
                    y: AlignmentVertical::Center,
                    origin_is_baseline: false,
                    ignore_whitespace: true,
                }),
                None,
                DEPTH_GUI,
                bar_color,
                ADDITIVITY_NONE,
            );
            draw.draw_text(
                text_value,
                font,
                1.0,
                pos_value,
                Vec2::zero(),
                Some(TextAlignment {
                    x: AlignmentHorizontal::Center,
                    y: AlignmentVertical::Center,
                    origin_is_baseline: false,
                    ignore_whitespace: true,
                }),
                None,
                DEPTH_GUI,
                bar_color,
                ADDITIVITY_NONE,
            );
        }

        let (
            player_hp,
            player_hp_max,
            player_boost,
            player_boost_max,
            player_ammo,
            player_ammo_max,
            player_cycle_percentage,
        ) = if let Some(player) = self.world.get::<Player>(self.player).ok() {
            (
                player.hp,
                player.hp_max,
                player.boost,
                player.boost_max,
                player.ammo,
                player.ammo_max,
                player.cycle_timer.completion_ratio(),
            )
        } else {
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        };

        let bar_width = 48.0;
        let bar_height = 4.0;

        draw_bar(
            draw,
            &self.fonts["gui_font"],
            "AMMO",
            &format!("{}/{}", roundi(player_ammo), roundi(player_ammo_max)),
            COLOR_AMMO,
            Vec2::new(globals.canvas_width / 2.0 - (bar_width / 2.0 + 4.0), 16.0),
            bar_width,
            bar_height,
            player_ammo / player_ammo_max,
            true,
        );
        draw_bar(
            draw,
            &self.fonts["gui_font"],
            "BOOST",
            &format!("{}/{}", roundi(player_boost), roundi(player_boost_max)),
            COLOR_BOOST,
            Vec2::new(globals.canvas_width / 2.0 + (bar_width / 2.0 + 4.0), 16.0),
            bar_width,
            bar_height,
            player_boost / player_boost_max,
            true,
        );
        draw_bar(
            draw,
            &self.fonts["gui_font"],
            "HP",
            &format!("{}/{}", roundi(player_hp), roundi(player_hp_max)),
            COLOR_HP,
            Vec2::new(
                globals.canvas_width / 2.0 - (bar_width / 2.0 + 4.0),
                globals.canvas_height - 16.0,
            ),
            bar_width,
            bar_height,
            player_hp / player_hp_max,
            false,
        );
        draw_bar(
            draw,
            &self.fonts["gui_font"],
            "CYCLE",
            "",
            COLOR_DEFAULT,
            Vec2::new(
                globals.canvas_width / 2.0 + (bar_width / 2.0 + 4.0),
                globals.canvas_height - 16.0,
            ),
            bar_width,
            bar_height,
            player_cycle_percentage,
            false,
        );

        let round_percentage = self.director.timer_round.completion_ratio();
        draw_bar(
            draw,
            &self.fonts["gui_font"],
            "DIFFICULTY",
            &self.director.difficulty.to_string(),
            COLOR_NEGATIVE_AMMO,
            Vec2::new(bar_width, globals.canvas_height - 16.0),
            bar_width,
            bar_height,
            round_percentage,
            false,
        );

        //------------------------------------------------------------------------------------------
        // RESTART GAME

        if self.world.get::<Player>(self.player).is_err() {
            let canvas_center =
                Rect::from_width_height(globals.canvas_width, globals.canvas_height).center();
            draw.draw_text(
                "PRESS ANY KEY TO RESTART",
                &self.fonts["gui_font"],
                1.0,
                canvas_center,
                Vec2::zero(),
                Some(TextAlignment {
                    x: AlignmentHorizontal::Center,
                    y: AlignmentVertical::Center,
                    origin_is_baseline: false,
                    ignore_whitespace: true,
                }),
                None,
                DEPTH_GUI,
                COLOR_DEFAULT,
                ADDITIVITY_NONE,
            );
            if input.keyboard.has_press_event {
                out_game_events.push(GameEvent::SwitchToScene {
                    scene_name: "stage".to_string(),
                })
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE DIRECTOR

        let (enemy_to_spawn, resource_to_spawn, attack_to_spawn) = self
            .director
            .update_and_spawn_enemies_and_collectibles(&mut globals.random, deltatime);

        fn create_spawn_pos_vel(
            random: &mut Random,
            canvas_width: f32,
            canvas_height: f32,
        ) -> (Vec2, Vec2) {
            let pos_offset = 10.0;
            let dir = random.pick_from_slice(&[-1.0, 1.0]);

            let pos = Vec2::new(
                canvas_width / 2.0 + dir * (canvas_width / 2.0 + pos_offset),
                random.f32_in_range_closed(pos_offset, canvas_height - pos_offset),
            );
            let vel = Vec2::filled_x(-dir * random.f32_in_range_closed(20.0, 40.0));

            (pos, vel)
        }

        // Spawn enemies
        if let Some(enemytype) = enemy_to_spawn {
            let (pos, vel) = create_spawn_pos_vel(
                &mut globals.random,
                globals.canvas_width,
                globals.canvas_height,
            );
            match enemytype {
                EnemyType::Rock => self.world.spawn(Archetypes::new_enemy_rock(
                    &mut globals.random,
                    pos,
                    vel,
                    8.0,
                )),
                EnemyType::Shooter => self.world.spawn(Archetypes::new_enemy_shooter(pos, vel)),
            };
        }

        // Spawn resources
        if let Some(resourcetype) = resource_to_spawn {
            let (pos, vel) = create_spawn_pos_vel(
                &mut globals.random,
                globals.canvas_width,
                globals.canvas_height,
            );
            match resourcetype {
                ResourceType::Boost => self.world.spawn(Archetypes::new_boost_collectible(
                    pos,
                    vel,
                    globals.random.f32_in_range_closed(0.0, 360.0),
                    globals.random.f32_in_range_closed(-360.0, 360.0),
                )),
                ResourceType::Skillpoint => {
                    self.world.spawn(Archetypes::new_skillpoints_collectible(
                        pos,
                        vel,
                        globals.random.f32_in_range_closed(0.0, 360.0),
                        globals.random.f32_in_range_closed(-360.0, 360.0),
                    ))
                }
                ResourceType::Health => self.world.spawn(Archetypes::new_hp_collectible(pos, vel)),
            };
        }

        // Spawn attacks
        if let Some(attacktype) = attack_to_spawn {
            let (pos, vel) = create_spawn_pos_vel(
                &mut globals.random,
                globals.canvas_width,
                globals.canvas_height,
            );
            self.world
                .spawn(Archetypes::new_attack_collectible(pos, vel, attacktype));
        }

        //------------------------------------------------------------------------------------------
        // SPAWN AMMO

        if input.keyboard.is_down(Scancode::A) {
            self.world.spawn(Archetypes::new_ammo_collectible(
                globals.random.vec2_in_rect(Rect::from_width_height(
                    globals.canvas_width,
                    globals.canvas_height,
                )),
                globals.random.vec2_in_unit_disk() * globals.random.f32_in_range_closed(10.0, 20.0),
                globals.random.f32_in_range_closed(0.0, 360.0),
                globals.random.f32_in_range_closed(-360.0, 360.0),
                self.player,
            ));
        }

        //------------------------------------------------------------------------------------------
        // AUTO-REMOVE ENTITIES

        for (entity, autoremove_timer) in &mut self.world.query::<&mut AutoremoveTimer>() {
            autoremove_timer.timer.update(deltatime);
            if autoremove_timer.timer.is_finished() {
                self.commands.remove_entity(entity);
            }
        }

        for (entity, autoremove_timer) in &mut self.world.query::<&mut AutoremoveTimerFrames>() {
            if autoremove_timer.update_and_check_if_finished() {
                self.commands.remove_entity(entity);
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
        for (entity, (xform, snap)) in &mut self.world.query::<(&mut Transform, &SnapToParent)>() {
            if let Some(parent_xform) = self.world.get::<Transform>(snap.parent).ok() {
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
            } else {
                if snap.remove_entity_if_lost_parent {
                    self.commands.remove_entity(entity);
                }
            }
        }

        //------------------------------------------------------------------------------------------
        // COLLISION

        // Clear collisions from last frame
        for (_entity, collider) in &mut self.world.query::<&mut Collider>() {
            collider.collisions.clear();
        }

        // Collect bodies for indexing
        let bodies: Vec<(Entity, Transform, Collider)> = self
            .world
            .query::<(&Transform, &Collider)>()
            .iter()
            .map(|(entity, (&xform, collider))| (entity, xform, collider.clone()))
            .collect();

        // Broadphase: Find collisions
        let mut pairs: HashSet<(usize, usize)> = HashSet::new();
        for index_a in 0..bodies.len() {
            for index_b in 0..bodies.len() {
                if index_a == index_b {
                    continue;
                }

                let body_a = &bodies[index_a];
                let body_b = &bodies[index_b];

                let body_a_entity = body_a.0;
                let body_b_entity = body_b.0;

                let body_a_xform = body_a.1;
                let body_b_xform = body_b.1;

                let body_a_collider = &body_a.2;
                let body_b_collider = &body_b.2;

                if body_a_collider.layers_own & body_b_collider.layers_affects == 0
                    && body_b_collider.layers_own & body_a_collider.layers_affects == 0
                {
                    continue;
                }

                if Vec2::distance_squared(body_a_xform.pos, body_b_xform.pos)
                    < squared(body_a_collider.radius + body_b_collider.radius)
                {
                    // Intersection found
                    if index_a < index_b {
                        pairs.insert((index_a, index_b));
                    } else {
                        pairs.insert((index_b, index_a));
                    }
                }
            }
        }

        // Resolve collisions
        for (index_a, index_b) in pairs {
            let body_a_entity = bodies[index_a].0;
            let body_b_entity = bodies[index_b].0;

            {
                let mut collider_a = self.world.get_mut::<Collider>(body_a_entity).unwrap();
                collider_a.collisions.push(body_b_entity);
            }
            {
                let mut collider_b = self.world.get_mut::<Collider>(body_b_entity).unwrap();
                collider_b.collisions.push(body_a_entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // STEERING

        // MOVE TOWARDS TARGET
        for (_entity, (xform, motion, follow)) in
            &mut self
                .world
                .query::<(&Transform, &mut Motion, &MoveTowardsTarget)>()
        {
            if let Some(target_xform) = self.world.get::<Transform>(follow.target).ok() {
                let dir_current = motion.vel.normalized();
                let dir_target = (target_xform.pos - xform.pos).normalized();
                let dir_final =
                    Vec2::lerp(dir_current, dir_target, follow.follow_precision_percent)
                        .normalized();
                motion.vel = motion.vel.magnitude() * dir_final;
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE ENEMY

        for (entity, (xform, motion, enemy, collider, drawable)) in
            &mut self
                .world
                .query::<(&Transform, &Motion, &mut Enemy, &Collider, &mut Drawable)>()
        {
            if enemy.can_shoot {
                if enemy.timer_charge.update_and_check(deltatime) {
                    // Spawn particle emitter
                    let muzzle_pos = xform.pos + enemy.radius * motion.vel.normalized();
                    let muzzle_pos_offset = enemy.radius * motion.vel.normalized();
                    self.commands.add_entity((EnemyChargeParticle::new(
                        muzzle_pos,
                        muzzle_pos_offset,
                        entity,
                        globals.random.f32_in_range_closed(2.0, 3.0),
                        globals.random.f32_in_range_closed(0.1, 0.3),
                        globals.random.vec2_in_disk(muzzle_pos, 20.0),
                    ),));
                }

                if enemy.timer_shoot.update_and_check_if_triggered(deltatime) {
                    // Schedule next shot
                    let shoot_time = globals.random.f32_in_range_closed(4.0, 6.0);
                    enemy.timer_shoot = TimerSimple::new_started(shoot_time);
                    enemy.timer_charge =
                        TriggerRepeating::new_with_distinct_triggertimes(shoot_time - 1.0, 0.02);

                    // Shoot projectile
                    let muzzle_pos = xform.pos + enemy.radius * motion.vel.normalized();
                    let player_pos =
                        if let Some(player_xform) = self.world.get::<Transform>(self.player).ok() {
                            player_xform.pos
                        } else {
                            globals.random.vec2_in_rect(Rect::from_width_height(
                                globals.canvas_width,
                                globals.canvas_height,
                            ))
                        };
                    self.commands.add_entity(Archetypes::new_enemy_projectile(
                        muzzle_pos,
                        (player_pos - xform.pos).normalized(),
                        globals.random.f32_in_range_closed(80.0, 100.0),
                        4.0,
                        10.0,
                    ));
                }
            }

            enemy.hitflash_timer.update(deltatime);
            if enemy.hitflash_timer.is_running() {
                drawable.color = COLOR_DEFAULT;
            } else {
                drawable.color = COLOR_HP;
            }

            let mut got_hit = false;
            for &collision_entity in &collider.collisions {
                if let Some(projectile) = self.world.get::<Projectile>(collision_entity).ok() {
                    enemy.hp = clampf(enemy.hp - projectile.damage, 0.0, enemy.hp_max);
                    got_hit = true;
                }
            }

            if enemy.hp == 0.0 {
                self.score += enemy.score;

                self.commands.remove_entity(entity);

                self.commands.add_entity(Archetypes::new_ammo_collectible(
                    xform.pos + enemy.radius * globals.random.vec2_in_unit_disk(),
                    globals.random.vec2_in_unit_disk()
                        * globals.random.f32_in_range_closed(10.0, 20.0),
                    globals.random.f32_in_range_closed(0.0, 360.0),
                    globals.random.f32_in_range_closed(-360.0, 360.0),
                    self.player,
                ));
                self.commands.add_entity(Archetypes::new_hit_effect(
                    xform.pos,
                    2.0 * enemy.radius,
                    2.0 * enemy.radius,
                    0.0,
                    COLOR_DEFAULT,
                    0.1,
                    COLOR_HP,
                    0.15,
                    true,
                ));
            } else {
                if got_hit {
                    enemy.hitflash_timer.restart();
                }
            }

            // Remove self when leaving screen
            if motion.vel.x > 0.0 && xform.pos.x >= globals.canvas_width + 2.0 * enemy.radius {
                self.commands.remove_entity(entity);
            }
            if motion.vel.x < 0.0 && xform.pos.x < -2.0 * enemy.radius {
                self.commands.remove_entity(entity);
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE PLAYER

        for (player_entity, (player_xform, player_motion, player, collider, drawable)) in
            &mut self.world.query::<(
                &Transform,
                &mut Motion,
                &mut Player,
                &Collider,
                &mut Drawable,
            )>()
        {
            player.invincible_timer.update(deltatime);
            if player.invincible_timer.is_running() {
                drawable.visible = floori(player.invincible_timer.time_cur / 0.04) % 2 != 0;
            } else {
                drawable.visible = true;
            }

            let mut player_damage: f32 = 0.0;
            let canvas_rect = Rect::from_width_height(globals.canvas_width, globals.canvas_height);
            if !canvas_rect.contains_point(player_xform.pos) {
                player_damage += player.hp_max;
            }

            for &collision_entity in &collider.collisions {
                if let Some(collectible) = self.world.get::<Collectible>(collision_entity).ok() {
                    match collectible.collectible {
                        CollectibleType::Boost => {
                            self.score += 150;
                            player.boost =
                                clampf(player.boost + player.boost_gain, 0.0, player.boost_max);
                            if player.boost > player.boost_max / 2.0 {
                                player.boost_allowed = true;
                                player.boost_cooldown_timer.stop();
                            }
                        }
                        CollectibleType::Ammo => {
                            self.score += 50;
                            player.ammo =
                                clampf(player.ammo + player.ammo_gain, 0.0, player.ammo_max);
                        }
                        CollectibleType::Hp => {
                            self.score += 100;
                            player.hp = clampf(player.hp + player.hp_gain, 0.0, player.hp_max);
                        }
                        CollectibleType::Skillpoint => {
                            self.score += 250;
                            self.skillpoint_count += 1;
                        }
                        CollectibleType::Attack(attacktype) => {
                            self.score += 500;
                            player.ammo = player.ammo_max;
                            player.attack = ATTACKS[&attacktype];
                            player.reload_timer = TriggerRepeating::new(player.attack.reload_time);
                        }
                    }
                }
                if let Some(_enemy) = self.world.get::<Enemy>(collision_entity).ok() {
                    if !player.invincible_timer.is_running() {
                        player_damage += 30.0;
                    }
                }
                if let Some(projectile) = self.world.get::<Projectile>(collision_entity).ok() {
                    if !player.invincible_timer.is_running() {
                        player_damage += projectile.damage;
                    }
                }
            }

            // BOOST
            let mut boost_active = false;
            player.speed_max = player.speed_base_max;
            if player.boost_allowed {
                if input.keyboard.is_down(Scancode::Up) {
                    player.speed_max = 1.5 * player.speed_base_max;
                    boost_active = true;
                }
                if input.keyboard.is_down(Scancode::Down) {
                    player.speed_max = 0.5 * player.speed_base_max;
                    boost_active = true;
                }
            } else {
                player.boost_cooldown_timer.update(deltatime);
                if player.boost_cooldown_timer.is_finished() {
                    player.boost_allowed = true;
                }
            }
            if boost_active {
                player.boost = f32::max(player.boost - 50.0 * deltatime, 0.0);
            } else {
                player.boost = f32::min(player.boost + 10.0 * deltatime, player.boost_max);
            }
            if player.boost == 0.0 {
                player.boost_allowed = false;
                player.boost_cooldown_timer = TimerSimple::new_started(player.boost_cooldown_time);
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
            let player_scale = Vec2::filled(player.size) / 4.0;

            // SHOOTING
            if player.reload_timer.update_and_check(deltatime) {
                player.ammo -= player.attack.ammo_consumption_on_shot;

                // Add muzzleflash
                let shoot_points_relative: Vec<Vec2> = linestrip_transform(
                    &get_shoot_points_for_ship(player.ship_type),
                    Vec2::zero(),
                    Vec2::zero(),
                    player_scale,
                    Vec2::unit_x(),
                    None,
                );
                let muzzle_pos_relative = shoot_points_relative.first().cloned().unwrap();
                self.commands.add_entity(Archetypes::new_muzzleflash(
                    player_entity,
                    muzzle_pos_relative,
                    45.0,
                ));

                // Add projectile(s)
                let shoot_points: Vec<Vec2> = linestrip_transform(
                    &get_shoot_points_for_ship(player.ship_type),
                    player_pos,
                    Vec2::zero(),
                    player_scale,
                    player_dir,
                    None,
                );
                let muzzle_pos_absolute = shoot_points.first().cloned().unwrap();

                match player.attack.typename {
                    AttackType::Neutral | AttackType::Rapid => {
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute,
                            player_dir,
                            200.0,
                            4.0,
                            player.attack.color,
                            100.0,
                        ));
                    }
                    AttackType::Double => {
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute,
                            player_dir.rotated(deg_to_rad(15.0)),
                            200.0,
                            4.0,
                            player.attack.color,
                            100.0,
                        ));
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute,
                            player_dir.rotated(deg_to_rad(-15.0)),
                            200.0,
                            4.0,
                            player.attack.color,
                            100.0,
                        ));
                    }
                    AttackType::Triple => {
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute,
                            player_dir,
                            200.0,
                            4.0,
                            player.attack.color,
                            100.0,
                        ));
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute,
                            player_dir.rotated(deg_to_rad(15.0)),
                            200.0,
                            4.0,
                            player.attack.color,
                            100.0,
                        ));
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute,
                            player_dir.rotated(deg_to_rad(-15.0)),
                            200.0,
                            4.0,
                            player.attack.color,
                            100.0,
                        ));
                    }
                    AttackType::Spread => {
                        let dir_angle_offset = globals.random.f32_in_range_closed(-20.0, 20.0);
                        let color = globals.random.pick_from_slice(&COLORS_ALL);
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute,
                            player_dir.rotated(deg_to_rad(dir_angle_offset)),
                            200.0,
                            4.0,
                            color,
                            100.0,
                        ));
                    }
                    AttackType::Back => {
                        let muzzle_pos_absolute_back =
                            player_pos + (player_pos - muzzle_pos_absolute);
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute,
                            player_dir,
                            200.0,
                            4.0,
                            player.attack.color,
                            100.0,
                        ));
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute_back,
                            -player_dir,
                            200.0,
                            4.0,
                            player.attack.color,
                            100.0,
                        ));
                    }
                    AttackType::Side => {
                        let muzzle_pos_absolute_left = player_pos
                            + (muzzle_pos_absolute - player_pos).rotated(deg_to_rad(90.0));
                        let muzzle_pos_absolute_right = player_pos
                            + (muzzle_pos_absolute - player_pos).rotated(deg_to_rad(-90.0));
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute,
                            player_dir,
                            200.0,
                            4.0,
                            player.attack.color,
                            100.0,
                        ));
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute_left,
                            player_dir.rotated(deg_to_rad(90.0)),
                            200.0,
                            4.0,
                            player.attack.color,
                            100.0,
                        ));
                        self.commands.add_entity(Archetypes::new_projectile(
                            muzzle_pos_absolute_right,
                            player_dir.rotated(deg_to_rad(-90.0)),
                            200.0,
                            4.0,
                            player.attack.color,
                            100.0,
                        ));
                    }
                }

                if player.ammo <= 0.0 {
                    player.ammo = player.ammo_max;
                    player.attack = ATTACKS[&AttackType::Neutral];
                    player.reload_timer = TriggerRepeating::new(player.attack.reload_time);
                }
            }

            // EXHAUST PARTICLES
            if player.timer_trail_particles.update_and_check(deltatime) {
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
                    let color = if boost_active {
                        COLOR_BOOST
                    } else {
                        COLOR_SKILL_POINT
                    };

                    self.commands
                        .add_entity(Archetypes::new_trailparticle(point, size, lifetime, color));
                }
            }

            // TICK EFFECT
            if player.cycle_timer.update_and_check_if_triggered(deltatime) {
                player.cycle_timer = TimerSimple::new_started(player.cycle_cooldown);
                self.commands
                    .add_entity(Archetypes::new_tick_effect(player_entity));
            }

            // TAKING DAMAGE
            if player_damage > 0.0 {
                player.hp = clampf(player.hp - player_damage, 0.0, player.hp_max);

                if player.hp == 0.0 {
                    self.commands.remove_entity(player_entity);
                } else {
                    if player_damage >= 30.0 {
                        player.invincible_timer.restart();
                    }
                }

                let (
                    screenshake_amplitude,
                    screenshake_duration,
                    screenshake_frequency,
                    slowmotion_duration,
                    slowmotion_factor,
                    particle_count_min,
                    particle_count_max,
                    screenflash_framecount,
                ) = {
                    if player.hp == 0.0 {
                        (6.0, 0.2, 80.0, 1.0, 0.15, 16, 32, 4)
                    } else {
                        if player_damage >= 30.0 {
                            (6.0, 0.2, 80.0, 0.5, 0.25, 8, 16, 3)
                        } else {
                            (6.0, 0.1, 80.0, 0.25, 0.75, 4, 8, 2)
                        }
                    }
                };

                let screen_shake = ModulatorScreenShake::new(
                    &mut globals.random,
                    screenshake_amplitude,
                    screenshake_duration,
                    screenshake_frequency,
                );
                globals.camera.add_shake(screen_shake);

                self.slowmotion
                    .add_slowmotion(slowmotion_duration, slowmotion_factor);

                self.commands.add_entity(Archetypes::new_screenflash(
                    globals.canvas_width,
                    globals.canvas_height,
                    screenflash_framecount,
                ));

                for _ in 0..globals
                    .random
                    .gen_range(particle_count_min, particle_count_max)
                {
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
        // UPDATE EXPLODE PARTICLES

        for (_entity, (xform, motion, particle, drawable)) in
            &mut self
                .world
                .query::<(&Transform, &mut Motion, &mut ExplodeParticle, &mut Drawable)>()
        {
            particle.timer_tween.update(deltatime);
            let percentage = particle.timer_tween.completion_ratio();
            let length = lerp(particle.length, 0.0, percentage);
            let speed = lerp(particle.speed, 0.0, percentage);
            let dir = Vec2::from_angle_flipped_y(deg_to_rad(xform.dir_angle));

            motion.vel = speed * dir;

            drawable.mesh = MeshType::LineWithThickness {
                length,
                thickness: particle.thickness,
                smooth_edges: false,
                centered: false,
            };
        }

        //------------------------------------------------------------------------------------------
        // UPDATE TICKEFFECT

        for (_entity, (tick, drawable)) in
            &mut self.world.query::<(&mut TickEffect, &mut Drawable)>()
        {
            tick.timer_tween.update(deltatime);
            let percentage = easing::cubic_inout(tick.timer_tween.completion_ratio());
            let width = tick.width;
            let height = lerp(tick.height, 0.0, percentage);
            let offset_y = lerp(0.0, -tick.height / 2.0, percentage);
            drawable.mesh = MeshType::Rectangle {
                width: width,
                height: height,
                filled: true,
                centered: true,
            };
            drawable.pos_offset = Vec2::filled_y(offset_y);
        }

        //------------------------------------------------------------------------------------------
        // UPDATE PROJECTILES

        for (entity, (xform, _projectile, collider)) in
            &mut self.world.query::<(&Transform, &Projectile, &Collider)>()
        {
            let mut explode = false;
            if !collider.collisions.is_empty() {
                explode = true;
            }
            let canvas_rect = Rect::from_width_height(globals.canvas_width, globals.canvas_height);
            if !canvas_rect.contains_point(xform.pos) {
                explode = true;
            }

            if explode {
                self.commands.remove_entity(entity);

                self.commands.add_entity(Archetypes::new_hit_effect(
                    xform.pos.clamped_to_rect(canvas_rect),
                    7.0,
                    7.0,
                    0.0,
                    COLOR_DEFAULT,
                    0.1,
                    COLOR_HP,
                    0.15,
                    true,
                ));
            }
        }

        //------------------------------------------------------------------------------------------
        // UPDATE COLLECTIBLES

        let mut infotext_create_buffer: Vec<InfoText> = Vec::new();

        for (entity, (xform, motion, collectible, collider)) in
            &mut self
                .world
                .query::<(&Transform, &Motion, &mut Collectible, &Collider)>()
        {
            let mut remove_self = false;
            let mut collected = false;

            // Check if collectible needs to be removed from game
            if !collider.collisions.is_empty() {
                remove_self = true;
                collected = true;
            }
            match collectible.collectible {
                CollectibleType::Ammo => {
                    // Follower collectibles
                    let canvas_rect =
                        Rect::from_width_height(globals.canvas_width, globals.canvas_height);
                    if !canvas_rect.contains_point(xform.pos) {
                        remove_self = true;
                    }
                }
                _ => {
                    // Horizontal moving collectibles
                    if motion.vel.x > 0.0 && xform.pos.x >= globals.canvas_width {
                        remove_self = true;
                    }
                    if motion.vel.x < 0.0 && xform.pos.x < 0.0 {
                        remove_self = true;
                    }
                }
            }

            if remove_self {
                self.commands.remove_entity(entity);

                // Create particles
                match collectible.collectible {
                    CollectibleType::Attack(_) => {}
                    _ => {
                        for _ in 0..globals.random.gen_range(4, 8) {
                            self.commands.add_entity(Archetypes::new_explode_particle(
                                xform.pos,
                                rad_to_deg(globals.random.vec2_in_unit_disk().to_angle_flipped_y()),
                                globals.random.f32_in_range_closed(50.0, 100.0),
                                globals.random.f32_in_range_closed(1.0, 2.0),
                                globals.random.f32_in_range_closed(3.0, 8.0),
                                globals.random.f32_in_range_closed(0.3, 0.5),
                                collectible.color,
                            ));
                        }
                    }
                };

                if !collected {
                    // Create explode effect
                    self.commands.add_entity(Archetypes::new_hit_effect(
                        xform.pos,
                        collectible.size,
                        collectible.size,
                        45.0,
                        COLOR_DEFAULT,
                        0.1,
                        collectible.color,
                        0.15,
                        true,
                    ));
                } else {
                    // Create collect effect
                    let text = collectible.collectible.get_infotext_string();
                    let text_pos = globals.random.vec2_in_disk(xform.pos, collider.radius);
                    infotext_create_buffer.push(InfoText::new(text_pos, text, collectible.color));

                    match collectible.collectible {
                        CollectibleType::Boost => {
                            // Inner
                            let entity = self.world.reserve_entity();
                            self.commands.add_component_bundle(
                                entity,
                                Archetypes::new_hit_effect(
                                    xform.pos,
                                    collectible.size,
                                    collectible.size,
                                    45.0,
                                    COLOR_DEFAULT,
                                    0.2,
                                    collectible.color,
                                    0.35,
                                    true,
                                ),
                            );
                            self.commands
                                .add_component(entity, Blinker::new(true, 0.2, 0.05));

                            // Outer
                            let entity = self.world.reserve_entity();
                            self.commands.add_component_bundle(
                                entity,
                                Archetypes::new_hit_effect(
                                    xform.pos,
                                    1.0,
                                    1.0,
                                    45.0,
                                    COLOR_DEFAULT,
                                    0.2,
                                    collectible.color,
                                    0.35,
                                    false,
                                ),
                            );
                            self.commands
                                .add_component(entity, Blinker::new(true, 0.2, 0.05));
                            self.commands.add_component(
                                entity,
                                TweenScale::new(
                                    collectible.size,
                                    2.5 * collectible.size,
                                    0.35,
                                    EasingType::CubicInOut,
                                ),
                            );
                        }
                        CollectibleType::Ammo => {
                            self.commands.add_entity(Archetypes::new_hit_effect(
                                xform.pos,
                                collectible.size,
                                collectible.size,
                                45.0,
                                COLOR_DEFAULT,
                                0.1,
                                collectible.color,
                                0.15,
                                true,
                            ));
                        }
                        CollectibleType::Hp => {
                            // Inner vertical
                            let entity = self.world.reserve_entity();
                            self.commands.add_component_bundle(
                                entity,
                                Archetypes::new_hit_effect(
                                    xform.pos,
                                    1.2 * collectible.size / 3.0,
                                    1.2 * collectible.size,
                                    0.0,
                                    COLOR_DEFAULT,
                                    0.2,
                                    collectible.color,
                                    0.35,
                                    true,
                                ),
                            );
                            self.commands
                                .add_component(entity, Blinker::new(true, 0.2, 0.05));

                            // Inner horizontal
                            let entity = self.world.reserve_entity();
                            self.commands.add_component_bundle(
                                entity,
                                Archetypes::new_hit_effect(
                                    xform.pos,
                                    1.2 * collectible.size,
                                    1.2 * collectible.size / 3.0,
                                    0.0,
                                    COLOR_DEFAULT,
                                    0.2,
                                    collectible.color,
                                    0.35,
                                    true,
                                ),
                            );
                            self.commands
                                .add_component(entity, Blinker::new(true, 0.2, 0.05));

                            // Outer
                            let entity = self.world.reserve_entity();
                            self.commands.add_component_bundle(
                                entity,
                                Archetypes::new_hit_effect_round(
                                    xform.pos,
                                    1.0,
                                    collectible.color,
                                    0.2,
                                    COLOR_DEFAULT,
                                    0.35,
                                    false,
                                ),
                            );
                            self.commands
                                .add_component(entity, Blinker::new(true, 0.2, 0.05));
                            self.commands.add_component(
                                entity,
                                TweenScale::new(
                                    1.2 * collectible.size,
                                    1.7 * collectible.size,
                                    0.35,
                                    EasingType::CubicInOut,
                                ),
                            );
                        }
                        CollectibleType::Skillpoint => {
                            // Inner
                            let entity = self.world.reserve_entity();
                            self.commands.add_component_bundle(
                                entity,
                                Archetypes::new_hit_effect(
                                    xform.pos,
                                    collectible.size,
                                    collectible.size,
                                    45.0,
                                    COLOR_DEFAULT,
                                    0.2,
                                    collectible.color,
                                    0.35,
                                    true,
                                ),
                            );
                            self.commands
                                .add_component(entity, Blinker::new(true, 0.2, 0.05));

                            // Outer
                            let entity = self.world.reserve_entity();
                            self.commands.add_component_bundle(
                                entity,
                                Archetypes::new_hit_effect(
                                    xform.pos,
                                    1.0,
                                    1.0,
                                    45.0,
                                    COLOR_DEFAULT,
                                    0.2,
                                    collectible.color,
                                    0.35,
                                    false,
                                ),
                            );
                            self.commands
                                .add_component(entity, Blinker::new(true, 0.2, 0.05));
                            self.commands.add_component(
                                entity,
                                TweenScale::new(
                                    collectible.size,
                                    2.5 * collectible.size,
                                    0.35,
                                    EasingType::CubicInOut,
                                ),
                            );
                        }
                        CollectibleType::Attack(_) => {
                            // Inner
                            let entity = self.world.reserve_entity();
                            self.commands.add_component_bundle(
                                entity,
                                Archetypes::new_hit_effect(
                                    xform.pos,
                                    1.0,
                                    1.0,
                                    45.0,
                                    COLOR_DEFAULT,
                                    0.2,
                                    COLOR_DEFAULT,
                                    0.35,
                                    false,
                                ),
                            );
                            self.commands
                                .add_component(entity, Blinker::new(true, 0.2, 0.05));
                            self.commands.add_component(
                                entity,
                                TweenScale::new(
                                    1.1 * collectible.size,
                                    3.1 * collectible.size,
                                    0.35,
                                    EasingType::CubicInOut,
                                ),
                            );

                            // Outer
                            let entity = self.world.reserve_entity();
                            self.commands.add_component_bundle(
                                entity,
                                Archetypes::new_hit_effect(
                                    xform.pos,
                                    1.0,
                                    1.0,
                                    45.0,
                                    collectible.color,
                                    0.2,
                                    collectible.color,
                                    0.35,
                                    false,
                                ),
                            );
                            self.commands
                                .add_component(entity, Blinker::new(true, 0.2, 0.05));
                            self.commands.add_component(
                                entity,
                                TweenScale::new(
                                    1.4 * collectible.size,
                                    3.4 * collectible.size,
                                    0.35,
                                    EasingType::CubicInOut,
                                ),
                            );
                        }
                    }
                }
            }
        }

        //------------------------------------------------------------------------------------------
        // TWEENERS

        for (_entity, (tween_scale, drawable)) in
            &mut self.world.query::<(&mut TweenScale, &mut Drawable)>()
        {
            tween_scale.update(drawable, deltatime);
        }

        for (_entity, (tween_color, drawable)) in
            &mut self.world.query::<(&mut TweenColor, &mut Drawable)>()
        {
            tween_color.update(drawable, deltatime);
        }

        //------------------------------------------------------------------------------------------
        // BLINKER

        for (_entity, (blinker, drawable)) in
            &mut self.world.query::<(&mut Blinker, &mut Drawable)>()
        {
            drawable.visible = blinker.update_and_check(deltatime);
        }

        //------------------------------------------------------------------------------------------
        // INFOTEXT

        {
            let gui_font = self.fonts.get("gui_font").unwrap();
            for (entity, infotext) in &mut self.world.query::<&mut InfoText>() {
                if infotext.update_and_check_if_finished(
                    draw,
                    &mut globals.random,
                    gui_font,
                    deltatime,
                ) {
                    self.commands.remove_entity(entity);
                }
            }
        }

        //------------------------------------------------------------------------------------------
        // ENEMY CHARGE PARTICLES

        for (entity, particle) in &mut self.world.query::<&mut EnemyChargeParticle>() {
            if particle.timer.update_and_check_if_triggered(deltatime) {
                self.commands.remove_entity(entity);
            }

            if let Some(muzzle_entity_xform) =
                self.world.get::<Transform>(particle.muzzle_entity).ok()
            {
                particle.muzzle_pos = muzzle_entity_xform.pos + particle.muzzle_pos_offset;
            }

            let pos = Vec2::lerp(
                particle.start_pos,
                particle.muzzle_pos,
                particle.timer.completion_ratio(),
            );
            let size = lerp(particle.size, 0.0, particle.timer.completion_ratio());

            draw.draw_rect_transformed(
                Vec2::new(size, size),
                Vec2::new(size, size) / 2.0,
                pos,
                Vec2::ones(),
                Vec2::from_angle_flipped_y(deg_to_rad(45.0)),
                DEPTH_EFFECTS,
                COLOR_HP,
                ADDITIVITY_NONE,
            );
        }

        //------------------------------------------------------------------------------------------
        // DRAWING

        for (_entity, (xform, multi_drawable)) in
            &mut self.world.query::<(&Transform, &DrawableMulti)>()
        {
            for drawable in &multi_drawable.drawables {
                draw_drawable(&self.fonts, draw, globals, xform, drawable);
            }
        }
        for (_entity, (xform, drawable)) in &mut self.world.query::<(&Transform, &Drawable)>() {
            draw_drawable(&self.fonts, draw, globals, xform, drawable);
        }

        //------------------------------------------------------------------------------------------
        // DEBUG DRAWING

        if DEBUG_DRAW_ENABLE {
            // Colliders
            for (_entity, (xform, collider)) in &mut self.world.query::<(&Transform, &Collider)>() {
                let color = if collider.collisions.len() > 0 {
                    Color::red()
                } else {
                    Color::yellow()
                };
                draw.draw_circle_bresenham(
                    xform.pos,
                    collider.radius,
                    DEPTH_DEBUG,
                    color,
                    ADDITIVITY_NONE,
                );
            }
        }

        //------------------------------------------------------------------------------------------
        // CREATE / DELETE ENTITIES

        // INFOTEXT CREATION
        {
            let gui_font = self.fonts.get("gui_font").unwrap();
            for mut infotext_to_create in infotext_create_buffer.drain(..) {
                // Collect existing infotext bounding boxes
                let text_rects_existing: Vec<Recti> = {
                    let mut result = Vec::new();
                    for (_entity, infotext) in &mut self.world.query::<&InfoText>() {
                        let infotext: &InfoText = infotext;
                        let text: String = infotext.text.iter().collect();
                        let rect = gui_font
                            .get_text_bounding_rect(&text, 1, false)
                            .translated_by(infotext.pos.pixel_snapped_i32());
                        result.push(rect);
                    }
                    result
                };

                // Check our bounding box against existing ones so that it does not overlap with
                // any existing
                let text_rect = {
                    let text: String = infotext_to_create.text.iter().collect();
                    gui_font
                        .get_text_bounding_rect(&text, 1, false)
                        .translated_by(infotext_to_create.pos.pixel_snapped_i32())
                };
                infotext_to_create.pos = text_rect
                    .get_closest_position_without_overlapping(&text_rects_existing)
                    .into();

                self.world.spawn((infotext_to_create,));
            }
        }

        self.commands.execute(&mut self.world);
    }
}
