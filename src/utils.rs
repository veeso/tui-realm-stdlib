//! ## Utils
//!
//! `Utilities functions to work with components

// deps
extern crate textwrap;
extern crate unicode_width;
// local
use tuirealm::props::{Alignment, AttrValue, Attribute, Borders, TextModifiers, TextSpan};
use tuirealm::Props;
// ext
use tuirealm::ratatui::style::{Color, Modifier, Style};
use tuirealm::ratatui::text::Line as Spans;
use tuirealm::ratatui::text::Span;
use tuirealm::ratatui::widgets::Block;
use unicode_width::UnicodeWidthStr;

/// ### wrap_spans
///
/// Given a vector of `TextSpans`, it creates a list of `Spans` which mustn't exceed the provided width parameter.
/// Each `Spans` in the returned `Vec` is a line in the text.
pub fn wrap_spans<'a>(spans: &[&TextSpan], width: usize, props: &Props) -> Vec<Spans<'a>> {
    // Prepare result (capacity will be at least spans.len)
    let mut res: Vec<Spans> = Vec::with_capacity(spans.len());
    // Prepare environment
    let mut line_width: usize = 0; // Incremental line width; mustn't exceed `width`.
    let mut line_spans: Vec<Span> = Vec::new(); // Current line; when done, push to res and re-initialize
    for span in spans.iter() {
        // Get styles
        let (fg, bg, tmod) = use_or_default_styles(props, span);
        // Check if width would exceed...
        if line_width + span.content.width() > width {
            // Check if entire line is wider than the area
            if span.content.width() > width {
                // Wrap
                let span_lines = textwrap::wrap(span.content.as_str(), width);
                // iter lines
                for span_line in span_lines.iter() {
                    // Check if width would exceed...
                    if line_width + span_line.width() > width {
                        // New line
                        res.push(Spans::from(line_spans));
                        line_width = 0;
                        line_spans = Vec::new();
                    }
                    // Increment line width
                    line_width += span_line.width();
                    // Push to line
                    line_spans.push(Span::styled(
                        span_line.to_string(),
                        Style::default().fg(fg).bg(bg).add_modifier(tmod),
                    ));
                }
                // Go to next iteration
                continue;
            } else {
                // Just initialize a new line
                res.push(Spans::from(line_spans));
                line_width = 0;
                line_spans = Vec::new();
            }
        }
        // Push span to line
        line_width += span.content.width();
        line_spans.push(Span::styled(
            span.content.to_string(),
            Style::default().fg(fg).bg(bg).add_modifier(tmod),
        ));
    }
    // if there are still elements in spans, push to result
    if !line_spans.is_empty() {
        res.push(Spans::from(line_spans));
    }
    // return res
    res
}

/// ### use_or_default_styles
///
/// Returns the styles to be used; in case in span are default, use props'.
/// The values returned are `(foreground, background, modifiers)`
pub fn use_or_default_styles(props: &Props, span: &TextSpan) -> (Color, Color, Modifier) {
    (
        match span.fg {
            Color::Reset => props
                .get_or(Attribute::Foreground, AttrValue::Color(Color::Reset))
                .unwrap_color(),
            _ => span.fg,
        },
        match span.bg {
            Color::Reset => props
                .get_or(Attribute::Background, AttrValue::Color(Color::Reset))
                .unwrap_color(),
            _ => span.bg,
        },
        match span.modifiers.is_empty() {
            true => props
                .get_or(
                    Attribute::TextProps,
                    AttrValue::TextModifiers(TextModifiers::empty()),
                )
                .unwrap_text_modifiers(),
            false => span.modifiers,
        },
    )
}

/// ### get_block
///
/// Construct a block for widget using block properties.
/// If focus is true the border color is applied, otherwise inactive_style
pub fn get_block<'a>(
    props: Borders,
    title: Option<(String, Alignment)>,
    focus: bool,
    inactive_style: Option<Style>,
) -> Block<'a> {
    let title = title.unwrap_or((String::default(), Alignment::Left));
    Block::default()
        .borders(props.sides)
        .border_style(match focus {
            true => props.style(),
            false => {
                inactive_style.unwrap_or_else(|| Style::default().fg(Color::Reset).bg(Color::Reset))
            }
        })
        .border_type(props.modifiers)
        .title(title.0)
        .title_alignment(title.1)
}

/// ### calc_utf8_cursor_position
///
/// Calculate the UTF8 compliant position for the cursor given the characters preceeding the cursor position.
/// Use this function to calculate cursor position whenever you want to handle UTF8 texts with cursors
pub fn calc_utf8_cursor_position(chars: &[char]) -> u16 {
    chars.iter().collect::<String>().width() as u16
}

#[cfg(test)]
mod test {

    use super::*;
    use tuirealm::props::{Alignment, BorderSides, BorderType, Props};

    use pretty_assertions::assert_eq;

    #[test]
    fn test_components_utils_wrap_spans() {
        let mut props: Props = Props::default();
        props.set(
            Attribute::TextProps,
            AttrValue::TextModifiers(TextModifiers::BOLD),
        );
        props.set(Attribute::Foreground, AttrValue::Color(Color::Red));
        props.set(Attribute::Background, AttrValue::Color(Color::White));
        // Prepare spans; let's start with two simple spans, which fits the line
        let spans: Vec<TextSpan> = vec![TextSpan::from("hello, "), TextSpan::from("world!")];
        let spans: Vec<&TextSpan> = spans.iter().collect();
        assert_eq!(wrap_spans(&spans, 64, &props).len(), 1);
        // Let's make a sentence, which would require two lines
        let spans: Vec<TextSpan> = vec![
            TextSpan::from("Hello, everybody, I'm Uncle Camel!"),
            TextSpan::from("How's it going today?"),
        ];
        let spans: Vec<&TextSpan> = spans.iter().collect();
        assert_eq!(wrap_spans(&spans, 32, &props).len(), 2);
        // Let's make a sentence, which requires 3 lines, but with only one span
        let spans: Vec<TextSpan> = vec![TextSpan::from(
            "Hello everybody! My name is Uncle Camel. How's it going today?",
        )];
        let spans: Vec<&TextSpan> = spans.iter().collect();
        // makes Hello everybody, my name is uncle, camel. how's it, goind today
        assert_eq!(wrap_spans(&spans, 16, &props).len(), 4);
        // Combine
        let spans: Vec<TextSpan> = vec![
            TextSpan::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit."),
            TextSpan::from("Canem!"),
            TextSpan::from("In posuere sollicitudin vulputate"),
            TextSpan::from("Sed vitae rutrum quam."),
        ];
        let spans: Vec<&TextSpan> = spans.iter().collect();
        // "Lorem ipsum dolor sit amet,", "consectetur adipiscing elit. Canem!", "In posuere sollicitudin vulputate", "Sed vitae rutrum quam."
        assert_eq!(wrap_spans(&spans, 36, &props).len(), 4);
    }

    #[test]
    fn test_components_utils_use_or_default_styles() {
        let mut props: Props = Props::default();
        props.set(
            Attribute::TextProps,
            AttrValue::TextModifiers(TextModifiers::BOLD),
        );
        props.set(Attribute::Foreground, AttrValue::Color(Color::Red));
        props.set(Attribute::Background, AttrValue::Color(Color::White));
        let span: TextSpan = TextSpan::from("test")
            .underlined()
            .fg(Color::Yellow)
            .bg(Color::Cyan);
        // Not-default
        let (fg, bg, modifiers) = use_or_default_styles(&props, &span);
        assert_eq!(fg, Color::Yellow);
        assert_eq!(bg, Color::Cyan);
        assert!(modifiers.intersects(Modifier::UNDERLINED));
        // Default
        let span: TextSpan = TextSpan::from("test");
        let (fg, bg, modifiers) = use_or_default_styles(&props, &span);
        assert_eq!(fg, Color::Red);
        assert_eq!(bg, Color::White);
        assert!(modifiers.intersects(Modifier::BOLD));
    }

    #[test]
    fn test_components_utils_get_block() {
        let props = Borders::default()
            .sides(BorderSides::ALL)
            .color(Color::Red)
            .modifiers(BorderType::Rounded);
        get_block(
            props.clone(),
            Some(("title".to_string(), Alignment::Center)),
            true,
            None,
        );
        get_block(props, None, false, None);
    }

    #[test]
    fn test_components_utils_calc_utf8_cursor_position() {
        let chars: Vec<char> = vec!['v', 'e', 'e', 's', 'o'];
        // Entire
        assert_eq!(calc_utf8_cursor_position(chars.as_slice()), 5);
        assert_eq!(calc_utf8_cursor_position(&chars[0..3]), 3);
        // With special characters
        let chars: Vec<char> = vec!['я', ' ', 'х', 'о', 'ч', 'у', ' ', 'с', 'п', 'а', 'т', 'ь'];
        assert_eq!(calc_utf8_cursor_position(&chars[0..6]), 6);
        let chars: Vec<char> = vec!['H', 'i', '😄'];
        assert_eq!(calc_utf8_cursor_position(chars.as_slice()), 4);
        let chars: Vec<char> = vec!['我', '之', '😄'];
        assert_eq!(calc_utf8_cursor_position(chars.as_slice()), 6);
    }
}
