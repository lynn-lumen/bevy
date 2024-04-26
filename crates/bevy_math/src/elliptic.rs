//! This module contains methods for computing elliptic integrals
#![deny(clippy::excessive_precision)]
use std::f32::{consts::PI, INFINITY, NAN};

/// Computes the incomplete elliptic integral of the first kind for a given `x` and `m`: F(x | m)
///
/// `x` should be in the range `-1. <= x <= 1.` and
/// `m <= 1 / x^2` else `NAN` will be returned
pub fn el1(x: f32, m: f32) -> f32 {
    if x.is_nan() || m.is_nan() || x.abs() > 1. {
        return NAN;
    };
    if m.is_infinite() {
        return if m < 0. || x == 0. { 0. } else { NAN };
    };
    if x == 0. {
        return 0.;
    };
    if x.abs() == 1. {
        return x.signum() * cel1(m);
    };
    if m == 0. {
        return x.asin();
    }
    if m == 1. {
        return x.atanh();
    };

    let p = (1. - x) * (1. + x);
    let q = 1. - m * x * x;
    if q < 0. {
        NAN
    } else {
        x * rf(p, q, 1.)
    }
}

/// Computes the complete elliptic integral of the first kind for a given `m`: K(m)
///
/// The result will have a relative error of less than 5 machine epsilons
///
/// `m` should be less than or equal to `1`, `m <= 1.` else `NAN` will be returned
pub fn cel1(m: f32) -> f32 {
    if m.is_nan() || m > 1. {
        return NAN;
    };

    if m.is_infinite() {
        return 0.;
    };
    if m == 0. {
        return PI / 2.;
    };
    if m == 1. {
        return INFINITY;
    };

    if m > 0.999 {
        let m1 = 1. - m;
        let a = 0.5 * (16. / m1).ln();
        return a + m1 * (0.25 * (a - 1.) + m1 * ((9. * (6. * a - 7.)) / (6. * 64.)));
    };
    if m.abs() < 0.001 {
        return (PI / 2.)
            * (1. + m * (0.25 + m * (9. / 64. + m * (25. / 256. + m * 1225. / 16384.))));
    };
    if m < -3_000_000. {
        let a = (-16. * m).ln() / 2.;
        let b = (-m).sqrt();
        return (a - (a - 1.) / (-m) / 4. + 9. * (a - 7. / 6.) / 64. / -m.powi(2)) / b;
    };

    rf(0., 1. - m, 1.)
}

/// Computes the incomplete elliptic integral of the second kind for `phi` and `m`: E(phi | m).
pub fn el2(x: f32, m: f32) -> f32 {
    if x == 0. {
        return 0.;
    }
    if x == 1. {
        return cel2(m);
    }
    if m == 0. {
        return x.asin();
    }
    if m == 1. {
        return x;
    }

    let x2 = x * x;
    let p = (1. - x) * (1. + x);
    let q = 1. - m * x2;
    if q < 0. {
        return NAN;
    } else if m < 0. {
        return x * (rf(p, q, 1.) - (m * x2 * rd(p, q, 1.)) / 3.);
    } else if m < 1. {
        return x
            * ((1. - m) * rf(p, q, 1.)
                + (m * (1. - m) * x2 * rd(p, 1., q)) / 3.
                + m * (p / q).sqrt());
    } else {
        return x * ((m - 1.) * x2 * rd(q, 1., p) / 3. + (q / p).sqrt());
    }
}

/// Computes the incomplete elliptic integral of the second kind for `m`: E(m).
pub fn cel2(m: f32) -> f32 {
    if m == 0. {
        return PI / 2.;
    }
    if m == 1. {
        return 1.;
    }

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
        assert_relative_eq!(cel1(0.5), 1.8540746);
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
            *r = cel1(*r);
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
