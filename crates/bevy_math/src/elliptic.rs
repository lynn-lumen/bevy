//! This module contains methods for computing elliptic integrals
#![deny(clippy::excessive_precision)]
use std::f32::consts::PI;

/// Computes the incomplete elliptic integral of the first kind for `phi` and `m`: F(phi | m).
/// The result will be accurate within ten machine epsilons
///
/// `phi` must be in the range `(0, PI / 2)` and
/// `m` must be in the range `(0, 1)`
pub fn elliptic_f(phi: f32, m: f32) -> f32 {
    const PHI_S: f32 = 1.249;
    const GAMMA_S: f32 = 0.9;

    let mc = 1.0 - m;

    if phi < PHI_S {
        let elf = asn(phi.sin(), m);
        return elf;
    } else {
        let c = phi.cos();
        let x = c * c;
        let d2 = mc + m * x;
        if x < GAMMA_S * d2 {
            let elf = elliptic_k(mc) - asn(c / d2.sqrt(), m);
            return elf;
        } else {
            let v = mc * (1. - x);
            if v < x * d2 {
                let elf = acn(c, mc);
                return elf;
            } else {
                let elf = elliptic_k(mc) - acn(f32::sqrt(v / d2), mc);
                return elf;
            }
        }
    };
}

/// Computes the inverse sine amplitude
///
/// 's' must be in the range `(0, 1)` and
/// 'm' must be in the range `(0, 1)`
///
/// The usage of this function for `s > 0.95` is not recommended. Please look into using [`acn`] instead.
pub fn asn(s: f32, m: f32) -> f32 {
    let y_a = 0.1888 - 0.0378 * m;

    let mut y = s * s;
    if y < y_a {
        let asn = s * serf(y, m);
        return asn;
    };

    let mut p = 1.0;
    for _ in 1..=10 {
        y = y / ((1.0 + f32::sqrt(1.0 - y)) * (1.0 + f32::sqrt(1.0 - m * y)));
        p *= 2.0;
        if y < y_a {
            let asn = p * y.sqrt() * serf(y, m);
            return asn;
        }
    }

    todo!()
}

/// Computes the inverse cosine amplitude
///
/// 'c' must be in the range `(0, 1)` and
/// 'mc' must be in the range `(0, 1)`
///
/// The usage of this function for `c > 0.95` is not recommended. Please look into using [`asn`] instead.
pub fn acn(c: f32, mc: f32) -> f32 {
    let m = 1.0 - mc;
    let mut p = 1.0;
    let mut x = c * c;
    for _ in 1..=10 {
        if x > 0.5 {
            let acn = p * asn(f32::sqrt(1.0 - x), m);
            return acn;
        }
        let d = f32::sqrt(mc + m * x);
        x = (x.sqrt() + d) / (1.0 + d);
        p = p * 2.0;
    }

    todo!()
}

/// Computes the truncated series expansion of the inverse sine amplitude function
fn serf(y: f32, m: f32) -> f32 {
    // These values are the numerators and the denominator of the polynomial coefficients, u_lj
    // as they appear in the Maclaurin series expansion of sn^(−1)(s|m) in terms of y ≡ s^2.
    const U10: f32 = 1. / 6.;
    const U20: f32 = 3. / 40.;
    const U21: f32 = 2. / 40.;
    const U30: f32 = 5. / 112.;
    const U31: f32 = 3. / 112.;
    const U40: f32 = 35. / 1152.;
    const U41: f32 = 20. / 1152.;
    const U42: f32 = 18. / 1152.;
    const U50: f32 = 65. / 2816.;
    const U51: f32 = 35. / 2816.;
    const U52: f32 = 30. / 2816.;
    const U60: f32 = 231. / 13312.;
    const U61: f32 = 126. / 13312.;
    const U62: f32 = 105. / 13312.;
    const U63: f32 = 100. / 13312.;

    let u1 = U10 + m * U10;
    let u2 = U20 + m * (U21 + m * U20);
    let u3 = U30 + m * (U31 + m * (U31 + m * U30));
    let u4 = U40 + m * (U41 + m * (U42 + m * (U41 + m * U40)));
    let u5 = U50 + m * (U51 + m * (U52 + m * (U52 + m * (U51 + m * U50))));
    let u6 = U60 + m * (U61 + m * (U62 + m * (U63 + m * (U62 + m * (U61 + m * U60)))));

    1.0 + y * (u1 + y * (u2 + y * (u3 + y * (u4 + y * (u5 + y * u6)))))
}

/// Computes the complete elliptic integral of the first kind for a given `m`:
/// The result will have a relative error of less than 6 machine epsilons
///
/// `m` should be in the range `(0, 1)`
pub fn elliptic_k(m: f32) -> f32 {
    if m < 0.9 {
        elliptic_k_small(m)
    } else {
        elliptic_k_large(m)
    }
}

/// Computes the complete elliptic integral of the first kind for a given `m`:
///
/// `m` should be in the range `(0, 0.9)`
fn elliptic_k_small(m: f32) -> f32 {
    let index = if m < 0.8 {
        (m / 0.1) as usize
    } else {
        ((m - 0.8) / 0.05) as usize + 8
    };

    // These are the coefficients of the Taylor expansion of K(m) around a given midpoint: (midpoint, coefficients)
    // The amount of coefficients is chosen so that K(m) is within one machine epsilon `f32::EPSILON` of the correct value
    const MIDPOINT_COEFFICIENTS: [(f32, &[f32]); 10] = [
        (
            0.05,
            &[1.59100345, 0.41600074, 0.24579151, 0.17948148, 0.14455606],
        ),
        (
            0.15,
            &[1.63525673, 0.47119063, 0.30972841, 0.25220831, 0.22672562],
        ),
        (
            0.25,
            &[1.68575035, 0.54173185, 0.40152444, 0.36964247, 0.37606072],
        ),
        (
            0.35,
            &[
                1.74435060, 0.63486428, 0.53984256, 0.57189271, 0.67029514, 0.83258659,
            ],
        ),
        (
            0.45,
            &[
                1.81388394, 0.76316325, 0.76192861, 0.95107465, 1.31518068, 1.92856069,
            ],
        ),
        (
            0.55,
            &[
                1.89892491, 0.95052179, 1.15107759, 1.75023911, 2.95267681, 5.28580040,
            ],
        ),
        (
            0.65,
            &[
                2.00759840, 1.24845723, 1.92623466, 3.75128964, 8.11994455, 18.6657213, 44.6039248,
            ],
        ),
        (
            0.75,
            &[
                2.15651565, 1.79180564, 3.82675129, 10.3867247, 31.4033141, 100.923704, 337.326828,
                1158.70793,
            ],
        ),
        (
            0.825,
            &[
                2.31812262, 2.61692015, 7.89793508, 30.5023972, 131.486937, 602.984764, 2877.02462,
            ],
        ),
        (
            0.875,
            &[
                2.47359617, 3.72762424, 15.6073930, 84.1285084, 506.981820, 3252.27706, 21713.2424,
                149037.045,
            ],
        ),
    ];

    let (midpoint, coefficients) = MIDPOINT_COEFFICIENTS[index];

    let mut sum = 0.;
    let mut d_m = 1.;
    for k_j in coefficients {
        sum += k_j * d_m;
        d_m *= m - midpoint;
    }

    sum
}

/// Computes the complete elliptic integral of the first kind for a given `m`:
///
/// `m` should be in the range `(0.9, 1)`
fn elliptic_k_large(m: f32) -> f32 {
    let m_c = 1. - m;
    let k_c = elliptic_k_small(m_c);

    // These values are the coefficients of the Maclaurin expansion of q_c with respect to m_c
    const COEFFICIENTS: [f32; 6] = [
        1. / 16.,
        1. / 32.,
        21. / 1024.,
        31. / 2048.,
        6257. / 524288.,
        10293. / 1048576.,
    ];

    let mut q_c = 0.;
    for (j, coef) in COEFFICIENTS.iter().enumerate() {
        q_c += m_c.powi(j as i32 + 1) * coef;
    }

    let k = -1. * q_c.ln() * k_c / PI;
    return k;
}

mod tests {
    use crate::elliptic::{elliptic_f, elliptic_k};
    use approx::assert_relative_eq;
    use std::{
        f32::consts::{FRAC_PI_2, FRAC_PI_4},
        time::Instant,
    };

    const FIVE_EPSILON: f32 = 6. * f32::EPSILON;

    #[test]
    fn complete_integral_first_kind_low() {
        assert_relative_eq!(elliptic_k(0.), FRAC_PI_2);
        assert_relative_eq!(elliptic_k(0.00001), 1.570800253);
        assert_relative_eq!(elliptic_k(0.09), 1.60804861);
        assert_relative_eq!(elliptic_k(0.185), 1.65213899);
        assert_relative_eq!(elliptic_k(0.25), 1.68575035);
        assert_relative_eq!(elliptic_k(0.31), 1.71978481);
        assert_relative_eq!(elliptic_k(0.493), 1.84818918);
        assert_relative_eq!(elliptic_k(0.5), 1.854074677);
        assert_relative_eq!(elliptic_k(0.597), 1.94633962);
        assert_relative_eq!(elliptic_k(0.618), 1.96950524);
        assert_relative_eq!(elliptic_k(0.75), 2.15651565);
        assert_relative_eq!(elliptic_k(0.801), 2.25948339);
        assert_relative_eq!(elliptic_k(0.853), 2.39835076);
    }

    #[test]
    fn complete_integral_first_kind_high() {
        // Use the larger epsilon for the high tests
        assert_relative_eq!(elliptic_k(0.903), 2.59243319, max_relative = FIVE_EPSILON);
        assert_relative_eq!(elliptic_k(0.916), 2.66041638, max_relative = FIVE_EPSILON);
        assert_relative_eq!(elliptic_k(0.924), 2.70791714, max_relative = FIVE_EPSILON);
        assert_relative_eq!(elliptic_k(0.933), 2.76797262, max_relative = FIVE_EPSILON);
        assert_relative_eq!(elliptic_k(0.948), 2.88945822, max_relative = FIVE_EPSILON);
        assert_relative_eq!(elliptic_k(0.952), 2.92800816, max_relative = FIVE_EPSILON);
        assert_relative_eq!(elliptic_k(0.961), 3.02837751, max_relative = FIVE_EPSILON);
        assert_relative_eq!(elliptic_k(0.976), 3.26482357, max_relative = FIVE_EPSILON);
        assert_relative_eq!(elliptic_k(0.985), 3.49554409, max_relative = FIVE_EPSILON);
        assert_relative_eq!(elliptic_k(0.995), 4.03925748, max_relative = FIVE_EPSILON);
        assert_relative_eq!(elliptic_k(1.), f32::INFINITY, max_relative = FIVE_EPSILON);
    }

    #[test]
    fn incomplete_integral_first_kind() {
        assert_relative_eq!(
            elliptic_f(FRAC_PI_4, 0.),
            0.7853982,
            max_relative = FIVE_EPSILON
        );
        assert_relative_eq!(
            elliptic_f(FRAC_PI_4, 0.25),
            0.8043661,
            max_relative = FIVE_EPSILON
        );
        assert_relative_eq!(
            elliptic_f(FRAC_PI_4, 0.5),
            0.8260179,
            max_relative = FIVE_EPSILON
        );
        assert_relative_eq!(
            elliptic_f(FRAC_PI_4, 0.75),
            0.8512237,
            max_relative = FIVE_EPSILON
        );
        assert_relative_eq!(
            elliptic_f(FRAC_PI_4, 1.0),
            0.8813736,
            max_relative = FIVE_EPSILON
        );
    }
}
