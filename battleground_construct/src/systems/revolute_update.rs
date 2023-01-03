use super::components::pose::Pose;
use super::components::revolute::Revolute;
use super::Clock;
use crate::components::velocity::Velocity;
use engine::prelude::*;

pub struct RevoluteUpdate {}
impl System for RevoluteUpdate {
    fn update(&mut self, world: &mut World) {
        let (_entity, clock) = world
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
        let dt = clock.step_as_f32();

        for (entity, mut rev) in world.component_iter_mut::<Revolute>() {
            rev.update(dt);
            if let Some(mut vel) = world.component_mut::<Velocity>(entity) {
                *vel = rev.to_twist().into();
            }
            if let Some(mut pose) = world.component_mut::<Pose>(entity) {
                let created_pose = rev.to_pose();
                *pose = created_pose;
            }
        }
    }
}
