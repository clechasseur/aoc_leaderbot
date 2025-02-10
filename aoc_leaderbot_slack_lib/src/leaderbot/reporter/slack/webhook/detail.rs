pub trait SlackWebhookReporterStringExt {
    fn right_pad(self, width: usize, with: char) -> String;
}

impl<S> SlackWebhookReporterStringExt for S
where
    S: Into<String>,
{
    fn right_pad(self, width: usize, with: char) -> String {
        let mut s = self.into();

        let missing = width.saturating_sub(s.chars().count());
        for _ in 0..missing {
            s.push(with);
        }
        s
    }
}
