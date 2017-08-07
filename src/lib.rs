//! Order floating point numbers, into this ordering:
//!
//!    NaN | -Infinity | x < 0 | -0 | +0 | x > 0 | +Infinity | NaN

#![no_std]

#[cfg(feature="pdqsort")]
extern crate pdqsort;

use core::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use core::hash::{Hash, Hasher};
use core::mem::transmute;

/// A wrapper for floats, that implements total equality and ordering
/// and hashing.
#[derive(Clone, Copy, Debug)]
pub struct FloatOrd<T>(pub T);

macro_rules! float_ord_impl {
    ($f:ident, $i:ident, $n:expr) => {
        impl FloatOrd<$f> {
            fn convert(self) -> $i {
                let u = unsafe { transmute::<$f, $i>(self.0) };
                let bit = 1 << ($n - 1);
                if u & bit == 0 {
                    u | bit
                } else {
                    !u
                }
            }
        }
        impl PartialEq for FloatOrd<$f> {
            fn eq(&self, other: &Self) -> bool {
                self.convert() == other.convert()
            }
        }
        impl Eq for FloatOrd<$f> {}
        impl PartialOrd for FloatOrd<$f> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.convert().partial_cmp(&other.convert())
            }
        }
        impl Ord for FloatOrd<$f> {
            fn cmp(&self, other: &Self) -> Ordering {
                self.convert().cmp(&other.convert())
            }
        }
        impl Hash for FloatOrd<$f> {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.convert().hash(state);
            }
        }
    }
}

float_ord_impl!(f32, u32, 32);
float_ord_impl!(f64, u64, 64);

impl<T> Default for FloatOrd<T>
    where T: Default
{
    fn default() -> FloatOrd<T> {
        FloatOrd(T::default())
    }
}

use core::ops::{Deref, Add, Sub, Mul, Div, Rem};

impl<T> Deref for FloatOrd<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! float_ord_ops_impl {
    ($t:ident, $f:ident) => {
        // FloatOrd<T> + FloatOrd<T> impl
        impl<T> $t for FloatOrd<T>
            where T: $t<Output = T>
        {
            type Output = Self;
            fn $f(self, rhs: Self) -> Self::Output {
                FloatOrd((self.0).$f(rhs.0))
            }
        }

        // FloatOrd<T> + T impl
        impl <T> $t<T> for FloatOrd<T> where T: $t<Output = T> {
            type Output = Self;
            fn $f(self, rhs: T) -> Self::Output {
                FloatOrd((self.0).$f(rhs))
            }
        }

        // A few impl's of T + FloatOrd<U>
        impl $t<FloatOrd<f64>> for f64 {
            type Output = FloatOrd<f64>;
            fn $f(self, rhs: FloatOrd<f64>) -> Self::Output {
                FloatOrd((self).$f(rhs.0))
            }
        }

        impl $t<FloatOrd<f32>> for f32 {
            type Output = FloatOrd<f32>;
            fn $f(self, rhs: FloatOrd<f32>) -> Self::Output {
                FloatOrd((self).$f(rhs.0))
            }
        }

        impl $t<FloatOrd<f64>> for f32 {
            type Output = FloatOrd<f64>;
            fn $f(self, rhs: FloatOrd<f64>) -> Self::Output {
                FloatOrd((self as f64).$f(rhs.0))
            }
        }
    }
}

float_ord_ops_impl!(Add, add);
float_ord_ops_impl!(Div, div);
float_ord_ops_impl!(Rem, rem);
float_ord_ops_impl!(Mul, mul);
float_ord_ops_impl!(Sub, sub);



#[cfg(feature="pdqsort")]
/// Sort a slice of floats.
///
/// # Allocation behavior
///
/// This routine uses a quicksort implementation that does not heap allocate.
///
/// # Example
///
/// ```
/// let mut v = [-5.0, 4.0, 1.0, -3.0, 2.0];
///
/// float_ord::sort(&mut v);
/// assert!(v == [-5.0, -3.0, 1.0, 2.0, 4.0]);
/// ```
pub fn sort<T>(v: &mut [T])
    where FloatOrd<T>: Ord
{
    let v_: &mut [FloatOrd<T>] = unsafe { transmute(v) };
    pdqsort::sort(v_);
}

#[cfg(test)]
mod tests {
    extern crate std;
    extern crate rand;

    use self::rand::{Rng, thread_rng};
    use self::std::prelude::v1::*;
    use self::std::collections::hash_map::DefaultHasher;
    use self::std::hash::{Hash, Hasher};
    use self::std::f64;
    use super::FloatOrd;

    #[test]
    fn test_ord() {
        assert!(FloatOrd(1.0f64) < FloatOrd(2.0f64));
        assert!(FloatOrd(2.0f32) > FloatOrd(1.0f32));
        assert!(FloatOrd(1.0f64) == FloatOrd(1.0f64));
        assert!(FloatOrd(1.0f32) == FloatOrd(1.0f32));
        assert!(FloatOrd(0.0f64) > FloatOrd(-0.0f64));
        assert!(FloatOrd(0.0f32) > FloatOrd(-0.0f32));
        assert!(FloatOrd(::core::f64::NAN) == FloatOrd(::core::f64::NAN));
        assert!(FloatOrd(::core::f32::NAN) == FloatOrd(::core::f32::NAN));
        assert!(FloatOrd(-::core::f64::NAN) < FloatOrd(::core::f64::NAN));
        assert!(FloatOrd(-::core::f32::NAN) < FloatOrd(::core::f32::NAN));
        assert!(FloatOrd(-::core::f64::INFINITY) < FloatOrd(::core::f64::INFINITY));
        assert!(FloatOrd(-::core::f32::INFINITY) < FloatOrd(::core::f32::INFINITY));
        assert!(FloatOrd(::core::f64::INFINITY) < FloatOrd(::core::f64::NAN));
        assert!(FloatOrd(::core::f32::INFINITY) < FloatOrd(::core::f32::NAN));
        assert!(FloatOrd(-::core::f64::NAN) < FloatOrd(::core::f64::INFINITY));
        assert!(FloatOrd(-::core::f32::NAN) < FloatOrd(::core::f32::INFINITY));
    }

    #[test]
    fn test_ord_numbers() {
        let mut rng = thread_rng();
        for n in 0..16 {
            for l in 0..16 {
                let v = rng.gen_iter::<f64>()
                    .map(|x| x % (1 << l) as i64 as f64)
                    .take((1 << n))
                    .collect::<Vec<_>>();
                assert!(v.windows(2)
                            .all(|w| (w[0] <= w[1]) == (FloatOrd(w[0]) <= FloatOrd(w[1]))));
            }
        }
    }

    fn hash<F: Hash>(f: F) -> u64 {
        let mut hasher = DefaultHasher::new();
        f.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_hash() {
        assert_ne!(hash(FloatOrd(0.0f64)), hash(FloatOrd(-0.0f64)));
        assert_ne!(hash(FloatOrd(0.0f32)), hash(FloatOrd(-0.0f32)));
        assert_eq!(hash(FloatOrd(-0.0f64)), hash(FloatOrd(-0.0f64)));
        assert_eq!(hash(FloatOrd(0.0f32)), hash(FloatOrd(0.0f32)));
        assert_ne!(hash(FloatOrd(::core::f64::NAN)),
                   hash(FloatOrd(-::core::f64::NAN)));
        assert_ne!(hash(FloatOrd(::core::f32::NAN)),
                   hash(FloatOrd(-::core::f32::NAN)));
        assert_eq!(hash(FloatOrd(::core::f64::NAN)),
                   hash(FloatOrd(::core::f64::NAN)));
        assert_eq!(hash(FloatOrd(-::core::f32::NAN)),
                   hash(FloatOrd(-::core::f32::NAN)));
    }

    #[cfg(feature="pdqsort")]
    #[test]
    fn test_sort_numbers() {
        let mut rng = thread_rng();
        for n in 0..16 {
            for l in 0..16 {
                let mut v = rng.gen_iter::<f64>()
                    .map(|x| x % (1 << l) as i64 as f64)
                    .take((1 << n))
                    .collect::<Vec<_>>();
                let mut v1 = v.clone();

                super::sort(&mut v);
                assert!(v.windows(2).all(|w| w[0] <= w[1]));

                v1.sort_by(|a, b| a.partial_cmp(b).unwrap());
                assert!(v1.windows(2).all(|w| w[0] <= w[1]));

                v1.sort_by(|a, b| b.partial_cmp(a).unwrap());
                assert!(v1.windows(2).all(|w| w[0] >= w[1]));
            }
        }

        let mut v = [5.0];
        super::sort(&mut v);
        assert!(v == [5.0]);
    }

    #[cfg(feature="pdqsort")]
    #[test]
    fn test_sort_nan() {
        let nan = ::core::f64::NAN;
        let mut v = [-1.0, 5.0, 0.0, -0.0, nan, 1.5, nan, 3.7];
        super::sort(&mut v);
        assert!(v[0] == -1.0);
        assert!(v[1] == 0.0 && v[1].is_sign_negative());
        assert!(v[2] == 0.0 && !v[2].is_sign_negative());
        assert!(v[3] == 1.5);
        assert!(v[4] == 3.7);
        assert!(v[5] == 5.0);
        assert!(v[6].is_nan());
        assert!(v[7].is_nan());
    }

    #[test]
    fn test_add() {
        assert_eq!(FloatOrd(1.5) + FloatOrd(1.5), FloatOrd(1.5 + 1.5));
        assert_eq!(1.5 + FloatOrd(1.5), FloatOrd(1.5 + 1.5));
        assert_eq!(FloatOrd(1.5) + 1.5, FloatOrd(1.5 + 1.5));
    }

    #[test]
    fn test_deref() {
        // Should be able to call methods exposed on floats directly.
        let f = FloatOrd(2.71828_f64);
        assert_eq!(f.floor(), 2.0);
        assert_eq!(f.ceil(), 3.0);
        assert_eq!(f.round(), 3.0);
    }
}
