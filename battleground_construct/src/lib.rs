// https://rust-lang.github.io/api-guidelines/naming.html

/*
    Todo:
        - Propagate velocities such that bullets get the correct initial velocity.
*/

pub mod components;
pub mod display;
pub mod systems;
use crate::display::primitives::Vec3;
use components::clock::{Clock, ClockSystem};
use engine::prelude::*;
use engine::Systems;

pub struct Construct {
    world: World,
    systems: Systems,
}

impl Construct {
    pub fn new() -> Self {
        let mut world = World::new();
        let clock_id = world.add_entity();
        world.add_component(&clock_id, Clock::new());

        for x in 0..1 {
            for y in 0..1 {
                let vehicle_id = world.add_entity();
                let mut pose = components::pose::Pose::new();
                pose.h.w[0] = (x as f32) * 5.0;
                pose.h.w[1] = (y as f32) * 5.0;
                world.add_component(&vehicle_id, pose);
                world.add_component(&vehicle_id, components::velocity::Velocity::new());
                let mut base = components::differential_drive_base::DifferentialDriveBase::new();
                base.set_velocities(0.6, 0.8);
                world.add_component(&vehicle_id, base);
                world.add_component(&vehicle_id, display::tank_body::TankBody::new());

                let turret_id = world.add_entity();
                let mut turret_revolute =
                    components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 0.0, 1.0));
                turret_revolute.set_velocity(0.1);

                world.add_component(&turret_id, turret_revolute);
                world.add_component(
                    &turret_id,
                    components::pose::PreTransform::from_translation(Vec3::new(
                        0.0,
                        0.0,
                        0.375 + 0.1 / 2.0,
                    )),
                );
                world.add_component(&turret_id, components::pose::Pose::new());
                world.add_component(
                    &turret_id,
                    components::parent::Parent::new(vehicle_id.clone()),
                );
                world.add_component(&turret_id, display::tank_turret::TankTurret::new());

                let barrel_id = world.add_entity();
                let mut barrel_revolute =
                    components::revolute::Revolute::new_with_axis(Vec3::new(0.0, 1.0, 0.0));
                barrel_revolute.set_velocity(-0.1);
                world.add_component(&barrel_id, barrel_revolute);
                world.add_component(
                    &barrel_id,
                    components::pose::PreTransform::from_translation(Vec3::new(0.25, 0.0, 0.0)),
                );
                world.add_component(&barrel_id, components::pose::Pose::new());
                world.add_component(
                    &barrel_id,
                    components::parent::Parent::new(turret_id.clone()),
                );
                world.add_component(&barrel_id, display::tank_barrel::TankBarrel::new());

                let nozzle_id = world.add_entity();
                world.add_component(
                    &nozzle_id,
                    components::parent::Parent::new(barrel_id.clone()),
                );
                world.add_component(
                    &nozzle_id,
                    components::damage_dealer::DamageDealer::new(0.1),
                );
                world.add_component(&nozzle_id, components::cannon::Cannon::new());
                world.add_component(
                    &nozzle_id,
                    components::pose::PreTransform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
                );
                // world.add_component(&nozzle_id, display::debug_box::DebugBox::from_size(0.2));
            }
        }

        let mut systems = engine::Systems::new();
        systems.add_system(Box::new(ClockSystem {}));
        systems.add_system(Box::new(
            systems::kinematics_differential_drive::KinematicsDifferentialDrive {},
        ));
        systems.add_system(Box::new(
            systems::acceleration_velocity::AccelerationVelocity {},
        ));
        systems.add_system(Box::new(systems::velocity_pose::VelocityPose {}));
        systems.add_system(Box::new(systems::revolute_pose::RevolutePose {}));
        systems.add_system(Box::new(systems::cannon_trigger::CannonTrigger {}));
        systems.add_system(Box::new(systems::projectile_floor::ProjectileFloor {}));

        Construct { world, systems }
    }

    pub fn update(&mut self) {
        self.systems.update(&mut self.world);
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn entity_pose(&self, entity: &EntityId) -> components::pose::Pose {
        components::pose::world_pose(&self.world, entity)
    }

    pub fn elapsed_as_f64(&self) -> f64 {
        let (_entity, clock) = self
            .world
            .component_iter_mut::<crate::components::clock::Clock>()
            .next()
            .expect("Should have one clock");
        clock.elapsed_as_f32().into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_things() {
        let mut construct = Construct::new();
        construct.update();
        construct.update();
        construct.update();
        let (_entity, clock) = construct
            .world()
            .component_iter_mut::<Clock>()
            .next()
            .expect("Should have one clock");
        assert_eq!(clock.elapsed_as_f32(), 0.03);
    }
}
