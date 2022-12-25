use super::primitives::*;
use engine::prelude::*;

#[derive(Copy, Debug, Clone)]
pub struct DisplayControlPoint {
    pub radius: f32,
    color: Color,
}
impl Default for DisplayControlPoint {
    fn default() -> Self {
        DisplayControlPoint::new()
    }
}

impl DisplayControlPoint {
    pub fn new() -> Self {
        DisplayControlPoint {
            radius: 1.0,
            color: Color {
                r: 128,
                g: 128,
                b: 128,
                a: 128,
            },
        }
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
        self.color.a = 128;
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
}
impl Component for DisplayControlPoint {}

impl Drawable for DisplayControlPoint {
    fn drawables(&self) -> Vec<Element> {
        let material = Material::FlatMaterial(FlatMaterial {
            color: self.color,
            is_transparent: true,
            ..Default::default()
        });
        vec![Element {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.01)),
            primitive: Primitive::Circle(Circle {
                radius: self.radius,
                subdivisions: 30,
            }),
            material,
        }]
    }
}
