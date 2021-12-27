use crate::{
    filter::Filter,
    formatter::{AsyncFormatter, Formatter, StandardFormatter},
    writer::{FileWriter, Writer},
    AsyncEvent, AsyncLogger, Event, Logger, Metadata, StandardFilter,
};

pub struct LocalFileLogger {
    pub(crate) printer: FileWriter,
    pub(crate) filter: StandardFilter,
    pub(crate) formatter: StandardFormatter,
}

impl Logger for LocalFileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.filter.enabled(metadata)
    }

    fn record(&self, event: &Event) {
        let s = Formatter::format(&self.formatter, event).expect("Unable to format");
        self.printer.write(s);
    }

    fn flush(&self) {}
}

impl AsyncLogger for LocalFileLogger {
    fn record(&self, event: &AsyncEvent) {
        let s = AsyncFormatter::format(&self.formatter, event).expect("Unable to format");
        self.printer.write(s);
    }
}
