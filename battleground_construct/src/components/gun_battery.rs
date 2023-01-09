use crate::display::primitives::Mat4;
use engine::prelude::*;

// This must be an Rc, as we need to be able to copy it to allow a mutable world, we cannot borrow
// it out of the cannon.
pub type GunBatteryFireEffect = std::rc::Rc<dyn for<'a> Fn(&'a mut World, EntityId, Mat4)>;

/*
    Usually called an artillery battery, but 'gun' has a more generic term to it, and we need it to
    not be called 'Battery', in case we ever introduce electric batteries like Colobot.

    A gun battery has multiple guns that each have a transform relative to the entity of the gun
    battery's pose.

    Each gun has an inter-gun delay period (may be zero).

    Multiple guns may be triggered at the same time, they shall fire in sequence.

    Each gun has its own reload period that starts counting immediately after fired. At the end of
    the sequence, there may be an additional delay.
*/

pub struct GunBatteryConfig {
    /// Function called when a gun in the battery is fired.
    pub fire_effect: GunBatteryFireEffect,
    /// Duration between firing of guns in the battery.
    pub inter_gun_duration: f32,
    /// Individual gun reload duration.
    pub gun_reload: f32,
    /// Battery reload duration, after the last gun has fired. This allows enforcing that nothing
    /// can be fired after the battery finishes, which cannot be guaranteed by gun_reload. Both
    /// gun_reload and battery_reload must have elapsed before firing can commence.
    pub battery_reload: f32,
    /// Pose for each individual gun.
    pub poses: Vec<Mat4>,
}

#[derive(Debug, Copy, Clone)]
struct GunStatus {
    /// Time of last firing of this gun, to track this gun's reload.
    last_fire_time: f32,
    /// Boolean to track whether this gun is ready.
    is_ready: bool,
    // is_triggered: bool,
}

pub struct GunBattery {
    config: GunBatteryConfig,
    current_index: usize,
    last_gun_fire_time: f32,
    last_in_battery_fire_time: f32,
    is_triggered: bool,
    is_ready: bool,
    status: Vec<GunStatus>,
}

impl GunBattery {
    pub fn new(config: GunBatteryConfig) -> Self {
        let status = vec![
            GunStatus {
                last_fire_time: -config.gun_reload,
                is_ready: true,
                // is_triggered: false,
            };
            config.poses.len()
        ];
        Self {
            current_index: 0,
            last_gun_fire_time: -config.gun_reload,
            last_in_battery_fire_time: -config.battery_reload,
            config,
            is_triggered: false,
            is_ready: true,
            status,
        }
    }

    pub fn is_triggered(&self) -> bool {
        self.is_triggered
    }

    pub fn set_trigger(&mut self, value: bool) {
        self.is_triggered = value;
    }

    pub fn is_ready(&self) -> bool {
        self.is_ready
    }

    pub fn update(&mut self, current_time: f32) {
        let gun_interval_done =
            (current_time - self.last_gun_fire_time) >= self.config.inter_gun_duration;
        let battery_reload_done =
            (current_time - self.last_in_battery_fire_time) >= self.config.battery_reload;

        let mut at_least_one_gun_loaded = false;
        for gun_status in self.status.iter_mut() {
            gun_status.is_ready =
                (current_time - gun_status.last_fire_time) >= self.config.gun_reload;
            if gun_status.is_ready {
                at_least_one_gun_loaded = true;
            }
        }
        self.is_ready = battery_reload_done && at_least_one_gun_loaded && gun_interval_done;
    }

    pub fn fired(&mut self, current_time: f32) -> Mat4 {
        // Modify this gun.
        self.status[self.current_index].is_ready = false;
        self.status[self.current_index].last_fire_time = current_time;
        self.last_gun_fire_time = current_time;

        let fire_pose = self.config.poses[self.current_index];

        // Increment the gun index.
        if self.current_index + 1 >= self.status.len() {
            // Wrap around, set the last gun fire time.
            self.last_in_battery_fire_time = current_time;

            self.current_index = 0;
        } else {
            self.current_index += 1;
        }

        // Always update, the is_ready flag needs to update immediately.
        self.update(current_time);
        fire_pose
    }

    pub fn gun_index(&self) -> usize {
        self.current_index
    }

    pub fn effect(&self) -> GunBatteryFireEffect {
        self.config.fire_effect.clone()
    }
}
impl Component for GunBattery {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::display::primitives::Vec3;
    #[test]
    fn test_gun_battery() {
        {
            let inter_gun_duration = 1.0;
            let gun_reload = 2.0;
            let battery_reload = 3.0;
            let config = GunBatteryConfig {
                fire_effect: std::rc::Rc::new(|_, _, _| {}),
                inter_gun_duration,
                gun_reload,
                battery_reload,
                poses: vec![
                    Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                    Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0)),
                    Mat4::from_translation(Vec3::new(0.0, 3.0, 0.0)),
                ],
            };

            let mut battery = GunBattery::new(config);
            let mut t = 0.0;

            assert!(battery.is_ready());
            battery.update(t);
            assert!(battery.is_ready());
            battery.fired(t);
            assert_eq!(battery.is_ready(), false); // not ready right now, in between the inter gun
            t += inter_gun_duration;
            battery.update(t);
            assert!(battery.is_ready());
            battery.fired(t);
            t += inter_gun_duration;
            t += inter_gun_duration;
            battery.update(t);
            battery.fired(t);
            battery.update(t);
            assert_eq!(battery.is_ready(), false);

            // Next shot is the last in the gun battery.
            battery.fired(t);
            assert_eq!(battery.is_ready(), false);
            t += inter_gun_duration;
            battery.update(t);
            assert_eq!(battery.is_ready(), false);
            t += inter_gun_duration;
            battery.update(t);
            assert_eq!(battery.is_ready(), false);
            t += inter_gun_duration;
            battery.update(t);
            assert_eq!(battery.is_ready(), true); // at start again.
            assert_eq!(battery.gun_index(), 0); // at start again.
        }

        {
            let inter_gun_duration = 0.0;
            let gun_reload = 2.0;
            let battery_reload = 3.0;
            let config = GunBatteryConfig {
                fire_effect: std::rc::Rc::new(|_, _, _| {}),
                inter_gun_duration,
                gun_reload,
                battery_reload,
                poses: vec![
                    Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                    Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0)),
                ],
            };

            let mut battery = GunBattery::new(config);
            let mut t = 0.0;

            assert!(battery.is_ready());
            battery.update(t);
            assert!(battery.is_ready());
            battery.fired(t);
            assert!(battery.is_ready()); // no inter gun delay.
            battery.fired(t);
            assert!(battery.is_ready());

            // Next shot is the last in the gun battery.
            battery.fired(t);
            assert_eq!(battery.is_ready(), false);
            t += battery_reload;
            battery.update(t);
            assert_eq!(battery.is_ready(), true); // at start again.
            assert_eq!(battery.gun_index(), 0); // at start again.
        }
    }
}

/*
use crate::components::unit_interface::{Register, RegisterMap, UnitModule};
use battleground_unit_control::modules::cannon::*;
pub struct CannonModule {
    entity: EntityId,
}

impl CannonModule {
    pub fn new(entity: EntityId) -> Self {
        CannonModule { entity }
    }
}

impl UnitModule for CannonModule {
    fn get_registers(&self, world: &World, registers: &mut RegisterMap) {
        registers.clear();
        if let Some(cannon) = world.component::<Cannon>(self.entity) {
            registers.insert(
                REG_CANNON_TRIGGER,
                Register::new_i32("trigger", cannon.is_triggered() as i32),
            );
            registers.insert(
                REG_CANNON_IS_TRIGGERED,
                Register::new_i32("is_triggered", cannon.is_triggered() as i32),
            );
            registers.insert(
                REG_CANNON_READY,
                Register::new_i32("ready", cannon.is_ready() as i32),
            );
            registers.insert(
                REG_CANNON_RELOAD_TIME,
                Register::new_f32("reload_time", cannon.config.reload_time),
            );
        }
    }

    fn set_component(&self, world: &mut World, registers: &RegisterMap) {
        if let Some(mut cannon) = world.component_mut::<Cannon>(self.entity) {
            let trigger = registers
                .get(&REG_CANNON_TRIGGER)
                .expect("register doesnt exist")
                .value_i32()
                .expect("wrong value type");
            if trigger != 0 {
                cannon.trigger();
            }
        }
    }
}
*/
