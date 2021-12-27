use crate::{
    filter::Filter,
    formatter::{AsyncFormatter, Formatter, JsonFormatter},
    writer::TcpWriter,
    AsyncEvent, AsyncLogger, Event, Logger, Metadata, StandardFilter, Writer,
};

pub struct TcpLogger {
    pub(crate) printer: TcpWriter,
    pub(crate) filter: StandardFilter,
    pub(crate) formatter: JsonFormatter,
}

impl Logger for TcpLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.filter.enabled(metadata)
    }

    fn record(&self, event: &Event) {
        let s = Formatter::format(&self.formatter, event).expect("Unable to format");
        self.printer.write(s);
    }

    fn flush(&self) {
        self.printer.flush()
    }
}

impl AsyncLogger for TcpLogger {
    fn record(&self, event: &AsyncEvent) {
        let s = AsyncFormatter::format(&self.formatter, event).expect("Unable to format");
        self.printer.write(s);
    }
}
