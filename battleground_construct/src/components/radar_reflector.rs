use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct RadarReflector {
    pub reflectivity: f32,
}

impl RadarReflector {
    pub fn new() -> Self {
        Self {
            reflectivity: 1.0,
        }
    }

}
impl Component for RadarReflector {}
