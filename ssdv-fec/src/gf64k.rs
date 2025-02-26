use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// GF(2¹⁶) field element.
///
/// The finite field GF(2¹⁶) is constructed as a field extension of GF(2⁸),
/// implemented using [`GF256`]. It is realized as the quotient
/// GF(2⁸)\[y\] / (y² + x³y + 1),
/// where x denotes the generator of GF(2⁸). Arithmetic in this
/// field extension is implemented using simple ad-hoc formulas for a field
/// extension of degree two.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct GF64K(GF256, GF256);

/// GF(2⁸) field element.
///
/// The finite field GF(2⁸) is realized as the
/// quotient
/// GF(2)\[x\] / (x⁸ + x⁴ + x³ + x² + 1).
/// Its arithmetic is implemented
/// using tables of exponentials and logarithms.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct GF256(u8);

const GF64K_POLY_XCOEFF: GF256 = GF256(1 << 3);

impl From<u16> for GF64K {
    fn from(value: u16) -> GF64K {
        GF64K(GF256((value >> 8) as u8), GF256((value & 0xff) as u8))
    }
}

impl From<GF64K> for u16 {
    fn from(value: GF64K) -> u16 {
        ((u8::from(value.0) as u16) << 8) | u8::from(value.1) as u16
    }
}

impl From<u8> for GF256 {
    fn from(value: u8) -> GF256 {
        GF256(value)
    }
}

impl From<GF256> for u8 {
    fn from(value: GF256) -> u8 {
        value.0
    }
}

impl From<GF256> for GF64K {
    fn from(value: GF256) -> GF64K {
        GF64K(GF256(0), value)
    }
}

impl Add for GF64K {
    type Output = GF64K;
    fn add(self, rhs: GF64K) -> GF64K {
        GF64K(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign for GF64K {
    fn add_assign(&mut self, rhs: GF64K) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl Add for GF256 {
    type Output = GF256;
    // Addition is XOR, although it freaks out clippy
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: GF256) -> GF256 {
        GF256(self.0 ^ rhs.0)
    }
}

impl AddAssign for GF256 {
    // Addition is XOR, although it freaks out clippy
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: GF256) {
        self.0 ^= rhs.0;
    }
}

impl Sub for GF64K {
    type Output = GF64K;
    // We are in characteristic 2, so subtraction is addition
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: GF64K) -> GF64K {
        self + rhs
    }
}

impl SubAssign for GF64K {
    // We are in characteristic 2, so subtraction is addition
    #[allow(clippy::suspicious_op_assign_impl)]
    fn sub_assign(&mut self, rhs: GF64K) {
        *self += rhs;
    }
}

impl Sub for GF256 {
    type Output = GF256;
    // We are in characteristic 2, so subtraction is addition
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: GF256) -> GF256 {
        self + rhs
    }
}

impl SubAssign for GF256 {
    // We are in characteristic 2, so subtraction is addition
    #[allow(clippy::suspicious_op_assign_impl)]
    fn sub_assign(&mut self, rhs: GF256) {
        *self += rhs;
    }
}

impl Neg for GF64K {
    type Output = GF64K;
    fn neg(self) -> GF64K {
        self
    }
}

impl Neg for GF256 {
    type Output = GF256;
    fn neg(self) -> GF256 {
        self
    }
}

impl Mul for GF64K {
    type Output = GF64K;
    fn mul(self, rhs: GF64K) -> GF64K {
        let overflow = self.0 * rhs.0;
        GF64K(
            self.0 * rhs.1 + self.1 * rhs.0 + GF64K_POLY_XCOEFF * overflow,
            self.1 * rhs.1 + overflow,
        )
    }
}

impl MulAssign for GF64K {
    fn mul_assign(&mut self, rhs: GF64K) {
        *self = *self * rhs;
    }
}

impl Mul for GF256 {
    type Output = GF256;
    fn mul(self, rhs: GF256) -> GF256 {
        if self.0 == 0 || rhs.0 == 0 {
            GF256(0)
        } else {
            let a = GF256_LOG_TABLE[self.0 as usize];
            let b = GF256_LOG_TABLE[rhs.0 as usize];
            let c = a as u32 + b as u32;
            let c = if c >= 255 { c - 255 } else { c };
            GF256(GF256_EXP_TABLE[c as usize])
        }
    }
}

impl MulAssign for GF256 {
    fn mul_assign(&mut self, rhs: GF256) {
        *self = *self * rhs;
    }
}

impl Div for GF64K {
    type Output = GF64K;
    fn div(self, rhs: GF64K) -> GF64K {
        assert_ne!(rhs, GF64K(GF256(0), GF256(0)));
        // Compute the inverse by solving a 2x2 linear system over GF(2^8) using
        // Cramer's rule.
        let discr = rhs.1 * rhs.1 + GF64K_POLY_XCOEFF * rhs.0 * rhs.1 + rhs.0 * rhs.0;
        GF64K(
            (self.0 * rhs.1 + self.1 * rhs.0) / discr,
            (self.1 * (rhs.1 + GF64K_POLY_XCOEFF * rhs.0) + self.0 * rhs.0) / discr,
        )
    }
}

impl DivAssign for GF64K {
    fn div_assign(&mut self, rhs: GF64K) {
        *self = *self / rhs;
    }
}

impl Div for GF256 {
    type Output = GF256;
    fn div(self, rhs: GF256) -> GF256 {
        assert_ne!(rhs, GF256(0));
        if self.0 == 0 {
            GF256(0)
        } else {
            let a = GF256_LOG_TABLE[self.0 as usize];
            let b = GF256_LOG_TABLE[rhs.0 as usize];
            let c = 255 + a as u32 - b as u32;
            let c = if c >= 255 { c - 255 } else { c };
            GF256(GF256_EXP_TABLE[c as usize])
        }
    }
}

impl DivAssign for GF256 {
    fn div_assign(&mut self, rhs: GF256) {
        *self = *self / rhs;
    }
}

// GF(256) exponential table.
//
// The j-th entry of this table for j=0,...,254 contains the element xʲ encoded
// as a `u8`. The 255-th entry is not used in practice and contains `0u8`.
static GF256_EXP_TABLE: [u8; 256] = include!(concat!(env!("OUT_DIR"), "/gf256_exp_table.rs"));

// GF(256) logarithm table.
//
// The j-th entry of this table for j=1,...,255 contains the exponent k such
// that the element xᵏ encoded as a `u8` is equal to j. The 0-th entry contains
// `0u8`, since the logarithm of 0 is undefined."
static GF256_LOG_TABLE: [u8; 256] = include!(concat!(env!("OUT_DIR"), "/gf256_log_table.rs"));

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn powers_gf256() {
        let mut a = GF256(1);
        for j in 0..8 {
            assert_eq!(a, GF256(1 << j));
            a *= GF256(2);
        }
        assert_eq!(a, GF256(0b11101)); // x^8 = x^4 + x^3 + x^2 + 1
    }

    #[test]
    fn div_gf256() {
        let a = GF256(123);
        let b = GF256(187);
        let c = a / b;
        assert_eq!(c * b, a);
    }

    #[test]
    fn div_gf64k() {
        let a = GF64K(GF256(87), GF256(34));
        let b = GF64K(GF256(153), GF256(221));
        let c = a / b;
        assert_eq!(c * b, a);
        let b = GF64K(GF256(13), GF256(0));
        let c = a / b;
        assert_eq!(c * b, a);
        let b = GF64K(GF256(0), GF256(174));
        let c = a / b;
        assert_eq!(c * b, a);
    }

    #[test]
    fn gf64k_poly_root() {
        let y = GF64K(GF256(1), GF256(0));
        assert_eq!(
            y * y + GF64K::from(GF64K_POLY_XCOEFF) * y + 1.into(),
            0.into()
        );
    }

    #[test]
    fn gf64k_poly_irreducible_over_gf256() {
        for j in 0..=255 {
            let x = GF256(j);
            assert_ne!(x * x + GF64K_POLY_XCOEFF * x + 1.into(), 0.into());
        }
    }

    #[test]
    fn frobenius_gf256() {
        let a = GF256(27);
        let b = GF256(94);
        assert_eq!((a + b) * (a + b), a * a + b * b);
    }

    #[test]
    fn frobenius_gf64k() {
        let a = GF64K(GF256(143), GF256(239));
        let b = GF64K(GF256(28), GF256(147));
        assert_eq!((a + b) * (a + b), a * a + b * b);
    }
}
