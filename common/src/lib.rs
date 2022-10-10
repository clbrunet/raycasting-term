use std::f64;

pub mod sprite;

pub static MAP: [[u8; 8]; 8] = [
    [1, 1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 2, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1, 1],
];

/// Get angle in [0; tau[
pub fn get_normalized_radians_angle(mut angle: f64) -> f64 {
    if angle.is_sign_negative() {
        angle = angle % f64::consts::TAU + f64::consts::TAU;
    }
    angle % f64::consts::TAU
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_normalized_radians_angle() {
        let abs_difference =
            (get_normalized_radians_angle(0_f64.to_radians()) - 0_f64.to_radians()).abs();
        assert!(abs_difference < 1e-10);

        let abs_difference =
            (get_normalized_radians_angle(1_f64.to_radians()) - 1_f64.to_radians()).abs();
        assert!(abs_difference < 1e-10);

        let abs_difference =
            (get_normalized_radians_angle(359_f64.to_radians()) - 359_f64.to_radians()).abs();
        assert!(abs_difference < 1e-10);

        let abs_difference =
            (get_normalized_radians_angle(360_f64.to_radians()) - 0_f64.to_radians()).abs();
        assert!(abs_difference < 1e-10);

        let abs_difference =
            (get_normalized_radians_angle(361_f64.to_radians()) - 1_f64.to_radians()).abs();
        assert!(abs_difference < 1e-10);

        let abs_difference =
            (get_normalized_radians_angle(-1_f64.to_radians()) - 359_f64.to_radians()).abs();
        assert!(abs_difference < 1e-10);

        let abs_difference =
            (get_normalized_radians_angle(-359_f64.to_radians()) - 1_f64.to_radians()).abs();
        assert!(abs_difference < 1e-10);

        let abs_difference =
            (get_normalized_radians_angle(-360_f64.to_radians()) - 0_f64.to_radians()).abs();
        assert!(abs_difference < 1e-10);

        let abs_difference =
            (get_normalized_radians_angle(-361_f64.to_radians()) - 359_f64.to_radians()).abs();
        assert!(abs_difference < 1e-10);
    }
}
