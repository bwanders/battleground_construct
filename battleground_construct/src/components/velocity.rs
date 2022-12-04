use engine::prelude::*;

/// Velocity expressed in body frame (in all cases).
#[derive(Copy, Debug, Clone)]
pub struct Velocity {
    /// Translation component.
    pub v: cgmath::Vector3<f32>,
    /// Rotation component.
    pub w: cgmath::Vector3<f32>,
}

macro_rules! create_velocity_implementation {
    ($the_type:ty) => {
        impl $the_type {
            pub fn new() -> Self {
                Self {
                    v: cgmath::Vector3::new(0.0, 0.0, 0.0),
                    w: cgmath::Vector3::new(0.0, 0.0, 0.0),
                }
            }
            pub fn from_se2(x: f32, y: f32, yaw: f32) -> Self {
                Self {
                    v: cgmath::Vector3::new(x, y, 0.0),
                    w: cgmath::Vector3::new(0.0, 0.0, yaw),
                }
            }

            pub fn from_velocities(v: cgmath::Vector3<f32>, w: cgmath::Vector3<f32>) -> Self {
                Self { v, w }
            }

            pub fn integrate(&self, dt: f32) -> cgmath::Matrix4<f32> {
                (cgmath::Matrix4::<f32>::from_translation(cgmath::Vector3::new(
                    self.v[0] * dt,
                    self.v[1] * dt,
                    self.v[2] * dt,
                )) * cgmath::Matrix4::<f32>::from_angle_x(cgmath::Rad(self.w[0]) * dt)
                    * cgmath::Matrix4::<f32>::from_angle_y(cgmath::Rad(self.w[1]) * dt)
                    * cgmath::Matrix4::<f32>::from_angle_z(cgmath::Rad(self.w[2]) * dt))
            }

            pub fn to_twist(&self) -> crate::util::cgmath::Twist<f32> {
                crate::util::cgmath::Twist::<f32>::new(self.v, self.w)
            }

            pub fn integrate_pose(&self, pose: &super::pose::Pose, dt: f32) -> super::pose::Pose {
                let res = (pose.h * self.integrate(dt));
                // return res.into();

                // Re-orthogonalize the rotation part of this matrix.
                //https://stackoverflow.com/a/23082112
                // x_new = 0.5*(3-dot(x_ort,x_ort))*x_ort
                // y_new = 0.5*(3-dot(y_ort,y_ort))*y_ort
                // z_new = 0.5*(3-dot(z_ort,z_ort))*z_ort
                use cgmath::InnerSpace;

                let x_ort = res.x.truncate();
                let y_ort = res.y.truncate();
                let z_ort = res.z.truncate();
                let c0 = (0.5 * (3.0 - x_ort.dot(x_ort)) * x_ort);
                let c1 = (0.5 * (3.0 - y_ort.dot(y_ort)) * y_ort);
                let c2 = (0.5 * (3.0 - z_ort.dot(z_ort)) * z_ort);

                // Finally, re-normalize the matrix as well.
                use cgmath::SquareMatrix;
                let m3 = cgmath::Matrix3::<f32>::from_cols(c0, c1, c2);
                let det = m3.determinant();
                let c0 = (1.0 / det) * c0;
                let c1 = (1.0 / det) * c1;
                let c2 = (1.0 / det) * c2;

                let c3 = res.w;

                // Finally, reconstruct the transform matrix.
                cgmath::Matrix4::<f32>::from_cols(
                    c0.extend(0.0),
                    c1.extend(0.0),
                    c2.extend(0.0),
                    c3,
                )
                .into()
            }
            pub fn integrate_global_pose(
                &self,
                pose: &super::pose::Pose,
                dt: f32,
            ) -> super::pose::Pose {
                (self.integrate(dt) * pose.h).into()
            }
        }
        impl Component for $the_type {}

        /*
        impl Into<cgmath::Matrix4<f32>> for $the_type {
            fn into(self) -> cgmath::Matrix4<f32> {
                use crate::util::cgmath::ToCross;
                let s = self.w.to_cross();
                cgmath::Matrix4::<f32>::from_cols(
                    s.x.extend(0.0),
                    s.y.extend(0.0),
                    s.z.extend(0.0),
                    self.v.extend(0.0),
                )
            }
        }*/
        impl Into<$the_type> for crate::util::cgmath::Twist<f32> {
            fn into(self) -> $the_type {
                <$the_type>::from_velocities(self.v, self.w)
            }
        }
    };
}
create_velocity_implementation!(Velocity);

pub fn world_velocity(world: &World, entity: EntityId) -> Velocity {
    // use crate::components::pose::world_pose;
    use crate::components::pose::Pose;
    use crate::components::pose::PreTransform;
    use crate::util::cgmath::prelude::*;

    let mut current_id = entity.clone();
    let mut current_velocity = Velocity::new().to_twist();
    let mut current_pose = Pose::new();
    /*
        Changing frame of a twist.
        twist = Adjoint(Frame) * frame_twist.
    */
    loop {
        // println!("Current id: {current_id:?}");

        let pose_t = if let Some(pose) = world.component::<Pose>(current_id) {
            // println!("Found pose for  {current_id:?}");
            *pose.transform()
        } else {
            *Pose::new().transform()
        };
        let vel_t = if let Some(vel) = world.component::<Velocity>(current_id) {
            // println!("Found vel for  {current_id:?}");
            vel.to_twist()
        } else {
            Velocity::new().to_twist()
        };

        current_pose = (pose_t * *current_pose).into();
        let combined_vel = current_velocity + vel_t;

        // println!("  current_velocity: {current_velocity:?}");
        // println!("  vel_t: {vel_t:?}");
        // println!("  combined_vel: {combined_vel:?}");
        // println!("  pose_t: {pose_t:?}");
        // println!("  pose_t adjoint: {:?}", pose_t.to_adjoint());
        current_velocity = pose_t.to_adjoint() * (combined_vel);
        // println!("  new current_velocity: {current_velocity:?}");

        let pre_pose_t = if let Some(pre_pose) = world.component::<PreTransform>(current_id) {
            // println!("Found PreTransform for  {current_id:?}");
            *pre_pose.transform()
        } else {
            *Pose::new().transform()
        };
        current_pose = (pre_pose_t * current_pose.transform()).into();
        // println!("  pre_pose_t: {pre_pose_t:?}");
        // println!("  pre_pose_t adjoint: {:?}", pre_pose_t.to_adjoint());
        current_velocity = pre_pose_t.to_adjoint() * current_velocity;
        // println!("  new current_velocity: {current_velocity:?}");

        if let Some(parent) = world.component::<super::parent::Parent>(current_id) {
            current_id = parent.parent().clone();
        } else {
            break;
        }
        // println!("  parent: {current_id:?}");
    }
    // That give us the velocity in the origin, do we need to mutiply that with the final transform
    // to the pose again?
    (current_pose.to_adjoint() * current_velocity).into()
    // current_velocity.into()
}

#[cfg(test)]
mod test {
    use super::super::pose::Pose;
    use super::*;
    #[test]
    fn test_velocity_integration() {
        let start = Pose::new();
        let dt = 0.01f32;
        let mut v = Velocity::new();
        v.v[0] = 1.0;

        let mut p = start;
        for _i in 0..100 {
            p = (p.h * v.integrate(dt)).into();
        }

        assert!((p.h.w[0] - 100.0 * dt * 1.0).abs() <= 0.00001);
    }
}
