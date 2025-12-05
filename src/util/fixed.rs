use anyhow::{Error, Result, anyhow};
use num::{CheckedAdd, CheckedSub, Num, One, Signed, Zero};
use serde::Deserialize;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::str::FromStr;

#[derive(Debug, Deserialize, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Fixed(i128);

impl FromStr for Fixed {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(".").collect();

        // ensure we only have 1 decimal point
        match parts[..] {
            [integer_str, fraction_str] => {
                let integer = integer_str.parse::<i128>().map_err(|err| {
                    anyhow!("Invalid integer part of fixed point value {s}: {err}")
                })?;
                let fraction = fraction_str.parse::<u32>().map_err(|err| {
                    anyhow!("Invalid fractional part of fixed point value {s}: {err}")
                })?;

                let integer_x_10_000 = integer
                    .checked_mul(10_000)
                    .ok_or(anyhow!("Integer part of fixed point value to large: {s}"))?;

                // handle trailing 0s
                let fraction_x_10_000 = if fraction < 10 {
                    Ok(fraction * 1_000)
                } else if fraction < 100 {
                    Ok(fraction * 100)
                } else if fraction < 1_000 {
                    Ok(fraction * 10)
                } else if fraction < 10_000 {
                    Ok(fraction)
                } else {
                    Err(anyhow!(
                        "Fractional part of fixed point value to large: {s}"
                    ))
                }?;

                Ok(Fixed(integer_x_10_000 + fraction_x_10_000 as i128))
            }
            _ => Err(anyhow!("Invalid fixed point value: {s}")),
        }
    }
}

impl fmt::Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.0 / 10_000, self.0 % 10_000)
    }
}

// In my opinion CheckedAdd should not require Add to be implemented
impl Add for Fixed {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Fixed(self.0 + rhs.0)
    }
}

impl CheckedAdd for Fixed {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        self.0.checked_add(v.0).map(Fixed)
    }
}

impl Sub for Fixed {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Fixed(self.0 - rhs.0)
    }
}

impl CheckedSub for Fixed {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        self.0.checked_sub(v.0).map(Fixed)
    }
}

impl Neg for Fixed {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Fixed(-self.0)
    }
}

impl Rem for Fixed {
    type Output = Fixed;

    fn rem(self, rhs: Self) -> Self::Output {
        Fixed(self.0.rem(rhs.0))
    }
}

impl Div for Fixed {
    type Output = Fixed;

    fn div(self, rhs: Self) -> Self::Output {
        Fixed(self.0.div(rhs.0))
    }
}

impl Mul for Fixed {
    type Output = Fixed;

    fn mul(self, rhs: Self) -> Self::Output {
        Fixed(self.0.mul(rhs.0))
    }
}

impl Zero for Fixed {
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    fn set_zero(&mut self) {
        self.0.set_zero();
    }

    fn zero() -> Self {
        Fixed(i128::zero())
    }
}

impl One for Fixed {
    fn is_one(&self) -> bool
    where
        Self: PartialEq,
    {
        self.0.is_one()
    }

    fn one() -> Self {
        Self(i128::one())
    }
}

impl Num for Fixed {
    type FromStrRadixErr = Error;

    fn from_str_radix(src: &str, radix: u32) -> Result<Self, <Self as Num>::FromStrRadixErr> {
        let parts: Vec<&str> = src.split(".").collect();

        // ensure we only have 1 decimal point
        match parts[..] {
            [integer_str, fraction_str] => {
                let integer = i128::from_str_radix(integer_str, radix)?;
                let fraction = u32::from_str_radix(fraction_str, radix)?;

                let integer_x_10_000 = integer
                    .checked_mul(10_000)
                    .ok_or(anyhow!("Integer part of fixed point value to large: {src}"))?;

                // handle trailing 0s
                let fraction_x_10_000 = if fraction < 10 {
                    Ok(fraction * 1_000)
                } else if fraction < 100 {
                    Ok(fraction * 100)
                } else if fraction < 1_000 {
                    Ok(fraction * 10)
                } else if fraction < 10_000 {
                    Ok(fraction)
                } else {
                    Err(anyhow!(
                        "Fractional part of fixed point value to large: {src}"
                    ))
                }?;

                Ok(Fixed(integer_x_10_000 + fraction_x_10_000 as i128))
            }
            _ => Err(anyhow!("Invalid fixed point value: {src}")),
        }
    }
}

impl Signed for Fixed {
    fn abs(&self) -> Self {
        Fixed(self.0.abs())
    }

    fn abs_sub(&self, other: &Self) -> Self {
        Fixed(self.0.abs_sub(&other.0))
    }

    fn signum(&self) -> Self {
        Fixed(self.0.signum())
    }

    fn is_positive(&self) -> bool {
        self.0.is_positive()
    }

    fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}

// impl Fixed {
//     pub fn zero() -> Self {
//         Self {
//             data: 0
//         }
//     }
// }
