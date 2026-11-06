use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Duration(i64);

impl Duration {
    pub const ZERO: Self = Self(0);

    pub const fn microseconds(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_microseconds(self) -> i64 {
        self.0
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Duration;

    #[test]
    fn checked_arithmetic_reports_overflow() {
        assert_eq!(
            Duration::microseconds(i64::MAX).checked_add(Duration::microseconds(1)),
            None
        );
        assert_eq!(
            Duration::microseconds(i64::MIN).checked_sub(Duration::microseconds(1)),
            None
        );
    }
}
