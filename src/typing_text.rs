//! The text being typed, with a caret.

use std::{collections::HashMap, ops::Range};

use iced::{
    Color, Element, Font, Length, Pixels, Point, Rectangle, Size, Vector,
    advanced::{
        Layout, Widget, layout, mouse, renderer,
        text::{self, Alignment, LineHeight, Paragraph, Shaping, Span, Text, Wrapping},
        widget::{Tree, tree},
    },
    alignment,
};

use crate::{screen::typing::InputResult, words};

const CARET_WIDTH: f32 = 2.0;

#[derive(Clone, Copy)]
pub struct Style {
    pub pending: Color,
    pub correct: Color,
    pub wrong: Color,
    pub caret: Color,
}

pub struct TypingText<'a> {
    text: &'a [char],
    results: &'a [InputResult],
    font: Font,
    size: Pixels,
    line_height: LineHeight,
    line_spacing: f32,
    visible_lines: usize,
    style: Style,
}

impl<'a> TypingText<'a> {
    /// The caret sits after the last entry of `results`.
    pub fn new(text: &'a [char], results: &'a [InputResult], style: Style) -> Self {
        Self {
            text,
            results,
            font: Font::default(),
            size: Pixels(16.0),
            line_height: LineHeight::default(),
            line_spacing: 0.0,
            visible_lines: 3,
            style,
        }
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into();
        self
    }

    pub fn line_spacing(mut self, spacing: f32) -> Self {
        self.line_spacing = spacing;
        self
    }

    pub fn visible_lines(mut self, count: usize) -> Self {
        self.visible_lines = count;
        self
    }

    fn line_height(&self) -> f32 {
        self.line_height.to_absolute(self.size).0
    }

    fn line_text<Content>(&self, content: Content) -> Text<Content, Font> {
        Text {
            content,
            bounds: Size::INFINITE,
            size: self.size,
            line_height: self.line_height,
            font: self.font,
            align_x: Alignment::Left,
            align_y: alignment::Vertical::Top,
            shaping: Shaping::Advanced,
            wrapping: Wrapping::None,
        }
    }

    /// The typed results falling inside `range`.
    fn results_in(&self, range: &Range<usize>) -> &[InputResult] {
        let end = range.end.min(self.results.len());
        self.results.get(range.start..end).unwrap_or_default()
    }

    /// Splits a line into spans, merging neighboring chars of the same color.
    fn spans(&self, range: &Range<usize>) -> Vec<Span<'static, (), Font>> {
        let mut spans: Vec<(String, Color)> = vec![];

        for i in range.clone() {
            let (color, ch) = match (self.results.get(i), self.text[i]) {
                (Some(InputResult::Correct), ch) => (self.style.correct, ch),
                (Some(InputResult::Wrong), ' ') => (self.style.wrong, '_'),
                (Some(InputResult::Wrong), ch) => (self.style.wrong, ch),
                (None, ch) => (self.style.pending, ch),
            };
            match spans.last_mut() {
                Some((run, run_color)) if *run_color == color => run.push(ch),
                _ => spans.push((ch.to_string(), color)),
            }
        }

        spans
            .into_iter()
            .map(|(run, color)| Span::new(run).color(color))
            .collect()
    }
}

/// Layout cached across frames: rewrapping and reshaping only happen when their inputs change.
struct State<P> {
    font: Font,
    size: Pixels,
    max_width: f32,
    /// Measured width of each word seen so far, trailing space included.
    word_widths: HashMap<String, f32>,
    /// The text `lines` was wrapped from.
    text: Vec<char>,
    lines: Vec<Range<usize>>,
    visible: Vec<VisibleLine<P>>,
    /// Top-left of the caret, relative to the widget.
    caret: Point,
}

struct VisibleLine<P> {
    range: Range<usize>,
    results: Vec<InputResult>,
    paragraph: P,
}

impl<P: Paragraph<Font = Font>> State<P> {
    fn new() -> Self {
        Self {
            font: Font::default(),
            size: Pixels(0.0),
            max_width: 0.0,
            word_widths: HashMap::new(),
            text: vec![],
            lines: vec![],
            visible: vec![],
            caret: Point::ORIGIN,
        }
    }

    /// Drops everything measured with a different font, size or width.
    fn invalidate(&mut self, widget: &TypingText<'_>, max_width: f32) {
        if self.font != widget.font || self.size != widget.size {
            self.font = widget.font;
            self.size = widget.size;
            self.word_widths.clear();
            self.visible.clear();
            self.text.clear();
        }
        if self.max_width != max_width {
            self.max_width = max_width;
            self.text.clear();
        }
    }

    fn rewrap(&mut self, widget: &TypingText<'_>) {
        if self.text == widget.text {
            return;
        }

        let word_widths = &mut self.word_widths;
        self.lines = words::wrap_lines(widget.text, self.max_width, |word| {
            let word: String = word.iter().collect();
            if let Some(&width) = word_widths.get(&word) {
                return width;
            }
            let width = measure::<P>(widget, &word);
            word_widths.insert(word, width);
            width
        });
        self.text = widget.text.to_vec();
    }

    fn reshape_visible(&mut self, widget: &TypingText<'_>) {
        let cursor = widget.results.len();
        let visible = words::visible_lines(&self.lines, cursor, widget.visible_lines);

        self.visible.truncate(visible.len());
        for (i, range) in visible.iter().enumerate() {
            let results = widget.results_in(range);
            let unchanged = self
                .visible
                .get(i)
                .is_some_and(|line| line.range == *range && line.results == results);
            if unchanged {
                continue;
            }

            let spans = widget.spans(range);
            let line = VisibleLine {
                range: range.clone(),
                results: results.to_vec(),
                paragraph: P::with_spans(widget.line_text(spans.as_slice())),
            };
            if i < self.visible.len() {
                self.visible[i] = line;
            } else {
                self.visible.push(line);
            }
        }
    }

    fn place_caret(&mut self, widget: &TypingText<'_>) {
        let cursor = widget.results.len();
        let row = self
            .visible
            .iter()
            .position(|line| line.range.contains(&cursor))
            .unwrap_or(self.visible.len().saturating_sub(1));
        let Some(line) = self.visible.get(row) else {
            self.caret = Point::ORIGIN;
            return;
        };

        let column = cursor.saturating_sub(line.range.start);
        let x = line
            .paragraph
            .grapheme_position(0, column)
            .map_or(0.0, |position| position.x);
        let y = row as f32 * (widget.line_height() + widget.line_spacing);
        self.caret = Point::new(x, y);
    }
}

/// Width of `word` as shaped in the widget's font, trailing whitespace included.
fn measure<P: Paragraph<Font = Font>>(widget: &TypingText<'_>, word: &str) -> f32 {
    P::with_text(widget.line_text(word))
        .grapheme_position(0, word.chars().count())
        .map_or(0.0, |position| position.x)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for TypingText<'_>
where
    Renderer: text::Renderer<Font = Font>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let width = limits.width(Length::Fill).max().width;
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        state.invalidate(self, width);
        state.rewrap(self);
        state.reshape_visible(self);
        state.place_caret(self);

        // Always reserve every line, so the layout doesn't shift near the end of the text.
        let lines = self.visible_lines as f32;
        let height = lines * self.line_height() + (lines - 1.0).max(0.0) * self.line_spacing;
        layout::Node::new(Size::new(width, height))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if !bounds.intersects(viewport) {
            return;
        }
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();

        let line_step = self.line_height() + self.line_spacing;
        for (row, line) in state.visible.iter().enumerate() {
            let position = bounds.position() + Vector::new(0.0, row as f32 * line_step);
            renderer.fill_paragraph(&line.paragraph, position, self.style.pending, *viewport);
        }

        let caret_height = self.size.0;
        let caret = Rectangle::new(
            Point::new(
                bounds.x + state.caret.x - CARET_WIDTH / 2.0,
                bounds.y + state.caret.y + (self.line_height() - caret_height) / 2.0,
            ),
            Size::new(CARET_WIDTH, caret_height),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: caret,
                ..Default::default()
            },
            self.style.caret,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<TypingText<'a>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: text::Renderer<Font = Font> + 'a,
{
    fn from(widget: TypingText<'a>) -> Self {
        Element::new(widget)
    }
}
