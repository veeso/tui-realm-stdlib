//! ## Select
//!
//! `Select` represents a select field, like in HTML. The size for the component must be 3 (border + selected) + the quantity of rows
//! you want to display other options when opened (at least 3)

use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::props::{
    Alignment, AttrValue, Attribute, BorderSides, Borders, Color, PropPayload, PropValue, Props,
    Style, TextModifiers,
};
use tuirealm::ratatui::text::Line as Spans;
use tuirealm::ratatui::{
    layout::{Constraint, Direction as LayoutDirection, Layout, Rect},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};
use tuirealm::{Frame, MockComponent, State, StateValue};

// -- states

/// ## SelectStates
///
/// Component states
#[derive(Default)]
pub struct SelectStates {
    /// Available choices
    pub choices: Vec<String>,
    /// Currently selected choice
    pub selected: usize,
    /// Choice selected before opening the tab
    pub previously_selected: usize,
    pub tab_open: bool,
}

impl SelectStates {
    /// ### next_choice
    ///
    /// Move choice index to next choice
    pub fn next_choice(&mut self, rewind: bool) {
        if self.tab_open {
            if rewind && self.selected + 1 >= self.choices.len() {
                self.selected = 0;
            } else if self.selected + 1 < self.choices.len() {
                self.selected += 1;
            }
        }
    }

    /// ### prev_choice
    ///
    /// Move choice index to previous choice
    pub fn prev_choice(&mut self, rewind: bool) {
        if self.tab_open {
            if rewind && self.selected == 0 && !self.choices.is_empty() {
                self.selected = self.choices.len() - 1;
            } else if self.selected > 0 {
                self.selected -= 1;
            }
        }
    }

    /// ### set_choices
    ///
    /// Set SelectStates choices from a vector of str
    /// In addition resets current selection and keep index if possible or set it to the first value
    /// available
    pub fn set_choices(&mut self, choices: &[String]) {
        self.choices = choices.to_vec();
        // Keep index if possible
        if self.selected >= self.choices.len() {
            self.selected = match self.choices.len() {
                0 => 0,
                l => l - 1,
            };
        }
    }

    pub fn select(&mut self, i: usize) {
        if i < self.choices.len() {
            self.selected = i;
        }
    }

    /// ### close_tab
    ///
    /// Close tab
    pub fn close_tab(&mut self) {
        self.tab_open = false;
    }

    /// ### open_tab
    ///
    /// Open tab
    pub fn open_tab(&mut self) {
        self.previously_selected = self.selected;
        self.tab_open = true;
    }

    /// Cancel tab open
    pub fn cancel_tab(&mut self) {
        self.close_tab();
        self.selected = self.previously_selected;
    }

    /// ### is_tab_open
    ///
    /// Returns whether the tab is open
    #[must_use]
    pub fn is_tab_open(&self) -> bool {
        self.tab_open
    }
}

// -- component

#[derive(Default)]
#[must_use]
pub struct Select {
    props: Props,
    pub states: SelectStates,
    hg_str: Option<String>, // CRAP CRAP CRAP
}

impl Select {
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

    pub fn title<S: Into<String>>(mut self, t: S, a: Alignment) -> Self {
        self.attr(Attribute::Title, AttrValue::Title((t.into(), a)));
        self
    }

    pub fn highlighted_str<S: Into<String>>(mut self, s: S) -> Self {
        self.attr(Attribute::HighlightedStr, AttrValue::String(s.into()));
        self
    }

    pub fn highlighted_color(mut self, c: Color) -> Self {
        self.attr(Attribute::HighlightedColor, AttrValue::Color(c));
        self
    }

    pub fn inactive(mut self, s: Style) -> Self {
        self.attr(Attribute::FocusStyle, AttrValue::Style(s));
        self
    }

    pub fn rewind(mut self, r: bool) -> Self {
        self.attr(Attribute::Rewind, AttrValue::Flag(r));
        self
    }

    pub fn choices<S: AsRef<str>>(mut self, choices: &[S]) -> Self {
        self.attr(
            Attribute::Content,
            AttrValue::Payload(PropPayload::Vec(
                choices
                    .iter()
                    .map(|x| PropValue::Str(x.as_ref().to_string()))
                    .collect(),
            )),
        );
        self
    }

    pub fn value(mut self, i: usize) -> Self {
        // Set state
        self.attr(
            Attribute::Value,
            AttrValue::Payload(PropPayload::One(PropValue::Usize(i))),
        );
        self
    }

    /// ### render_open_tab
    ///
    /// Render component when tab is open
    fn render_open_tab(&mut self, render: &mut Frame, area: Rect) {
        // Make choices
        let choices: Vec<ListItem> = self
            .states
            .choices
            .iter()
            .map(|x| ListItem::new(Spans::from(x.as_str())))
            .collect();
        let foreground = self
            .props
            .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let background = self
            .props
            .get_or(Attribute::Background, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let hg: Color = self
            .props
            .get_or(Attribute::HighlightedColor, AttrValue::Color(foreground))
            .unwrap_color();
        // Prepare layout
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .margin(0)
            .constraints([Constraint::Length(2), Constraint::Min(1)].as_ref())
            .split(area);
        // Render like "closed" tab in chunk 0
        let selected_text: String = match self.states.choices.get(self.states.selected) {
            None => String::default(),
            Some(s) => s.clone(),
        };
        let borders = self
            .props
            .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
            .unwrap_borders();
        let block: Block = Block::default()
            .borders(BorderSides::LEFT | BorderSides::TOP | BorderSides::RIGHT)
            .border_style(borders.style())
            .border_type(borders.modifiers)
            .style(Style::default().bg(background));
        let title = self.props.get(Attribute::Title).map(|x| x.unwrap_title());
        let block = match title {
            Some((text, alignment)) => block.title(text).title_alignment(alignment),
            None => block,
        };
        let focus = self
            .props
            .get_or(Attribute::Focus, AttrValue::Flag(false))
            .unwrap_flag();
        let inactive_style = self
            .props
            .get(Attribute::FocusStyle)
            .map(|x| x.unwrap_style());
        let p: Paragraph = Paragraph::new(selected_text)
            .style(if focus {
                borders.style()
            } else {
                inactive_style.unwrap_or_default()
            })
            .block(block);
        render.render_widget(p, chunks[0]);
        // Render the list of elements in chunks [1]
        // Make list
        let mut list = List::new(choices)
            .block(
                Block::default()
                    .borders(BorderSides::LEFT | BorderSides::BOTTOM | BorderSides::RIGHT)
                    .border_style(if focus {
                        borders.style()
                    } else {
                        Style::default()
                    })
                    .border_type(borders.modifiers)
                    .style(Style::default().bg(background)),
            )
            .direction(tuirealm::ratatui::widgets::ListDirection::TopToBottom)
            .style(Style::default().fg(foreground).bg(background))
            .highlight_style(
                Style::default()
                    .fg(hg)
                    .add_modifier(TextModifiers::REVERSED),
            );
        // Highlighted symbol
        self.hg_str = self
            .props
            .get(Attribute::HighlightedStr)
            .map(|x| x.unwrap_string());
        if let Some(hg_str) = &self.hg_str {
            list = list.highlight_symbol(hg_str);
        }
        let mut state: ListState = ListState::default();
        state.select(Some(self.states.selected));
        render.render_stateful_widget(list, chunks[1], &mut state);
    }

    /// ### render_closed_tab
    ///
    /// Render component when tab is closed
    fn render_closed_tab(&self, render: &mut Frame, area: Rect) {
        let foreground = self
            .props
            .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let background = self
            .props
            .get_or(Attribute::Background, AttrValue::Color(Color::Reset))
            .unwrap_color();
        let inactive_style = self
            .props
            .get(Attribute::FocusStyle)
            .map(|x| x.unwrap_style());
        let focus = self
            .props
            .get_or(Attribute::Focus, AttrValue::Flag(false))
            .unwrap_flag();
        let style = if focus {
            Style::default().bg(background).fg(foreground)
        } else {
            inactive_style.unwrap_or_default()
        };
        let borders = self
            .props
            .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
            .unwrap_borders();
        let borders_style = if focus {
            borders.style()
        } else {
            inactive_style.unwrap_or_default()
        };
        let block: Block = Block::default()
            .borders(BorderSides::ALL)
            .border_style(borders_style)
            .border_type(borders.modifiers)
            .style(style);
        let title = self.props.get(Attribute::Title).map(|x| x.unwrap_title());
        let block = match title {
            Some((text, alignment)) => block.title(text).title_alignment(alignment),
            None => block,
        };
        let selected_text: String = match self.states.choices.get(self.states.selected) {
            None => String::default(),
            Some(s) => s.clone(),
        };
        let p: Paragraph = Paragraph::new(selected_text).style(style).block(block);
        render.render_widget(p, area);
    }

    fn rewindable(&self) -> bool {
        self.props
            .get_or(Attribute::Rewind, AttrValue::Flag(false))
            .unwrap_flag()
    }
}

impl MockComponent for Select {
    fn view(&mut self, render: &mut Frame, area: Rect) {
        if self.props.get_or(Attribute::Display, AttrValue::Flag(true)) == AttrValue::Flag(true) {
            if self.states.is_tab_open() {
                self.render_open_tab(render, area);
            } else {
                self.render_closed_tab(render, area);
            }
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        match attr {
            Attribute::Content => {
                // Reset choices
                let choices: Vec<String> = value
                    .unwrap_payload()
                    .unwrap_vec()
                    .iter()
                    .map(|x| x.clone().unwrap_str())
                    .collect();
                self.states.set_choices(&choices);
            }
            Attribute::Value => {
                self.states
                    .select(value.unwrap_payload().unwrap_one().unwrap_usize());
            }
            Attribute::Focus if self.states.is_tab_open() => {
                if let AttrValue::Flag(false) = value {
                    self.states.cancel_tab();
                }
                self.props.set(attr, value);
            }
            attr => {
                self.props.set(attr, value);
            }
        }
    }

    fn state(&self) -> State {
        if self.states.is_tab_open() {
            State::None
        } else {
            State::One(StateValue::Usize(self.states.selected))
        }
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Down) => {
                // Increment choice
                self.states.next_choice(self.rewindable());
                // Return CmdResult On Change or None if tab is closed
                if self.states.is_tab_open() {
                    CmdResult::Changed(State::One(StateValue::Usize(self.states.selected)))
                } else {
                    CmdResult::None
                }
            }
            Cmd::Move(Direction::Up) => {
                // Increment choice
                self.states.prev_choice(self.rewindable());
                // Return CmdResult On Change or None if tab is closed
                if self.states.is_tab_open() {
                    CmdResult::Changed(State::One(StateValue::Usize(self.states.selected)))
                } else {
                    CmdResult::None
                }
            }
            Cmd::Cancel => {
                self.states.cancel_tab();
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => {
                // Open or close tab
                if self.states.is_tab_open() {
                    self.states.close_tab();
                    CmdResult::Submit(self.state())
                } else {
                    self.states.open_tab();
                    CmdResult::None
                }
            }
            _ => CmdResult::None,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use pretty_assertions::assert_eq;

    use tuirealm::props::{PropPayload, PropValue};

    #[test]
    fn test_components_select_states() {
        let mut states: SelectStates = SelectStates::default();
        assert_eq!(states.selected, 0);
        assert_eq!(states.choices.len(), 0);
        assert_eq!(states.tab_open, false);
        let choices: &[String] = &[
            "lemon".to_string(),
            "strawberry".to_string(),
            "vanilla".to_string(),
            "chocolate".to_string(),
        ];
        states.set_choices(choices);
        assert_eq!(states.selected, 0);
        assert_eq!(states.choices.len(), 4);
        // Move
        states.prev_choice(false);
        assert_eq!(states.selected, 0);
        states.next_choice(false);
        // Tab is closed!!!
        assert_eq!(states.selected, 0);
        states.open_tab();
        assert_eq!(states.is_tab_open(), true);
        // Now we can move
        states.next_choice(false);
        assert_eq!(states.selected, 1);
        states.next_choice(false);
        assert_eq!(states.selected, 2);
        // Forward overflow
        states.next_choice(false);
        states.next_choice(false);
        assert_eq!(states.selected, 3);
        states.prev_choice(false);
        assert_eq!(states.selected, 2);
        // Close tab
        states.close_tab();
        assert_eq!(states.is_tab_open(), false);
        states.prev_choice(false);
        assert_eq!(states.selected, 2);
        // Update
        let choices: &[String] = &["lemon".to_string(), "strawberry".to_string()];
        states.set_choices(choices);
        assert_eq!(states.selected, 1); // Move to first index available
        assert_eq!(states.choices.len(), 2);
        let choices = vec![];
        states.set_choices(&choices);
        assert_eq!(states.selected, 0); // Move to first index available
        assert_eq!(states.choices.len(), 0);
        // Rewind
        let choices: &[String] = &[
            "lemon".to_string(),
            "strawberry".to_string(),
            "vanilla".to_string(),
            "chocolate".to_string(),
        ];
        states.set_choices(choices);
        states.open_tab();
        assert_eq!(states.selected, 0);
        states.prev_choice(true);
        assert_eq!(states.selected, 3);
        states.next_choice(true);
        assert_eq!(states.selected, 0);
        states.next_choice(true);
        assert_eq!(states.selected, 1);
        states.prev_choice(true);
        assert_eq!(states.selected, 0);
        // Cancel tab
        states.close_tab();
        states.select(2);
        states.open_tab();
        states.prev_choice(true);
        states.prev_choice(true);
        assert_eq!(states.selected, 0);
        states.cancel_tab();
        assert_eq!(states.selected, 2);
        assert_eq!(states.is_tab_open(), false);
    }

    #[test]
    fn test_components_select() {
        // Make component
        let mut component = Select::default()
            .foreground(Color::Red)
            .background(Color::Black)
            .borders(Borders::default())
            .highlighted_color(Color::Red)
            .highlighted_str(">>")
            .title("C'est oui ou bien c'est non?", Alignment::Center)
            .choices(&["Oui!", "Non", "Peut-être"])
            .value(1)
            .rewind(false);
        assert_eq!(component.states.is_tab_open(), false);
        component.states.open_tab();
        assert_eq!(component.states.is_tab_open(), true);
        component.states.close_tab();
        assert_eq!(component.states.is_tab_open(), false);
        // Update
        component.attr(
            Attribute::Value,
            AttrValue::Payload(PropPayload::One(PropValue::Usize(2))),
        );
        // Get value
        assert_eq!(component.state(), State::One(StateValue::Usize(2)));
        // Open tab
        component.states.open_tab();
        // Events
        // Move cursor
        assert_eq!(
            component.perform(Cmd::Move(Direction::Up)),
            CmdResult::Changed(State::One(StateValue::Usize(1))),
        );
        assert_eq!(
            component.perform(Cmd::Move(Direction::Up)),
            CmdResult::Changed(State::One(StateValue::Usize(0))),
        );
        // Upper boundary
        assert_eq!(
            component.perform(Cmd::Move(Direction::Up)),
            CmdResult::Changed(State::One(StateValue::Usize(0))),
        );
        // Move down
        assert_eq!(
            component.perform(Cmd::Move(Direction::Down)),
            CmdResult::Changed(State::One(StateValue::Usize(1))),
        );
        assert_eq!(
            component.perform(Cmd::Move(Direction::Down)),
            CmdResult::Changed(State::One(StateValue::Usize(2))),
        );
        // Lower boundary
        assert_eq!(
            component.perform(Cmd::Move(Direction::Down)),
            CmdResult::Changed(State::One(StateValue::Usize(2))),
        );
        // Press enter
        assert_eq!(
            component.perform(Cmd::Submit),
            CmdResult::Submit(State::One(StateValue::Usize(2))),
        );
        // Tab should be closed
        assert_eq!(component.states.is_tab_open(), false);
        // Re open
        assert_eq!(component.perform(Cmd::Submit), CmdResult::None);
        assert_eq!(component.states.is_tab_open(), true);
        // Move arrows
        assert_eq!(
            component.perform(Cmd::Submit),
            CmdResult::Submit(State::One(StateValue::Usize(2))),
        );
        assert_eq!(component.states.is_tab_open(), false);
        assert_eq!(
            component.perform(Cmd::Move(Direction::Down)),
            CmdResult::None
        );
        assert_eq!(component.perform(Cmd::Move(Direction::Up)), CmdResult::None);
    }
}
