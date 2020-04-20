// THIS FILE IS CURRENTLY UNUSED UNTIL WE FIGURE OUT WHAT TO DO WITH PHYSICS

const COLLISION_LAYER_CIRCLES: u64 = 1 << 0;
const COLLISION_LAYER_RECTANGLES: u64 = 1 << 1;

struct Test {
    bodies: Vec<Body>,
    contact_points: Vec<(usize, Vec2, Vec2)>,
}

impl Test {
    fn test(&mut self) {
        //------------------------------------------------------------------------------------------
        // UPDATE PHYSICS

        if self.bodies.len() == 0 {
            let pos = globals.random.vec2_in_rect(Rect::from_width_height(
                globals.canvas_width,
                globals.canvas_height,
            ));
            let shape = Shape::Disk { radius: 10.0 };
            let bouncyness = 1.0;
            let mass = 2.0;
            let inertia = 2.0;
            self.bodies.push(Body::new(
                COLLISION_LAYER_CIRCLES,
                COLLISION_LAYER_CIRCLES | COLLISION_LAYER_RECTANGLES,
                pos,
                0.0,
                shape,
                mass,
                inertia,
                bouncyness,
            ));
        }
        self.bodies[0].pos = globals.cursors.mouse_coords.pos_canvas;
        self.bodies[0].vel = if deltatime != 0.0 {
            globals.cursors.mouse_coords.delta_canvas / deltatime
        } else {
            Vec2::zero()
        };
        let cursor_canvas = globals.cursors.mouse_coords.pos_canvas;
        let cursor_world = globals.cursors.mouse_coords.pos_world;
        let cursor_screen = globals.cursors.mouse_coords.pos_screen;
        draw.debug_log(dformat!(cursor_canvas));
        draw.debug_log(dformat!(cursor_world));
        draw.debug_log(dformat!(cursor_screen));

        // Add static objects
        if self.bodies.len() == 1 {
            // static circles
            for _ in 0..5 {
                let pos = globals.random.vec2_in_rect(Rect::from_width_height(
                    globals.canvas_width,
                    globals.canvas_height,
                ));
                let shape = Shape::Disk {
                    radius: globals.random.f32_in_range_closed(5.0, 15.0),
                };
                let bouncyness = globals.random.f32_in_range_closed(0.5, 1.0);
                let mass = 0.0;
                let inertia = 0.0;
                self.bodies.push(Body::new(
                    COLLISION_LAYER_CIRCLES,
                    COLLISION_LAYER_CIRCLES | COLLISION_LAYER_RECTANGLES,
                    pos,
                    0.0,
                    shape,
                    mass,
                    inertia,
                    bouncyness,
                ));
            }
            // static boxes
            for _ in 0..5 {
                let pos = globals.random.vec2_in_rect(Rect::from_width_height(
                    globals.canvas_width,
                    globals.canvas_height,
                ));
                let dim = Vec2::new(
                    globals.random.f32_in_range_closed(10.0, 20.0),
                    globals.random.f32_in_range_closed(10.0, 20.0),
                );
                let angle = globals.random.f32_in_range_closed(0.0, 360.0);
                let shape = Shape::Box {
                    width: dim.x,
                    height: dim.y,
                };
                let bouncyness = globals.random.f32_in_range_closed(0.5, 1.0);
                let mass = 0.0;
                let inertia = 0.0;
                self.bodies.push(Body::new(
                    COLLISION_LAYER_RECTANGLES,
                    COLLISION_LAYER_CIRCLES,
                    pos,
                    angle,
                    shape,
                    mass,
                    inertia,
                    bouncyness,
                ));
            }
        }

        // Add dynamic objects
        if self.bodies.len() < 30 {
            // moving circles
            {
                let canvas_rect =
                    Rect::from_width_height(globals.canvas_width, globals.canvas_height);
                let pos = loop {
                    let point = globals
                        .random
                        .vec2_in_rect(canvas_rect.extended_uniformly_by(100.0));
                    if !canvas_rect.contains_point(point) {
                        break point;
                    }
                };
                let dir = ((canvas_rect.center()
                    + (globals.canvas_height / 2.0) * globals.random.vec2_in_unit_disk())
                    - pos)
                    .normalized();
                let vel = dir * globals.random.f32_in_range_closed(100.0, 200.0);

                let shape = Shape::Disk {
                    radius: globals.random.f32_in_range_closed(1.0, 20.0),
                };
                let bouncyness = globals.random.f32_in_range_closed(0.5, 1.0);
                let mass = globals.random.f32_in_range_closed(1.0, 2.0);
                let inertia = mass;
                let mut body = Body::new(
                    COLLISION_LAYER_CIRCLES,
                    COLLISION_LAYER_CIRCLES | COLLISION_LAYER_RECTANGLES,
                    pos,
                    0.0,
                    shape,
                    mass,
                    inertia,
                    bouncyness,
                );
                body.vel = vel;
                self.bodies.push(body);
            }
            // moving boxes
            {
                let canvas_rect =
                    Rect::from_width_height(globals.canvas_width, globals.canvas_height);
                let pos = loop {
                    let point = globals
                        .random
                        .vec2_in_rect(canvas_rect.extended_uniformly_by(100.0));
                    if !canvas_rect.contains_point(point) {
                        break point;
                    }
                };
                let dim = Vec2::new(
                    globals.random.f32_in_range_closed(10.0, 20.0),
                    globals.random.f32_in_range_closed(10.0, 20.0),
                );
                let angle = globals.random.f32_in_range_closed(0.0, 360.0);
                let shape = Shape::Box {
                    width: dim.x,
                    height: dim.y,
                };

                let dir = ((canvas_rect.center()
                    + (globals.canvas_height / 2.0) * globals.random.vec2_in_unit_disk())
                    - pos)
                    .normalized();
                let vel = dir * globals.random.f32_in_range_closed(100.0, 200.0);

                let bouncyness = globals.random.f32_in_range_closed(0.5, 1.0);
                let mass = globals.random.f32_in_range_closed(1.0, 2.0);
                let inertia = mass;
                let mut body = Body::new(
                    COLLISION_LAYER_RECTANGLES,
                    COLLISION_LAYER_CIRCLES,
                    pos,
                    angle,
                    shape,
                    mass,
                    inertia,
                    bouncyness,
                );
                body.vel = vel;
                self.bodies.push(body);
            }
        }

        // Broadphase: Find collisions
        let mut pairs: HashSet<(usize, usize)> = HashSet::new();
        for index_a in 0..self.bodies.len() {
            for index_b in 0..self.bodies.len() {
                if index_a == index_b {
                    continue;
                }

                let body_a = &self.bodies[index_a];
                let body_b = &self.bodies[index_b];

                if body_a.layers & body_b.layers_affects == 0
                    && body_b.layers & body_a.layers_affects == 0
                {
                    continue;
                }

                let a_aabb = body_a.compute_aabb();
                let b_aabb = body_b.compute_aabb();

                if intersects_rect_rect(a_aabb, b_aabb) {
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
            let mut body_a = self.bodies[index_a];
            let mut body_b = self.bodies[index_b];
            if let Some(contact) = Body::collide(&body_a, &body_b) {
                collision_resolve(&mut body_a, &mut body_b, &contact);
                // collision_positional_correction(&mut a.obj, &mut b.obj, &manifold);

                for _ in 0..5 {
                    self.contact_points.push((0, contact.point, contact.normal));
                }

                self.bodies[index_a] = body_a;
                self.bodies[index_b] = body_b;
            }
        }

        for (drawcount, point, normal) in self.contact_points.iter_mut() {
            draw.circle_filled(*point, 2.0, DEPTH_DEBUG, Color::magenta(), ADDITIVITY_NONE);
            draw.debug_arrow(*point, 5.0 * *normal, Color::yellow(), ADDITIVITY_NONE);
            *drawcount += 1;
        }
        self.contact_points
            .retain(|(drawcount, _, _)| *drawcount <= 20);

        // Update forces
        for body in self.bodies.iter_mut() {
            body.force = Vec2::zero();
            let gravity = body.mass * Vec2::filled_y(50.0);
            // body.force += gravity;

            body.vel += body.force * body.mass_inverse * deltatime;
        }

        // Update objects
        for body in self.bodies.iter_mut() {
            body.pos += body.vel * deltatime;
            body.angle += body.angle_vel * deltatime;
        }

        // Draw objects
        for body in &self.bodies {
            let color = if body.mass == 0.0 {
                Color::green()
            } else {
                Color::white()
            };
            draw.pixel(body.pos, DEPTH_PROJECTILE, color, ADDITIVITY_NONE);
            match body.shape {
                Shape::Disk { radius } => {
                    draw.circle_bresenham(
                        body.pos,
                        radius,
                        DEPTH_PROJECTILE,
                        color,
                        ADDITIVITY_NONE,
                    );
                    draw.line_bresenham(
                        body.pos,
                        body.pos + radius * Vec2::from_angle_flipped_y(deg_to_rad(body.angle)),
                        DEPTH_PROJECTILE,
                        color,
                        ADDITIVITY_NONE,
                    );
                    draw.pixel(body.pos, DEPTH_PROJECTILE, Color::red(), ADDITIVITY_NONE);
                }
                Shape::Box { width, height } => {
                    draw.rect_transformed(
                        Vec2::new(width, height),
                        Vec2::new(width, height) / 2.0,
                        body.pos,
                        Vec2::ones(),
                        Vec2::from_angle_flipped_y(deg_to_rad(body.angle)),
                        DEPTH_PROJECTILE,
                        color,
                        ADDITIVITY_NONE,
                    );
                    draw.line_bresenham(
                        body.pos,
                        body.pos + 0.5 * width * Vec2::from_angle_flipped_y(deg_to_rad(body.angle)),
                        DEPTH_PROJECTILE,
                        Color::red(),
                        ADDITIVITY_NONE,
                    );
                    draw.pixel(body.pos, DEPTH_PROJECTILE, Color::red(), ADDITIVITY_NONE);
                }
            }
        }

        // Remove objects that are out of bounds
        let canvas_rect = Rect::from_width_height(globals.canvas_width, globals.canvas_height);
        self.bodies.retain(|body| {
            canvas_rect
                .extended_uniformly_by(100.0)
                .contains_point(body.pos)
        });
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Physics

// Based on
// https://gamedevelopment.tutsplus.com/tutorials/how-to-create-a-custom-2d-physics-engine-the-basics-and-impulse-resolution--gamedev-6331

#[derive(Clone, Copy)]
enum Shape {
    Box { width: f32, height: f32 },
    Disk { radius: f32 },
}

impl Shape {
    fn compute_volume(&self) -> f32 {
        match self {
            Shape::Disk { radius } => PI * squared(*radius),
            Shape::Box { width, height, .. } => width * height,
        }
    }

    fn compute_inertia(&self, mass: f32) -> f32 {
        // From https://en.wikipedia.org/wiki/List_of_moments_of_inertia
        match self {
            Shape::Disk { radius } => mass * squared(*radius),
            Shape::Box { width, height, .. } => {
                (1.0 / 12.0) * mass * (squared(*width) + squared(*height))
            }
        }
    }

    fn compute_mass(&self, density: f32) -> MassData {
        let mass = density * self.compute_volume();
        let mass_inverse = if mass == 0.0 { 0.0 } else { 1.0 / mass };

        let inertia = self.compute_inertia(mass);
        let inertia_inverse = if inertia == 0.0 { 0.0 } else { 1.0 / inertia };

        MassData {
            mass,
            mass_inverse,
            inertia,
            inertia_inverse,
        }
    }
}

#[derive(Clone, Copy)]
struct MassData {
    pub mass: f32,
    pub mass_inverse: f32,

    pub inertia: f32,
    pub inertia_inverse: f32,
}

#[derive(Clone, Copy)]
struct Body {
    pub layers: u64,
    pub layers_affects: u64,

    pub shape: Shape,

    pub bouncyness: f32,
    pub mass: f32,
    pub mass_inverse: f32,
    pub inertia: f32,
    pub inertia_inverse: f32,

    pub pos: Vec2,
    pub vel: Vec2,
    pub force: Vec2,

    pub angle: f32,
    pub angle_vel: f32,
    pub angle_force: f32,
}

impl Body {
    fn new(
        layers: u64,
        layers_affects: u64,
        pos: Vec2,
        angle: f32,
        shape: Shape,
        mass: f32,
        inertia: f32,
        bouncyness: f32,
    ) -> Body {
        let mass_inverse = if mass == 0.0 { 0.0 } else { 1.0 / mass };
        let inertia_inverse = if inertia == 0.0 { 0.0 } else { 1.0 / inertia };

        Body {
            layers,
            layers_affects,

            shape,

            bouncyness,
            mass,
            mass_inverse,
            inertia,
            inertia_inverse,

            pos,
            vel: Vec2::zero(),
            force: Vec2::zero(),

            angle,
            angle_vel: 0.0,
            angle_force: 0.0,
        }
    }

    fn compute_aabb(&self) -> Rect {
        match self.shape {
            Shape::Disk { radius } => {
                Rect::from_square(2.0 * radius).centered_in_position(self.pos)
            }
            Shape::Box { width, height } => {
                Rect::from_width_height(2.0 * width, 2.0 * height).centered_in_position(self.pos)
                //TODO: angle
            }
        }
    }

    fn collide(a: &Body, b: &Body) -> Option<Contact> {
        match a.shape {
            Shape::Disk { radius } => {
                let a_circle = Circle {
                    center: a.pos,
                    radius,
                };
                match b.shape {
                    Shape::Disk { radius } => {
                        let b_circle = Circle {
                            center: b.pos,
                            radius,
                        };
                        collide_circle_circle(a_circle, b_circle)
                    }
                    Shape::Box { width, height } => {
                        let b_rect =
                            Rect::from_width_height(width, height).centered_in_position(b.pos);
                        collide_box_circle(b_rect, b.angle, a_circle).map(|contact| Contact {
                            normal: -contact.normal,
                            penetration_depth: contact.penetration_depth,
                            point: contact.point,
                        })
                    }
                }
            }
            Shape::Box { width, height } => {
                let a_rect = Rect::from_width_height(width, height).centered_in_position(a.pos);
                match b.shape {
                    Shape::Disk { radius } => {
                        let b_circle = Circle {
                            center: b.pos,
                            radius,
                        };
                        collide_box_circle(a_rect, a.angle, b_circle)
                    }
                    Shape::Box { width, height } => {
                        let b_rect =
                            Rect::from_width_height(width, height).centered_in_position(b.pos);
                        collide_box_box(a_rect, a.angle, b_rect, b.angle)
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Contact {
    pub penetration_depth: f32,
    pub normal: Vec2,
    pub point: Vec2,
}

fn collide_circle_circle(a: Circle, b: Circle) -> Option<Contact> {
    let vec_a_to_b = b.center - a.center;
    let distance_squared = vec_a_to_b.magnitude_squared();
    let radius_sum_squared = squared(a.radius + b.radius);
    if distance_squared > radius_sum_squared {
        // The circles do not touch
        return None;
    }

    let distance = f32::sqrt(distance_squared);
    if distance != 0.0 {
        let normal = vec_a_to_b / distance;
        Some(Contact {
            penetration_depth: f32::sqrt(radius_sum_squared) - distance,
            normal,
            point: ((a.center + a.radius * normal) + (b.center + b.radius * -normal)) / 2.0,
        })
    } else {
        // Circles are on the same position -> choose arbitrary but fixed values
        Some(Contact {
            penetration_depth: a.radius,
            normal: Vec2::unit_x(),
            point: a.center,
        })
    }
}

fn collide_box_box(a: Rect, a_angle: f32, b: Rect, b_angle: f32) -> Option<Contact> {
    assert!(
        a_angle == 0.0 && b_angle == 0.0,
        "Box vs box collision only supports real AABBs only"
    );
    let half_dim_a = a.dim / 2.0;
    let half_dim_b = b.dim / 2.0;

    let center_a = a.center();
    let center_b = b.center();
    let vec_a_to_b = center_b - center_a;

    let overlap_x = (half_dim_a.x + half_dim_b.x) - f32::abs(vec_a_to_b.x);
    let overlap_y = (half_dim_a.y + half_dim_b.y) - f32::abs(vec_a_to_b.y);

    if overlap_x > 0.0 && overlap_y > 0.0 {
        if overlap_x < overlap_y {
            let normal = if vec_a_to_b.x > 0.0 {
                // A is left of B
                Vec2::new(1.0, 0.0)
            } else {
                // A is right of B
                Vec2::new(-1.0, 0.0)
            };
            let penetration_depth = overlap_x;
            return Some(Contact {
                penetration_depth,
                normal,
                point: Vec2::zero(), // TODO
            });
        } else {
            let normal = if vec_a_to_b.y > 0.0 {
                // A is above of B
                Vec2::new(0.0, 1.0)
            } else {
                // A is below of B
                Vec2::new(0.0, -1.0)
            };
            let penetration_depth = overlap_x;
            return Some(Contact {
                penetration_depth,
                normal,
                point: Vec2::zero(), // TODO
            });
        }
    }
    None
}

fn collide_box_circle(rect: Rect, rect_angle: f32, circle: Circle) -> Option<Contact> {
    if rect_angle == 0.0 {
        // Simple case
        return collide_rect_circle(rect, circle);
    }

    let rect_center = rect.center();
    let circle_transformed_center =
        (circle.center - rect_center).rotated_flipped_y(-deg_to_rad(rect_angle)) + rect_center;
    collide_rect_circle(
        rect,
        Circle {
            radius: circle.radius,
            center: circle_transformed_center,
        },
    )
    .map(|contact| Contact {
        normal: contact.normal.rotated_flipped_y(deg_to_rad(rect_angle)),
        penetration_depth: contact.penetration_depth,
        point: (contact.point - rect_center).rotated_flipped_y(deg_to_rad(rect_angle))
            + rect_center,
    })
}

fn collide_rect_circle(rect: Rect, circle: Circle) -> Option<Contact> {
    let rect_halfdim = rect.dim / 2.0;
    let rect_center = rect.center();
    let vec_rect_to_circle = circle.center - rect_center;

    let mut closest_point_on_rect = Vec2::new(
        clampf(vec_rect_to_circle.x, -rect_halfdim.x, rect_halfdim.x),
        clampf(vec_rect_to_circle.y, -rect_halfdim.y, rect_halfdim.y),
    );

    let circle_center_inside_rect = closest_point_on_rect == vec_rect_to_circle;
    if circle_center_inside_rect {
        // Clamp the circle's center to the nearest edge of the AABB
        if f32::abs(vec_rect_to_circle.x) > f32::abs(vec_rect_to_circle.y) {
            if closest_point_on_rect.x > 0.0 {
                // Our circle-center is closest to the right edge of the AABB
                closest_point_on_rect.x = rect_halfdim.x;
            } else {
                // Our circle-center is closest to the left edge of the AABB
                closest_point_on_rect.x = -rect_halfdim.x;
            }
        } else {
            if closest_point_on_rect.y > 0.0 {
                // Our circle-center is closest to the bottom edge of the AABB
                closest_point_on_rect.y = rect_halfdim.y;
            } else {
                // Our circle-center is closest to the top edge of the AABB
                closest_point_on_rect.y = -rect_halfdim.y;
            }
        }
    }

    let normal = vec_rect_to_circle - closest_point_on_rect;
    let distance_squared = normal.magnitude_squared();

    if distance_squared > squared(circle.radius) && !circle_center_inside_rect {
        // The circle is too far away
        return None;
    }

    let distance = f32::sqrt(distance_squared);

    if circle_center_inside_rect {
        // We need to flip the normal so that it points outside the rect
        Some(Contact {
            penetration_depth: circle.radius - distance,
            normal: -normal / distance,
            point: circle.center,
        })
    } else {
        Some(Contact {
            penetration_depth: circle.radius - distance,
            normal: normal / distance,
            point: rect_center + closest_point_on_rect,
        })
    }
}

fn collision_resolve(a: &mut Body, b: &mut Body, collision: &Contact) {
    if a.mass == 0.0 && b.mass == 0.0 {
        // Nothing to do
        return;
    }

    let vel_relative = b.vel - a.vel;
    let vel_along_normal = Vec2::dot(vel_relative, collision.normal);

    if vel_along_normal > 0.0 {
        // Object are moving away from each other
        return;
    }

    let bouncyness = f32::min(a.bouncyness, b.bouncyness);
    let impulse_along_normal =
        (-(1.0 + bouncyness) * vel_along_normal) / (a.mass_inverse + b.mass_inverse);
    let impulse_vector = impulse_along_normal * collision.normal;

    a.vel -= impulse_vector * a.mass_inverse;
    b.vel += impulse_vector * b.mass_inverse;

    let a_r = collision.point - a.pos;
    let b_r = collision.point - b.pos;
    a.angle_vel -= rad_to_deg(Vec2::cross_z(a_r, collision.normal) * a.inertia_inverse);
    b.angle_vel -= rad_to_deg(Vec2::cross_z(b_r, collision.normal) * b.inertia_inverse);
}

fn collision_positional_correction(a: &mut Body, b: &mut Body, collision: &Contact) {
    if a.mass == 0.0 && b.mass == 0.0 {
        // Nothing to do
        return;
    }

    let slop = 0.05;
    let percent = 0.4;
    let penetration_with_slop = f32::max(collision.penetration_depth - slop, 0.0);
    let correction =
        (penetration_with_slop / (a.mass_inverse + b.mass_inverse)) * percent * collision.normal;
    a.pos -= a.mass_inverse * correction;
    b.pos += b.mass_inverse * correction;
}

// We may get away with just using circles and oriented rectangles?
// Do we even need oriented rectangle vs aabb collisions?
// We can do collision detection with minkowski sums and lines (which we already have done)
// We still need collision detection between lines and oriented rectangles (we could do that with
// with inverse transformation of the rectangle?) -> no just line intersection will be enough
// Restrictions:
// - only circles_vs_circle, circle_vs_oriented_rect, aabb_vs_oriented_rect allowed
// - minkowski sum between rectangles requires one rectangle to be axis aligned
//   (its a little easier that way and we later can add minkowski sums between arbitrary rectangles)
// - collisions will operate on points vs sums or lines vs sums only
// - no friction only damping
// Open questions:
// - How do we get the contact points of the collision in case we are doing
//   rectange-rectangle collisions?
