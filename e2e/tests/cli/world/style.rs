#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Plain,
    Bold,
    Red,
    Green,
    Yellow,
    Blue,
    Cyan,
    SolarizedBlue,
    SolarizedRed,
    SolarizedGreen,
}

impl Style {
    #[track_caller]
    pub fn of(code: &str) -> Style {
        match code {
            "0" => Style::Plain,
            "1" => Style::Bold,
            "31" => Style::Red,
            "32" => Style::Green,
            "33" => Style::Yellow,
            "34" => Style::Blue,
            "36" => Style::Cyan,
            "38;2;38;139;210" => Style::SolarizedBlue,
            "38;2;220;50;47" => Style::SolarizedRed,
            "38;2;133;153;0" => Style::SolarizedGreen,
            other => panic!("the viewer used a style the tests don't name: {other:?}"),
        }
    }
}
