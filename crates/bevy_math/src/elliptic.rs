//! This module contains methods for computing elliptic integrals
#![deny(clippy::excessive_precision)]
use std::f32::{consts::PI, INFINITY, NAN};

/// Computes the incomplete elliptic integral of the second kind for `phi` and `m`: E(phi | m).
/// The result will be accurate within ten machine epsilons
///
/// `phi` must be in the range `(0, PI / 2)` and
/// `m` must be in the range `(0, 1)`
pub fn el2(x: f32, m: f32) -> f32 {
    todo!();
}

/// Computes the incomplete elliptic integral of the second kind for `m`: E(m).
/// The result will be accurate within ten machine epsilons
///
/// `m` must be in the range `(0, 1)`
pub fn cel2(m: f32) -> f32 {
    if m == 0. {
        return PI / 2.;
    }
    if m == 1. {
        return 1.;
    }

    if m < -2e6 {
        let b = (-16. * m).ln();
        let a = (-m).sqrt();
        return a + 0.25 * (b + 1.) / a - 1. / 32. * (b - 1.5) / a.powi(3);
    };

    if m > 0.99999 {
        let m1 = 1. - m;
        let a = (16. / m1).ln();
        return 1. + m1 * 0.25 * (a - 1.);
    };

    2. * rg(0., 1. - m, 1.)
}

/// Computes the Carlson degenerate integral of [`el1`]
fn rc(x: f32, y: f32) -> f32 {
    if y == 0. {
        return f32::INFINITY;
    }
    if x == 0. && y > 0. {
        return PI / (2. * y.sqrt());
    }
    if x == y {
        return 1. / x.sqrt();
    }

    let (w, mut x, mut y) = if y > 0. {
        (1., x, y)
    } else {
        ((x / (x - y)).sqrt(), x - y, -y)
    };

    const ERRTOL: f32 = 3e-4;

    let (mut m, mut s);
    loop {
        let p = 2. * (x * y).sqrt() + y;
        x = 0.25 * (x + p);
        y = 0.25 * (y + p);
        s = x + y + y;
        m = s / 3.;
        s = (y - x) / s;
        if s.is_nan() {
            return f32::NAN;
        }
        if s.abs() < ERRTOL {
            break;
        }
    }

    const C1: f32 = 0.3;
    const C2: f32 = 1. / 7.;
    const C3: f32 = 0.375;
    const C4: f32 = 9. / 22.;
    let s = (C1 + s * (C2 + s * (C3 + s * C4))) * s * s;

    w * ((1. + s) / m.sqrt())
}

/// Computes the Carlson degenerate cases of [`cel2`]
fn rd(mut x: f32, mut y: f32, mut z: f32) -> f32 {
    if x + y == 0. {
        return f32::NAN;
    }
    if x == y && x == 0. {
        return f32::INFINITY;
    }

    if y == z {
        if x == 0. {
            return 3. * PI / (4. * y * y.sqrt());
        }
        if x == y {
            return 1. / (x * x.sqrt());
        }
    }

    const ERRTOL: f32 = 0.002;

    let mut sm = 0.;
    let mut fc = 1.;
    let (mut dx, mut dy, mut dz, mut m);
    loop {
        let rx = x.sqrt();
        let ry = y.sqrt();
        let rz = z.sqrt();
        let lm = rx * (ry + rz) + ry * rz;
        sm = sm + fc / (rz * (z + lm));
        fc = 0.25 * fc;
        x = 0.25 * (lm + x);
        y = 0.25 * (lm + y);
        z = 0.25 * (lm + z);
        m = (x + y + 3. * z) / 5.;
        if m.is_infinite() {
            return f32::NAN;
        }
        dx = (m - x) / m;
        dy = (m - y) / m;
        dz = (m - z) / m;
        if dx.abs() < ERRTOL && dy.abs() < ERRTOL && dz.abs() < ERRTOL {
            break;
        }
    }
    let ea = dx * dy;
    let eb = dz * dz;
    let ec = ea - eb;
    let ed = ea - 6. * eb;
    let ee = ed + ec + ec;

    const C1: f32 = -3. / 14.;
    const C2: f32 = 1. / 6.;
    const C3: f32 = 9. / 22.;
    const C4: f32 = 3. / 26.;
    const C5: f32 = 9. / 88.;
    const C6: f32 = 9. / 52.;
    let s = ed * (C1 + C5 * ed - C6 * dz * ee);
    let s = s + dz * (C2 * ee + dz * (dz * C4 * ea - C3 * ec));

    3. * sm + fc * (1. + s) / (m * m.sqrt())
}

/// Computes the Carlson elliptic integral of the first kind
fn rf(mut x: f32, mut y: f32, mut z: f32) -> f32 {
    if z == 0. && x > 0. {
        std::mem::swap(&mut x, &mut z);
    }
    if y == 0. && x > 0. {
        std::mem::swap(&mut x, &mut y);
    }

    if x == 0. && y == 0. {
        return INFINITY;
    }
    if x == y && y == z {
        return 1. / x.sqrt();
    }

    if y == z {
        return if x == 0. {
            PI / (2. * y.sqrt())
        } else {
            rc(x, y)
        };
    }

    let (mut dx, mut dy, mut dz, mut av);
    const ERRTOL: f32 = 0.001;
    loop {
        let rx = x.sqrt();
        let ry = y.sqrt();
        let rz = z.sqrt();
        let lm = rx * (ry + rz) + ry * rz;
        x = 0.25 * (lm + x);
        y = 0.25 * (lm + y);
        z = 0.25 * (lm + z);
        av = (x + y + z) / 3.;
        if av.is_infinite() {
            return NAN;
        }
        dx = (av - x) / av;
        dy = (av - y) / av;
        dz = (av - z) / av;
        if dx.abs() < ERRTOL && dy.abs() < ERRTOL && dz.abs() < ERRTOL {
            break;
        }
    }
    let e2 = dx * dy - dz * dz;
    let e3 = dx * dy * dz;

    const C1: f32 = 1. / 24.;
    const C2: f32 = 1. / 10.;
    const C3: f32 = 3. / 44.;
    const C4: f32 = 1. / 14.;
    let s = e2 * (C1 * e2 - C2 - C3 * e3) + C4 * e3;

    (1. + s) / av.sqrt()
}

/// Computes the Carlson's completely symmetric elliptic integral of the 2nd kind
fn rg(x: f32, y: f32, z: f32) -> f32 {
    let [y, z, x] = {
        let mut xyz = [x, y, z];
        xyz.sort_by(f32::total_cmp);
        xyz
    };
    if x == y {
        return x.sqrt();
    }
    if z == 0. {
        return 0.5 * x.sqrt();
    }
    if y == 0. && x == z {
        return PI / 4. * x.sqrt();
    }

    let s = ((x / z) * y).sqrt();
    let f = z * rf(x, y, z);
    let d = if z == x || z == y {
        0.
    } else {
        let d = rd(x, y, z) / 3.;
        let d = (x - z) * d;
        (z - y) * d
    };
    0.5 * (f + d + s)
}
/// Computes the incomplete elliptic integral of the first kind for `phi` and `m`: F(phi | m).
/// The result will be accurate within ten machine epsilons
///
/// `phi` must be in the range `(0, PI / 2)` and
/// `m` must be in the range `(0, 1)`
pub fn el1(phi: f32, m: f32) -> f32 {
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
            let elf = cel1(mc) - asn(c / d2.sqrt(), m);
            return elf;
        } else {
            let v = mc * (1. - x);
            if v < x * d2 {
                let elf = acn(c, mc);
                return elf;
            } else {
                let elf = cel1(mc) - acn(f32::sqrt(v / d2), mc);
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

/// Computes the complete elliptic integral of the first kind for a given `m`: K(m)
/// The result will have a relative error of less than 5 machine epsilons
///
/// `m` should be in the range `(0, 1)`
pub fn cel1(m: f32) -> f32 {
    if m < 0.9 {
        cel1_small(m)
    } else {
        cel1_large(m)
    }
}

/// Computes the complete elliptic integral of the first kind for a given `m`:
///
/// `m` should be in the range `(0, 0.9)`
fn cel1_small(m: f32) -> f32 {
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
fn cel1_large(m: f32) -> f32 {
    let m_c = 1. - m;
    let k_c = cel1_small(m_c);

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
    use crate::elliptic::{cel1, cel2, el1};
    use approx::assert_relative_eq;
    use std::{
        f32::consts::{FRAC_PI_2, FRAC_PI_4},
        time::Instant,
    };

    const FIVE_EPSILON: f32 = 6. * f32::EPSILON;

    #[test]
    fn complete_integral_first_kind_low() {
        assert_relative_eq!(cel1(0.), FRAC_PI_2);
        assert_relative_eq!(cel1(0.00001), 1.570800253);
        assert_relative_eq!(cel1(0.09), 1.60804861);
        assert_relative_eq!(cel1(0.185), 1.65213899);
        assert_relative_eq!(cel1(0.25), 1.68575035);
        assert_relative_eq!(cel1(0.31), 1.71978481);
        assert_relative_eq!(cel1(0.493), 1.84818918);
        assert_relative_eq!(cel1(0.5), 1.854074677);
        assert_relative_eq!(cel1(0.597), 1.94633962);
        assert_relative_eq!(cel1(0.618), 1.96950524);
        assert_relative_eq!(cel1(0.75), 2.15651565);
        assert_relative_eq!(cel1(0.801), 2.25948339);
        assert_relative_eq!(cel1(0.853), 2.39835076);
    }

    #[test]
    fn complete_integral_first_kind_high() {
        // Use the larger epsilon for the high tests
        assert_relative_eq!(cel1(0.903), 2.59243319, max_relative = FIVE_EPSILON);
        assert_relative_eq!(cel1(0.916), 2.66041638, max_relative = FIVE_EPSILON);
        assert_relative_eq!(cel1(0.924), 2.70791714, max_relative = FIVE_EPSILON);
        assert_relative_eq!(cel1(0.933), 2.76797262, max_relative = FIVE_EPSILON);
        assert_relative_eq!(cel1(0.948), 2.88945822, max_relative = FIVE_EPSILON);
        assert_relative_eq!(cel1(0.952), 2.92800816, max_relative = FIVE_EPSILON);
        assert_relative_eq!(cel1(0.961), 3.02837751, max_relative = FIVE_EPSILON);
        assert_relative_eq!(cel1(0.976), 3.26482357, max_relative = FIVE_EPSILON);
        assert_relative_eq!(cel1(0.985), 3.49554409, max_relative = FIVE_EPSILON);
        assert_relative_eq!(cel1(0.995), 4.03925748, max_relative = FIVE_EPSILON);
        assert_relative_eq!(cel1(1.), f32::INFINITY, max_relative = FIVE_EPSILON);
    }

    #[test]
    fn incomplete_integral_first_kind() {
        assert_relative_eq!(el1(FRAC_PI_4, 0.), 0.7853982, max_relative = FIVE_EPSILON);
        assert_relative_eq!(el1(FRAC_PI_4, 0.25), 0.8043661, max_relative = FIVE_EPSILON);
        assert_relative_eq!(el1(FRAC_PI_4, 0.5), 0.8260179, max_relative = FIVE_EPSILON);
        assert_relative_eq!(el1(FRAC_PI_4, 0.75), 0.8512237, max_relative = FIVE_EPSILON);
        assert_relative_eq!(el1(FRAC_PI_4, 1.0), 0.8813736, max_relative = FIVE_EPSILON);
    }

    #[test]
    fn complete_integral_second_kind() {
        assert_relative_eq!(cel2(0.062), 1.5461584);
        assert_relative_eq!(cel2(0.158), 1.5067791);
        assert_relative_eq!(cel2(0.217), 1.4817567);
        assert_relative_eq!(cel2(0.334), 1.4300116);
        assert_relative_eq!(cel2(0.409), 1.3951269);
        assert_relative_eq!(cel2(0.582), 1.3081238);
        assert_relative_eq!(cel2(0.618), 1.2885870);
        assert_relative_eq!(cel2(0.753), 1.2091616);
        assert_relative_eq!(cel2(0.829), 1.1584934);
        assert_relative_eq!(cel2(0.964), 1.0463593);
        assert_relative_eq!(cel2(0.99992), 1.0002241);
        assert_relative_eq!(cel2(0.999995), 1.0000175);
    }

    #[test]
    fn stress() {
        const N: usize = 1_000_000;
        let mut random = Vec::with_capacity(N);

        for _ in 0..N {
            random.push(rand::random());
        }

        let mut cloned = random.clone();
        let start = Instant::now();
        for r in random.iter_mut() {
            *r = cel2(*r);
        }
        let int_dur = Instant::now().duration_since(start);

        let start = Instant::now();
        for r in cloned.iter_mut() {
            *r = (*r).sin();
        }
        let sin_dur = Instant::now().duration_since(start);

        println!("{N} integrals took {}ms", int_dur.as_millis());
        println!("{N} sin() took {}ms", sin_dur.as_millis());
        println!(
            "Integrals are {:.2} times slower",
            int_dur.as_secs_f64() / sin_dur.as_secs_f64()
        );
    }
}
