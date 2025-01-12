use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

#[derive(Debug, Clone, Copy)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0., im: 0. };

    #[inline]
    pub const fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }

    #[inline]
    pub const fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn powu(self, n: usize) -> Complex {
        (0..n).fold(self, |acc, _| acc * acc)
    }
}

impl Add for Complex {
    type Output = Complex;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl AddAssign for Complex {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl Mul for Complex {
    type Output = Complex;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let k1 = rhs.re * (self.re + self.im);
        let k2 = self.re * (rhs.im - rhs.re);
        let k3 = self.im * (rhs.re + rhs.im);

        Complex {
            re: k1 - k3,
            im: k1 + k2,
        }
    }
}

impl Neg for Complex {
    type Output = Complex;

    #[inline]
    fn neg(self) -> Self::Output {
        Complex {
            re: -self.re,
            im: -self.im,
        }
    }
}

impl Sub for Complex {
    type Output = Complex;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Complex {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl SubAssign for Complex {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}
