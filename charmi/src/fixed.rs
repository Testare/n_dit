use std::borrow::Cow;

use super::CharmiCell;

#[derive(Clone, Debug, PartialEq)]
pub struct CharmiFixed {
    grid: Cow<'static, [Option<CharmiCell>]>,
    height: usize,
    width: usize,
}

impl CharmiFixed {
    pub const fn from_slice(
        width: usize,
        height: usize,
        slice: &'static [Option<CharmiCell>],
    ) -> Self {
        if slice.len() != height * width {
            panic!("vec input should be the same as width * height")
        }
        Self::from_cow(width, height, Cow::Borrowed(slice))
    }

    pub fn from_vec(width: usize, height: usize, vec: Vec<Option<CharmiCell>>) -> Self {
        if vec.len() != height * width {
            panic!("vec input should be the same as width * height")
        }
        Self::from_cow(width, height, Cow::Owned(vec))
    }

    const fn from_cow(
        width: usize,
        height: usize,
        grid: Cow<'static, [Option<CharmiCell>]>,
    ) -> Self {
        CharmiFixed {
            grid,
            height,
            width,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CharmiStr(Cow<'static, [Option<CharmiCell>]>);

impl CharmiStr {
    pub const fn from_slice(slice: &'static [Option<CharmiCell>]) -> Self {
        CharmiStr(Cow::Borrowed(slice))
    }

    pub fn from_vec(vec: Vec<Option<CharmiCell>>) -> Self {
        CharmiStr(Cow::Owned(vec))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test() {
        static CHARMI_STR_TEST: CharmiStr =
            CharmiStr(Cow::Borrowed(&[Some(CharmiCell::new_blank())]));
        static CHARMI_STR_TEST_2: CharmiStr =
            CharmiStr::from_slice(&[Some(CharmiCell::new_blank())]);
        let charmstr: CharmiStr = CharmiStr(Cow::Owned(vec![]));
        assert_eq!(CHARMI_STR_TEST, CHARMI_STR_TEST_2);
        assert_eq!(CHARMI_STR_TEST, charmstr);
    }
}
