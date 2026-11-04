use std::process::Output;

#[derive(Debug)]
pub struct Ran(Output);

impl Ran {
    pub fn new(output: Output) -> Ran {
        Ran(output)
    }

    #[track_caller]
    pub fn succeeds(&self) -> &Self {
        assert!(self.0.status.success(), "expected success: {self:#?}");
        self
    }

    #[track_caller]
    pub fn exits(&self, code: i32) -> &Self {
        assert_eq!(self.0.status.code(), Some(code), "{self:#?}");
        self
    }

    #[track_caller]
    pub fn prints(&self, expected: &str) -> &Self {
        assert_eq!(self.printed(), expected, "{self:#?}");
        self
    }

    #[track_caller]
    pub fn prints_error(&self, expected: &str) -> &Self {
        let printed = str::from_utf8(&self.0.stderr).expect("play-by-play errors in UTF-8");
        assert_eq!(printed, expected, "{self:#?}");
        self
    }

    pub fn printed(&self) -> &str {
        str::from_utf8(&self.0.stdout).expect("play-by-play prints UTF-8")
    }
}
