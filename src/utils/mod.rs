use bevy::prelude::*;

pub fn look_to(direction: Vec3) -> Quat {
    let back = -direction.try_normalize().unwrap_or(Vec3::NEG_Z);
    let up = Vec3::Y;
    let right = up
        .cross(back)
        .try_normalize()
        .unwrap_or_else(|| up.any_orthonormal_vector());
    let up = back.cross(right);

    Quat::from_mat3(&Mat3::from_cols(right, up, back))
}
