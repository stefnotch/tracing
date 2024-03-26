#[cfg(feature = "ansi")]
use owo_colors::{Style as AnsiStyle, Styled};
use tracing_core::Level;

pub(crate) trait StylePainter {
    fn paint<T>(&self, d: T) -> Styled<T>;
}

#[cfg(feature = "ansi")]
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Style {
    is_ansi: bool,
    inner: AnsiStyle,
}

impl Style {
    pub(crate) fn new(is_ansi: bool) -> Self {
        Style {
            is_ansi,
            inner: AnsiStyle::new(),
        }
    }

    pub(crate) fn with_ansi(self, is_ansi: bool) -> Self {
        Style { is_ansi, ..self }
    }

    pub(crate) fn is_ansi(&self) -> bool {
        self.is_ansi
    }

    pub(crate) fn bold(self) -> Self {
        Style {
            is_ansi: self.is_ansi,
            inner: self.inner.bold(),
        }
    }

    pub(crate) fn dimmed(self) -> Self {
        Style {
            is_ansi: self.is_ansi,
            inner: self.inner.dimmed(),
        }
    }

    pub(crate) fn italic(self) -> Self {
        Style {
            is_ansi: self.is_ansi,
            inner: self.inner.italic(),
        }
    }

    pub(crate) fn level_color(self, level: &Level) -> Self {
        let inner = match *level {
            Level::TRACE => self.inner.purple(),
            Level::DEBUG => self.inner.blue(),
            Level::INFO => self.inner.green(),
            Level::WARN => self.inner.yellow(),
            Level::ERROR => self.inner.red(),
        };
        Style {
            is_ansi: true,
            inner,
        }
    }
}

#[cfg(feature = "ansi")]
impl StylePainter for Style {
    fn paint<T>(&self, d: T) -> Styled<T> {
        if self.is_ansi {
            self.inner.style(d)
        } else {
            AnsiStyle::new().style(d)
        }
    }
}

#[cfg(not(feature = "ansi"))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Style;

#[cfg(not(feature = "ansi"))]
impl Style {
    pub fn new(_is_ansi: bool) -> Self {
        Style
    }

    pub fn with_ansi(self, is_ansi: bool) -> Self {
        self
    }

    pub fn is_ansi(&self) -> bool {
        false
    }

    pub fn bold(self) -> Self {
        self
    }

    pub fn dimmed(self) -> Self {
        self
    }

    pub fn italic(self) -> Self {
        self
    }
}
#[cfg(not(feature = "ansi"))]
impl StylePainter for Style {
    fn paint<T>(&self, d: T) -> Styled<T> {
        Styled { target: d }
    }
}

#[cfg(not(feature = "ansi"))]
pub(crate) struct Styled<T> {
    target: T,
}

#[cfg(not(feature = "ansi"))]
macro_rules! impl_fmt {
    ($($trait:path),* $(,)?) => {
        $(
            impl<T: $trait> $trait for Styled<T> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    <T as $trait>::fmt(&self.target, f)
                }
            }
        )*
    };
}

#[cfg(not(feature = "ansi"))]
impl_fmt! {
    fmt::Display,
    fmt::Debug,
    fmt::UpperHex,
    fmt::LowerHex,
    fmt::Binary,
    fmt::UpperExp,
    fmt::LowerExp,
    fmt::Octal,
    fmt::Pointer,
}
