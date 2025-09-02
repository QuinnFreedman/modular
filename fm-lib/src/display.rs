use core::ops::Deref;
use ufmt::{uDisplay, uWrite, uwrite};

pub struct DisplayableFloat(pub f32);

impl From<f32> for DisplayableFloat {
    #[inline]
    fn from(x: f32) -> Self {
        Self(x)
    }
}

impl From<DisplayableFloat> for f32 {
    #[inline]
    fn from(x: DisplayableFloat) -> f32 {
        x.0
    }
}

impl Deref for DisplayableFloat {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl uDisplay for DisplayableFloat {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        let mut val = self.0;

        if val.is_nan() {
            return f.write_str("NaN");
        }
        if val.is_infinite() {
            if val.is_sign_negative() {
                return f.write_str("-inf");
            } else {
                return f.write_str("inf");
            }
        }

        if val < 0.0 {
            f.write_str("-")?;
            val = -val;
        }

        let int_part = val as u32;
        let mut frac = val - (int_part as f32);

        uwrite!(f, "{}.", int_part)?;

        for _ in 0..5 {
            frac *= 10.0;
            let digit = frac as u32;
            frac -= digit as f32;
            let c = b'0' + (digit as u8);
            f.write_char(c as char)?;
        }

        Ok(())
    }
}

pub fn show_float<T>(x: T) -> DisplayableFloat
where
    T: Into<f32>,
{
    DisplayableFloat(x.into())
}
