use super::*;
use crate::{
    field::{VisitFmt, VisitOutput},
    fmt::fmt_subscriber::{FmtContext, FormattedFields},
    registry::LookupSpan,
};

use std::fmt;
use tracing_core::{
    field::{self, Field},
    Collect, Event,
};

#[cfg(feature = "tracing-log")]
use tracing_log::NormalizeEvent;

/// An excessively pretty, human-readable event formatter.
///
/// Unlike the [`Full`], [`Compact`], and [`Json`] formatters, this is a
/// multi-line output format. Each individual event may output multiple lines of
/// text.
///
/// # Example Output
///
/// <pre><font color="#4E9A06"><b>:;</b></font> <font color="#4E9A06">cargo</font> run --example fmt-pretty
/// <font color="#4E9A06"><b>    Finished</b></font> dev [unoptimized + debuginfo] target(s) in 0.08s
/// <font color="#4E9A06"><b>     Running</b></font> `target/debug/examples/fmt-pretty`
///   2022-02-15T18:44:24.535324Z <font color="#4E9A06"> INFO</font> <font color="#4E9A06"><b>fmt_pretty</b></font><font color="#4E9A06">: preparing to shave yaks, </font><font color="#4E9A06"><b>number_of_yaks</b></font><font color="#4E9A06">: 3</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt-pretty.rs:16 <font color="#AAAAAA"><i>on</i></font> main
///
///   2022-02-15T18:44:24.535403Z <font color="#4E9A06"> INFO</font> <font color="#4E9A06"><b>fmt_pretty::yak_shave</b></font><font color="#4E9A06">: shaving yaks</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:41 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535442Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: hello! I&apos;m gonna shave a yak, </font><font color="#75507B"><b>excitement</b></font><font color="#75507B">: &quot;yay!&quot;</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:16 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 1
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535469Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: yak shaved successfully</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:25 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 1
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535502Z <font color="#3465A4">DEBUG</font> <font color="#3465A4"><b>yak_events</b></font><font color="#3465A4">: </font><font color="#3465A4"><b>yak</b></font><font color="#3465A4">: 1, </font><font color="#3465A4"><b>shaved</b></font><font color="#3465A4">: true</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:46 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535524Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: </font><font color="#75507B"><b>yaks_shaved</b></font><font color="#75507B">: 1</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:55 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535551Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: hello! I&apos;m gonna shave a yak, </font><font color="#75507B"><b>excitement</b></font><font color="#75507B">: &quot;yay!&quot;</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:16 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 2
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535573Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: yak shaved successfully</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:25 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 2
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535600Z <font color="#3465A4">DEBUG</font> <font color="#3465A4"><b>yak_events</b></font><font color="#3465A4">: </font><font color="#3465A4"><b>yak</b></font><font color="#3465A4">: 2, </font><font color="#3465A4"><b>shaved</b></font><font color="#3465A4">: true</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:46 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535618Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: </font><font color="#75507B"><b>yaks_shaved</b></font><font color="#75507B">: 2</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:55 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535644Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: hello! I&apos;m gonna shave a yak, </font><font color="#75507B"><b>excitement</b></font><font color="#75507B">: &quot;yay!&quot;</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:16 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 3
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535670Z <font color="#C4A000"> WARN</font> <font color="#C4A000"><b>fmt_pretty::yak_shave</b></font><font color="#C4A000">: could not locate yak</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:18 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shave</b> <font color="#AAAAAA"><i>with</i></font> <b>yak</b>: 3
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535698Z <font color="#3465A4">DEBUG</font> <font color="#3465A4"><b>yak_events</b></font><font color="#3465A4">: </font><font color="#3465A4"><b>yak</b></font><font color="#3465A4">: 3, </font><font color="#3465A4"><b>shaved</b></font><font color="#3465A4">: false</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:46 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535720Z <font color="#CC0000">ERROR</font> <font color="#CC0000"><b>fmt_pretty::yak_shave</b></font><font color="#CC0000">: failed to shave yak, </font><font color="#CC0000"><b>yak</b></font><font color="#CC0000">: 3, </font><font color="#CC0000"><b>error</b></font><font color="#CC0000">: missing yak, </font><font color="#CC0000"><b>error.sources</b></font><font color="#CC0000">: [out of space, out of cash]</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:51 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535742Z <font color="#75507B">TRACE</font> <font color="#75507B"><b>fmt_pretty::yak_shave</b></font><font color="#75507B">: </font><font color="#75507B"><b>yaks_shaved</b></font><font color="#75507B">: 2</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt/yak_shave.rs:55 <font color="#AAAAAA"><i>on</i></font> main
///     <font color="#AAAAAA"><i>in</i></font> fmt_pretty::yak_shave::<b>shaving_yaks</b> <font color="#AAAAAA"><i>with</i></font> <b>yaks</b>: 3
///
///   2022-02-15T18:44:24.535765Z <font color="#4E9A06"> INFO</font> <font color="#4E9A06"><b>fmt_pretty</b></font><font color="#4E9A06">: yak shaving completed, </font><font color="#4E9A06"><b>all_yaks_shaved</b></font><font color="#4E9A06">: false</font>
///     <font color="#AAAAAA"><i>at</i></font> examples/examples/fmt-pretty.rs:19 <font color="#AAAAAA"><i>on</i></font> main
/// </pre>
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pretty {
    display_location: bool,
}

/// The [visitor] produced by [`Pretty`]'s [`MakeVisitor`] implementation.
///
/// [visitor]: field::Visit
/// [`MakeVisitor`]: crate::field::MakeVisitor
#[derive(Debug)]
pub struct PrettyVisitor<'a> {
    writer: Writer<'a>,
    is_empty: bool,
    result: fmt::Result,
}

/// An excessively pretty, human-readable [`MakeVisitor`] implementation.
///
/// [`MakeVisitor`]: crate::field::MakeVisitor
#[derive(Debug)]
pub struct PrettyFields {
    /// A value to override the provided `Writer`'s ANSI formatting
    /// configuration.
    ///
    /// If this is `Some`, we override the `Writer`'s ANSI setting. This is
    /// necessary in order to continue supporting the deprecated
    /// `PrettyFields::with_ansi` method. If it is `None`, we don't override the
    /// ANSI formatting configuration (because the deprecated method was not
    /// called).
    // TODO: when `PrettyFields::with_ansi` is removed, we can get rid
    // of this entirely.
    ansi: Option<bool>,
}

// === impl Pretty ===

impl Default for Pretty {
    fn default() -> Self {
        Self {
            display_location: true,
        }
    }
}

impl Pretty {
    /// Sets whether the event's source code location is displayed.
    ///
    /// This defaults to `true`.
    #[deprecated(
        since = "0.3.6",
        note = "all formatters now support configurable source locations. Use `Format::with_source_location` instead."
    )]
    pub fn with_source_location(self, display_location: bool) -> Self {
        Self {
            display_location,
            ..self
        }
    }
}

impl<C, N, T> FormatEvent<C, N> for Format<Pretty, T>
where
    C: Collect + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
    T: FormatTime,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, C, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        #[cfg(feature = "tracing-log")]
        let normalized_meta = event.normalized_metadata();
        #[cfg(feature = "tracing-log")]
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());
        #[cfg(not(feature = "tracing-log"))]
        let meta = event.metadata();
        write!(&mut writer, "  ")?;

        self.format_timestamp(&mut writer)?;

        let style = if self.display_level {
            writer.style.level_color(meta.level())
        } else {
            writer.style
        };

        if self.display_level {
            self.format_level(*meta.level(), &mut writer)?;
        }

        if self.display_target {
            write!(
                writer,
                "{}{}",
                style.bold().paint(meta.target()),
                style.paint(":")
            )?;
        }
        let line_number = if self.display_line_number {
            meta.line()
        } else {
            None
        };

        // If the file name is disabled, format the line number right after the
        // target. Otherwise, if we also display the file, it'll go on a
        // separate line.
        if let (Some(line_number), false, true) = (
            line_number,
            self.display_filename,
            self.format.display_location,
        ) {
            write!(writer, "{}{}", style.paint(line_number), style.paint(":"))?;
        }

        writer.write_char(' ')?;

        let mut v = PrettyVisitor::new(writer.by_styled_ref(style), true);
        event.record(&mut v);
        v.finish()?;
        writer.write_char('\n')?;

        let dimmed_italic = writer.style.dimmed().italic();
        let thread = self.display_thread_name || self.display_thread_id;

        if let (Some(file), true, true) = (
            meta.file(),
            self.format.display_location,
            self.display_filename,
        ) {
            write!(writer, "    {} {}", dimmed_italic.paint("at"), file,)?;

            if let Some(line) = line_number {
                write!(writer, ":{}", line)?;
            }
            writer.write_char(if thread { ' ' } else { '\n' })?;
        } else if thread {
            write!(writer, "    ")?;
        };

        if thread {
            write!(writer, "{} ", dimmed_italic.paint("on"))?;
            let thread = std::thread::current();
            if self.display_thread_name {
                if let Some(name) = thread.name() {
                    write!(writer, "{}", name)?;
                    if self.display_thread_id {
                        writer.write_char(' ')?;
                    }
                }
            }
            if self.display_thread_id {
                write!(writer, "{:?}", thread.id())?;
            }
            writer.write_char('\n')?;
        }

        let bold = writer.style.bold();
        let span = event
            .parent()
            .and_then(|id| ctx.span(id))
            .or_else(|| ctx.lookup_current());

        let scope = span.into_iter().flat_map(|span| span.scope());

        for span in scope {
            let meta = span.metadata();
            if self.display_target {
                write!(
                    writer,
                    "    {} {}::{}",
                    dimmed_italic.paint("in"),
                    meta.target(),
                    bold.paint(meta.name()),
                )?;
            } else {
                write!(
                    writer,
                    "    {} {}",
                    dimmed_italic.paint("in"),
                    bold.paint(meta.name()),
                )?;
            }

            let ext = span.extensions();
            let fields = &ext
                .get::<FormattedFields<N>>()
                .expect("Unable to find FormattedFields in extensions; this is a bug");
            if !fields.is_empty() {
                write!(writer, " {} {}", dimmed_italic.paint("with"), fields)?;
            }
            writer.write_char('\n')?;
        }

        writer.write_char('\n')
    }
}

impl<'writer> FormatFields<'writer> for Pretty {
    fn format_fields<R: RecordFields>(&self, writer: Writer<'writer>, fields: R) -> fmt::Result {
        let mut v = PrettyVisitor::new(writer, true);
        fields.record(&mut v);
        v.finish()
    }

    fn add_fields(
        &self,
        current: &'writer mut FormattedFields<Self>,
        fields: &span::Record<'_>,
    ) -> fmt::Result {
        let empty = current.is_empty();
        let writer = current.as_writer();
        let mut v = PrettyVisitor::new(writer, empty);
        fields.record(&mut v);
        v.finish()
    }
}

// === impl PrettyFields ===

impl Default for PrettyFields {
    fn default() -> Self {
        Self::new()
    }
}

impl PrettyFields {
    /// Returns a new default [`PrettyFields`] implementation.
    pub fn new() -> Self {
        // By default, don't override the `Writer`'s ANSI colors
        // configuration. We'll only do this if the user calls the
        // deprecated `PrettyFields::with_ansi` method.
        Self { ansi: None }
    }

    /// Enable ANSI encoding for formatted fields.
    #[deprecated(
        since = "0.3.3",
        note = "Use `fmt::Subscriber::with_ansi` or `fmt::Collector::with_ansi` instead."
    )]
    pub fn with_ansi(self, ansi: bool) -> Self {
        Self {
            ansi: Some(ansi),
            ..self
        }
    }
}

impl<'a> MakeVisitor<Writer<'a>> for PrettyFields {
    type Visitor = PrettyVisitor<'a>;

    #[inline]
    fn make_visitor(&self, mut target: Writer<'a>) -> Self::Visitor {
        if let Some(ansi) = self.ansi {
            target = target.with_ansi(ansi);
        }
        PrettyVisitor::new(target, true)
    }
}

// === impl PrettyVisitor ===

impl<'a> PrettyVisitor<'a> {
    /// Returns a new default visitor that formats to the provided `writer`.
    ///
    /// # Arguments
    /// - `writer`: the writer to format to.
    /// - `is_empty`: whether or not any fields have been previously written to
    ///   that writer.
    pub fn new(writer: Writer<'a>, is_empty: bool) -> Self {
        Self {
            writer,
            is_empty,
            result: Ok(()),
        }
    }

    fn style(&self) -> Style {
        self.writer.style
    }

    #[must_use]
    fn write_padding(&mut self) -> fmt::Result {
        if self.is_empty {
            self.is_empty = false;
            Ok(())
        } else {
            write!(self.writer, "{}", self.writer.style.paint(", "))
        }
    }

    #[must_use]
    fn record_debug_impl(&mut self, field: &Field, styled_value: &dyn fmt::Debug) -> fmt::Result {
        let bold = self.style().bold();
        match field.name() {
            "message" => {
                self.write_padding()?;
                write!(self.writer, "{:?}", styled_value)
            }
            // Skip fields that are actually log metadata that have already been handled
            #[cfg(feature = "tracing-log")]
            name if name.starts_with("log.") => Ok(()),
            name if name.starts_with("r#") => {
                self.write_padding()?;
                write!(
                    self.writer,
                    "{}{} {:?}",
                    bold.paint(&name[2..]),
                    bold.paint(":"),
                    styled_value
                )
            }
            name => {
                self.write_padding()?;
                write!(
                    self.writer,
                    "{}{} {:?}",
                    bold.paint(name),
                    bold.paint(":"),
                    styled_value
                )
            }
        }
    }
}

impl<'a> field::Visit for PrettyVisitor<'a> {
    fn record_str(&mut self, field: &Field, value: &str) {
        if self.result.is_err() {
            return;
        }

        self.result = if field.name() == "message" {
            self.record_debug_impl(field, &format_args!("{}", self.style().paint(value)))
        } else {
            self.record_debug_impl(field, &self.style().paint(value))
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if self.result.is_err() {
            return;
        }

        let style = self.style();
        self.result = if let Some(source) = value.source() {
            let bold = style.bold();
            self.record_debug_impl(
                field,
                &format_args!(
                    "{}{} {}{}{} {}",
                    style.paint(value),
                    style.paint(","),
                    bold.paint(field),
                    bold.paint(".sources"),
                    style.paint(":"),
                    style.paint(ErrorSourceList(source))
                ),
            )
        } else {
            self.record_debug_impl(field, &format_args!("{}", style.paint(value)))
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if self.result.is_err() {
            return;
        }
        self.result = self.record_debug_impl(field, &self.style().paint(value));
    }
}

impl<'a> VisitOutput<fmt::Result> for PrettyVisitor<'a> {
    fn finish(self) -> fmt::Result {
        self.result
    }
}

impl<'a> VisitFmt for PrettyVisitor<'a> {
    fn writer(&mut self) -> &mut dyn fmt::Write {
        &mut self.writer
    }
}
