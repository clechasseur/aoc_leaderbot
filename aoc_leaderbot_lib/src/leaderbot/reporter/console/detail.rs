use itertools::repeat_n;

pub const STARS_HEADER: &str = "Stars ⭐";

pub trait ConsoleReporterStringExt {
    fn right_pad(self, width: usize, with: char) -> String;
}

impl<S> ConsoleReporterStringExt for S
where
    S: Into<String>,
{
    // noinspection DuplicatedCode
    fn right_pad(self, width: usize, with: char) -> String {
        let mut s = self.into();

        match width.saturating_sub(s.chars().count()) {
            0 => s,
            missing => {
                s.extend(repeat_n(with, missing));
                s
            },
        }
    }
}
