use itertools::repeat_n;

pub trait SlackWebhookReporterStringExt {
    fn right_pad(self, width: usize, with: char) -> String;
}

impl<S> SlackWebhookReporterStringExt for S
where
    S: Into<String>,
{
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
