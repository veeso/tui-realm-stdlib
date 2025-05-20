//! ## Textarea
//!
//! `Textarea` represents a read-only text component inside a container, the text is wrapped inside the container automatically
//! using the [textwrap](https://docs.rs/textwrap/0.13.4/textwrap/) crate.
//! The textarea supports multi-style spans and it is scrollable with arrows.

extern crate unicode_width;

use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{
    Alignment, AttrValue, Attribute, Borders, Color, PropPayload, PropValue, Props, Style,
    TextModifiers, TextSpan,
};
use tuirealm::ratatui::{
    layout::Rect,
    widgets::{List, ListItem, ListState},
};
use tuirealm::{Frame, MockComponent, State};
use unicode_width::UnicodeWidthStr;

// -- States

#[derive(Default)]
pub struct TextareaStates {
    pub list_index: usize, // Index of selected item in textarea
    pub list_len: usize,   // Lines in text area
}

impl TextareaStates {
    /// ### set_list_len
    ///
    /// Set list length and fix list index
    pub fn set_list_len(&mut self, len: usize) {
        self.list_len = len;
        self.fix_list_index();
    }

    /// ### incr_list_index
    ///
    /// Incremenet list index
    pub fn incr_list_index(&mut self) {
        // Check if index is at last element
        if self.list_index + 1 < self.list_len {
            self.list_index += 1;
        }
    }

    /// ### decr_list_index
    ///
    /// Decrement list index
    pub fn decr_list_index(&mut self) {
        // Check if index is bigger than 0
        if self.list_index > 0 {
            self.list_index -= 1;
        }
    }

    /// ### fix_list_index
    ///
    /// Keep index if possible, otherwise set to lenght - 1
    pub fn fix_list_index(&mut self) {
        if self.list_index >= self.list_len && self.list_len > 0 {
            self.list_index = self.list_len - 1;
        } else if self.list_len == 0 {
            self.list_index = 0;
        }
    }

    /// ### list_index_at_first
    ///
    /// Set list index to the first item in the list
    pub fn list_index_at_first(&mut self) {
        self.list_index = 0;
    }

    /// ### list_index_at_last
    ///
    /// Set list index at the last item of the list
    pub fn list_index_at_last(&mut self) {
        if self.list_len > 0 {
            self.list_index = self.list_len - 1;
        } else {
            self.list_index = 0;
        }
    }

    /// ### calc_max_step_ahead
    ///
    /// Calculate the max step ahead to scroll list
    fn calc_max_step_ahead(&self, max: usize) -> usize {
        let remaining: usize = match self.list_len {
            0 => 0,
            len => len - 1 - self.list_index,
        };
        if remaining > max {
            max
        } else {
            remaining
        }
    }

    /// ### calc_max_step_ahead
    ///
    /// Calculate the max step ahead to scroll list
    fn calc_max_step_behind(&self, max: usize) -> usize {
        if self.list_index > max {
            max
        } else {
            self.list_index
        }
    }
}

// -- Component

/// ## Textarea
///
/// represents a read-only text component without any container.
#[derive(Default)]
pub struct Textarea {
    props: Props,
    pub states: TextareaStates,
}

impl Textarea {
    pub fn foreground(mut self, fg: Color) -> Self {
        self.attr(Attribute::Foreground, AttrValue::Color(fg));
        self
    }

    pub fn background(mut self, bg: Color) -> Self {
        self.attr(Attribute::Background, AttrValue::Color(bg));
        self
    }

    pub fn inactive(mut self, s: Style) -> Self {
        self.attr(Attribute::FocusStyle, AttrValue::Style(s));
        self
    }

    pub fn modifiers(mut self, m: TextModifiers) -> Self {
        self.attr(Attribute::TextProps, AttrValue::TextModifiers(m));
        self
    }

    pub fn borders(mut self, b: Borders) -> Self {
        self.attr(Attribute::Borders, AttrValue::Borders(b));
        self
    }

    pub fn title<S: Into<String>>(mut self, t: S, a: Alignment) -> Self {
        self.attr(Attribute::Title, AttrValue::Title((t.into(), a)));
        self
    }

    pub fn step(mut self, step: usize) -> Self {
        self.attr(Attribute::ScrollStep, AttrValue::Length(step));
        self
    }

    pub fn highlighted_str<S: Into<String>>(mut self, s: S) -> Self {
        self.attr(Attribute::HighlightedStr, AttrValue::String(s.into()));
        self
    }

    pub fn text_rows(mut self, s: impl IntoIterator<Item = TextSpan>) -> Self {
        let rows: Vec<PropValue> = s.into_iter().map(PropValue::TextSpan).collect();
        self.states.set_list_len(rows.len());
        self.attr(Attribute::Text, AttrValue::Payload(PropPayload::Vec(rows)));
        self
    }
}

impl MockComponent for Textarea {
    fn view(&mut self, render: &mut Frame, area: Rect) {
        // Make a Span
        if self.props.get_or(Attribute::Display, AttrValue::Flag(true)) == AttrValue::Flag(true) {
            // Make text items
            // Highlighted symbol
            let hg_str = self
                .props
                .get_ref(Attribute::HighlightedStr)
                .and_then(|x| x.as_string());
            // NOTE: wrap width is width of area minus 2 (block) minus width of highlighting string
            let wrap_width =
                (area.width as usize) - hg_str.as_ref().map(|x| x.width()).unwrap_or(0) - 2;
            let lines: Vec<ListItem> = match self
                .props
                .get_ref(Attribute::Text)
                .and_then(|x| x.as_payload())
            {
                Some(PropPayload::Vec(spans)) => spans
                    .iter()
                    // this will skip any "PropValue" that is not a "TextSpan", instead of panicing
                    .flat_map(|x| x.as_text_span())
                    .map(|x| crate::utils::wrap_spans(&[x], wrap_width, &self.props))
                    .map(ListItem::new)
                    .collect(),
                _ => Vec::new(),
            };
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
            let title = crate::utils::get_title_or_center(&self.props);
            let borders = self
                .props
                .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
                .unwrap_borders();
            let focus = self
                .props
                .get_or(Attribute::Focus, AttrValue::Flag(false))
                .unwrap_flag();
            let inactive_style = self
                .props
                .get(Attribute::FocusStyle)
                .map(|x| x.unwrap_style());
            let mut state: ListState = ListState::default();
            state.select(Some(self.states.list_index));
            // Make component

            let mut list = List::new(lines)
                .block(crate::utils::get_block(
                    borders,
                    Some(&title),
                    focus,
                    inactive_style,
                ))
                .direction(tuirealm::ratatui::widgets::ListDirection::TopToBottom)
                .style(
                    Style::default()
                        .fg(foreground)
                        .bg(background)
                        .add_modifier(modifiers),
                );

            if let Some(hg_str) = hg_str {
                list = list.highlight_symbol(hg_str);
            }
            render.render_stateful_widget(list, area, &mut state);
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
        // Update list len and fix index
        self.states.set_list_len(
            match self.props.get(Attribute::Text).map(|x| x.unwrap_payload()) {
                Some(PropPayload::Vec(spans)) => spans.len(),
                _ => 0,
            },
        );
        self.states.fix_list_index();
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Down) => {
                self.states.incr_list_index();
            }
            Cmd::Move(Direction::Up) => {
                self.states.decr_list_index();
            }
            Cmd::Scroll(Direction::Down) => {
                let step = self
                    .props
                    .get_or(Attribute::ScrollStep, AttrValue::Length(8))
                    .unwrap_length();
                let step = self.states.calc_max_step_ahead(step);
                (0..step).for_each(|_| self.states.incr_list_index());
            }
            Cmd::Scroll(Direction::Up) => {
                let step = self
                    .props
                    .get_or(Attribute::ScrollStep, AttrValue::Length(8))
                    .unwrap_length();
                let step = self.states.calc_max_step_behind(step);
                (0..step).for_each(|_| self.states.decr_list_index());
            }
            Cmd::GoTo(Position::Begin) => {
                self.states.list_index_at_first();
            }
            Cmd::GoTo(Position::End) => {
                self.states.list_index_at_last();
            }
            _ => {}
        }
        CmdResult::None
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_components_textarea() {
        // Make component
        let mut component = Textarea::default()
            .foreground(Color::Red)
            .background(Color::Blue)
            .modifiers(TextModifiers::BOLD)
            .borders(Borders::default())
            .highlighted_str("🚀")
            .step(4)
            .title("textarea", Alignment::Center)
            .text_rows([TextSpan::from("welcome to "), TextSpan::from("tui-realm")]);
        // Increment list index
        component.states.list_index += 1;
        assert_eq!(component.states.list_index, 1);
        // Add one row
        component.attr(
            Attribute::Text,
            AttrValue::Payload(PropPayload::Vec(vec![
                PropValue::TextSpan(TextSpan::from("welcome")),
                PropValue::TextSpan(TextSpan::from("to")),
                PropValue::TextSpan(TextSpan::from("tui-realm")),
            ])),
        );
        // Verify states
        assert_eq!(component.states.list_index, 1); // Kept
        assert_eq!(component.states.list_len, 3);
        // get value
        assert_eq!(component.state(), State::None);
        // Render
        assert_eq!(component.states.list_index, 1);
        // Handle inputs
        assert_eq!(
            component.perform(Cmd::Move(Direction::Down)),
            CmdResult::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 2);
        // Index should be decremented
        assert_eq!(component.perform(Cmd::Move(Direction::Up)), CmdResult::None);
        // Index should be incremented
        assert_eq!(component.states.list_index, 1);
        // Index should be 2
        assert_eq!(
            component.perform(Cmd::Scroll(Direction::Down)),
            CmdResult::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 2);
        // Index should be 0
        assert_eq!(
            component.perform(Cmd::Scroll(Direction::Up)),
            CmdResult::None
        );
        // End
        assert_eq!(component.perform(Cmd::GoTo(Position::End)), CmdResult::None);
        assert_eq!(component.states.list_index, 2);
        // Home
        assert_eq!(
            component.perform(Cmd::GoTo(Position::Begin)),
            CmdResult::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 0);
        // On key
        assert_eq!(component.perform(Cmd::Delete), CmdResult::None);
    }

    #[test]
    fn various_textrows_types() {
        // Vec
        let _ = Textarea::default().text_rows(vec![TextSpan::new("hello")]);
        // static array
        let _ = Textarea::default().text_rows([TextSpan::new("hello")]);
        // boxed array
        let _ = Textarea::default().text_rows(vec![TextSpan::new("hello")].into_boxed_slice());
        // already a iterator
        let _ = Textarea::default().text_rows(["Hello"].map(TextSpan::new));
    }
}
