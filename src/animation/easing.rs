//! Easing Functions
//!
//! A comprehensive library of easing functions for smooth animation transitions.

use std::f32::consts::PI;

/// Easing function types for animation interpolation
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum EasingFunction {
    /// No easing, linear interpolation
    #[default]
    Linear,

    // Quadratic
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,

    // Cubic
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,

    // Quartic
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,

    // Quintic
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,

    // Sinusoidal
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,

    // Exponential
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,

    // Circular
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,

    // Back (overshoot)
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,

    // Elastic
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,

    // Bounce
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,

    /// Step function - instant transition at t=1
    Step,

    /// Smooth step (Hermite interpolation)
    SmoothStep,

    /// Smoother step (Ken Perlin's smootherstep)
    SmootherStep,
}

impl EasingFunction {
    /// Apply the easing function to a value t in range [0, 1]
    ///
    /// Returns the eased value, also in range [0, 1] for most functions.
    /// Some functions like Back and Elastic may overshoot.
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::Linear => t,

            // Quadratic
            Self::EaseInQuad => t * t,
            Self::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }

            // Cubic
            Self::EaseInCubic => t * t * t,
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }

            // Quartic
            Self::EaseInQuart => t * t * t * t,
            Self::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            Self::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }

            // Quintic
            Self::EaseInQuint => t * t * t * t * t,
            Self::EaseOutQuint => 1.0 - (1.0 - t).powi(5),
            Self::EaseInOutQuint => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
                }
            }

            // Sinusoidal
            Self::EaseInSine => 1.0 - (t * PI / 2.0).cos(),
            Self::EaseOutSine => (t * PI / 2.0).sin(),
            Self::EaseInOutSine => -(((t * PI).cos() - 1.0) / 2.0),

            // Exponential
            Self::EaseInExpo => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0_f32.powf(10.0 * t - 10.0)
                }
            }
            Self::EaseOutExpo => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f32.powf(-10.0 * t)
                }
            }
            Self::EaseInOutExpo => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    2.0_f32.powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
                }
            }

            // Circular
            Self::EaseInCirc => 1.0 - (1.0 - t * t).sqrt(),
            Self::EaseOutCirc => (1.0 - (t - 1.0).powi(2)).sqrt(),
            Self::EaseInOutCirc => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }

            // Back (overshoot)
            Self::EaseInBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                C3 * t * t * t - C1 * t * t
            }
            Self::EaseOutBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
            }
            Self::EaseInOutBack => {
                const C1: f32 = 1.70158;
                const C2: f32 = C1 * 1.525;
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (t * 2.0 - 2.0) + C2) + 2.0) / 2.0
                }
            }

            // Elastic
            Self::EaseInElastic => {
                const C4: f32 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -2.0_f32.powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * C4).sin()
                }
            }
            Self::EaseOutElastic => {
                const C4: f32 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4).sin() + 1.0
                }
            }
            Self::EaseInOutElastic => {
                const C5: f32 = (2.0 * PI) / 4.5;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0
                } else {
                    (2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0 + 1.0
                }
            }

            // Bounce
            Self::EaseInBounce => 1.0 - Self::EaseOutBounce.apply(1.0 - t),
            Self::EaseOutBounce => {
                const N1: f32 = 7.5625;
                const D1: f32 = 2.75;

                if t < 1.0 / D1 {
                    N1 * t * t
                } else if t < 2.0 / D1 {
                    let t = t - 1.5 / D1;
                    N1 * t * t + 0.75
                } else if t < 2.5 / D1 {
                    let t = t - 2.25 / D1;
                    N1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / D1;
                    N1 * t * t + 0.984375
                }
            }
            Self::EaseInOutBounce => {
                if t < 0.5 {
                    (1.0 - Self::EaseOutBounce.apply(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + Self::EaseOutBounce.apply(2.0 * t - 1.0)) / 2.0
                }
            }

            // Step functions
            Self::Step => {
                if t < 1.0 {
                    0.0
                } else {
                    1.0
                }
            }
            Self::SmoothStep => t * t * (3.0 - 2.0 * t),
            Self::SmootherStep => t * t * t * (t * (t * 6.0 - 15.0) + 10.0),
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::EaseInQuad => "Ease In Quad",
            Self::EaseOutQuad => "Ease Out Quad",
            Self::EaseInOutQuad => "Ease In Out Quad",
            Self::EaseInCubic => "Ease In Cubic",
            Self::EaseOutCubic => "Ease Out Cubic",
            Self::EaseInOutCubic => "Ease In Out Cubic",
            Self::EaseInQuart => "Ease In Quart",
            Self::EaseOutQuart => "Ease Out Quart",
            Self::EaseInOutQuart => "Ease In Out Quart",
            Self::EaseInQuint => "Ease In Quint",
            Self::EaseOutQuint => "Ease Out Quint",
            Self::EaseInOutQuint => "Ease In Out Quint",
            Self::EaseInSine => "Ease In Sine",
            Self::EaseOutSine => "Ease Out Sine",
            Self::EaseInOutSine => "Ease In Out Sine",
            Self::EaseInExpo => "Ease In Expo",
            Self::EaseOutExpo => "Ease Out Expo",
            Self::EaseInOutExpo => "Ease In Out Expo",
            Self::EaseInCirc => "Ease In Circ",
            Self::EaseOutCirc => "Ease Out Circ",
            Self::EaseInOutCirc => "Ease In Out Circ",
            Self::EaseInBack => "Ease In Back",
            Self::EaseOutBack => "Ease Out Back",
            Self::EaseInOutBack => "Ease In Out Back",
            Self::EaseInElastic => "Ease In Elastic",
            Self::EaseOutElastic => "Ease Out Elastic",
            Self::EaseInOutElastic => "Ease In Out Elastic",
            Self::EaseInBounce => "Ease In Bounce",
            Self::EaseOutBounce => "Ease Out Bounce",
            Self::EaseInOutBounce => "Ease In Out Bounce",
            Self::Step => "Step",
            Self::SmoothStep => "Smooth Step",
            Self::SmootherStep => "Smoother Step",
        }
    }

    /// Get all easing functions
    pub fn all() -> &'static [EasingFunction] {
        &[
            Self::Linear,
            Self::EaseInQuad,
            Self::EaseOutQuad,
            Self::EaseInOutQuad,
            Self::EaseInCubic,
            Self::EaseOutCubic,
            Self::EaseInOutCubic,
            Self::EaseInQuart,
            Self::EaseOutQuart,
            Self::EaseInOutQuart,
            Self::EaseInQuint,
            Self::EaseOutQuint,
            Self::EaseInOutQuint,
            Self::EaseInSine,
            Self::EaseOutSine,
            Self::EaseInOutSine,
            Self::EaseInExpo,
            Self::EaseOutExpo,
            Self::EaseInOutExpo,
            Self::EaseInCirc,
            Self::EaseOutCirc,
            Self::EaseInOutCirc,
            Self::EaseInBack,
            Self::EaseOutBack,
            Self::EaseInOutBack,
            Self::EaseInElastic,
            Self::EaseOutElastic,
            Self::EaseInOutElastic,
            Self::EaseInBounce,
            Self::EaseOutBounce,
            Self::EaseInOutBounce,
            Self::Step,
            Self::SmoothStep,
            Self::SmootherStep,
        ]
    }
}

/// Interpolate between two values using an easing function
pub fn ease<T: Lerp>(a: T, b: T, t: f32, easing: EasingFunction) -> T {
    let eased_t = easing.apply(t);
    a.lerp(b, eased_t)
}

/// Trait for types that can be linearly interpolated
pub trait Lerp {
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Lerp for f64 {
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        assert_eq!(EasingFunction::Linear.apply(0.0), 0.0);
        assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);
        assert_eq!(EasingFunction::Linear.apply(1.0), 1.0);
    }

    #[test]
    fn test_ease_in_out_quad() {
        let easing = EasingFunction::EaseInOutQuad;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!((easing.apply(0.5) - 0.5).abs() < 0.001);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_bounce() {
        let easing = EasingFunction::EaseOutBounce;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(1.0), 1.0);
        // Bounce should have intermediate value less than 1 at some point
        assert!(easing.apply(0.5) < 1.0);
    }

    #[test]
    fn test_clamping() {
        // Values outside [0,1] should be clamped
        assert_eq!(EasingFunction::Linear.apply(-0.5), 0.0);
        assert_eq!(EasingFunction::Linear.apply(1.5), 1.0);
    }
}
