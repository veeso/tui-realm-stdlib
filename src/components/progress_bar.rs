//! ## ProgressBar
//!
//! `ProgressBar` provides a component which shows the progress. It is possible to set the style for the progress bar and the text shown above it.

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::props::{
    Alignment, AttrValue, Attribute, Borders, Color, PropPayload, PropValue, Props, Style,
    TextModifiers,
};
use tuirealm::tui::{layout::Rect, widgets::Gauge};
use tuirealm::{Frame, MockComponent, State};

// -- Component

/// ## ProgressBar
///
/// provides a component which shows the progress. It is possible to set the style for the progress bar and the text shown above it.
#[derive(Default)]
pub struct ProgressBar {
    props: Props,
}

impl ProgressBar {
    pub fn foreground(mut self, fg: Color) -> Self {
        self.attr(Attribute::Foreground, AttrValue::Color(fg));
        self
    }

    pub fn background(mut self, bg: Color) -> Self {
        self.attr(Attribute::Background, AttrValue::Color(bg));
        self
    }

    pub fn borders(mut self, b: Borders) -> Self {
        self.attr(Attribute::Borders, AttrValue::Borders(b));
        self
    }

    pub fn modifiers(mut self, m: TextModifiers) -> Self {
        self.attr(Attribute::TextProps, AttrValue::TextModifiers(m));
        self
    }

    pub fn title<S: Into<String>>(mut self, t: S, a: Alignment) -> Self {
        self.attr(Attribute::Title, AttrValue::Title((t.into(), a)));
        self
    }

    pub fn label<S: Into<String>>(mut self, s: S) -> Self {
        self.attr(Attribute::Text, AttrValue::String(s.into()));
        self
    }

    pub fn progress(mut self, p: f64) -> Self {
        Self::assert_progress(p);
        self.attr(
            Attribute::Value,
            AttrValue::Payload(PropPayload::One(PropValue::F64(p))),
        );
        self
    }

    fn assert_progress(p: f64) {
        if !(0.0..=1.0).contains(&p) {
            panic!("Progress value must be in range [0.0, 1.0]");
        }
    }
}

impl MockComponent for ProgressBar {
    fn view(&mut self, render: &mut Frame, area: Rect) {
        // Make a Span
        if self.props.get_or(Attribute::Display, AttrValue::Flag(true)) == AttrValue::Flag(true) {
            // Text
            let label = self
                .props
                .get_or(Attribute::Text, AttrValue::String(String::default()))
                .unwrap_string();
            let foreground = self
                .props
                .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
                .unwrap_color();
            let background = self
                .props
                .get_or(Attribute::Background, AttrValue::Color(Color::Reset))
                .unwrap_color();
            let modifiers = self
                .props
                .get_or(
                    Attribute::TextProps,
                    AttrValue::TextModifiers(TextModifiers::empty()),
                )
                .unwrap_text_modifiers();
            let borders = self
                .props
                .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
                .unwrap_borders();
            let title = self.props.get(Attribute::Title).map(|x| x.unwrap_title());
            // Get percentage
            let percentage = self
                .props
                .get_or(
                    Attribute::Value,
                    AttrValue::Payload(PropPayload::One(PropValue::F64(0.0))),
                )
                .unwrap_payload()
                .unwrap_one()
                .unwrap_f64();
            let div = crate::utils::get_block(borders, title, true, None);
            // Make progress bar
            render.render_widget(
                Gauge::default()
                    .block(div)
                    .gauge_style(
                        Style::default()
                            .fg(foreground)
                            .bg(background)
                            .add_modifier(modifiers),
                    )
                    .label(label)
                    .ratio(percentage),
                area,
            );
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        if let Attribute::Value = attr {
            if let AttrValue::Payload(p) = value.clone() {
                Self::assert_progress(p.unwrap_one().unwrap_f64());
            }
        }
        self.props.set(attr, value)
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_components_progress_bar() {
        let component = ProgressBar::default()
            .background(Color::Red)
            .foreground(Color::White)
            .progress(0.60)
            .title("Downloading file...", Alignment::Center)
            .label("60% - ETA 00:20")
            .borders(Borders::default());
        // Get value
        assert_eq!(component.state(), State::None);
    }

    #[test]
    #[should_panic]
    fn test_components_progress_bar_bad_prog() {
        ProgressBar::default()
            .background(Color::Red)
            .foreground(Color::White)
            .progress(6.0)
            .title("Downloading file...", Alignment::Center)
            .label("60% - ETA 00:20")
            .borders(Borders::default());
    }
}
