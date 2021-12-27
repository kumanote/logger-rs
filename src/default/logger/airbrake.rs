use crate::{
    filter::{Filter, LevelFilter},
    formatter::{AirbrakeFormatter, AsyncFormatter, Formatter},
    writer::HttpWriter,
    AsyncEvent, AsyncLogger, Event, Logger, Metadata, StandardFilter, StandardFilterBuilder,
    Writer,
};

pub struct AirbrakeLogger {
    printer: HttpWriter,
    filter: StandardFilter,
    formatter: AirbrakeFormatter,
}

impl AirbrakeLogger {
    pub fn new(endpoint: String, environment: Option<String>) -> Self {
        let printer = HttpWriter::new(endpoint);
        let filter = StandardFilterBuilder::new()
            .filter_level(LevelFilter::Error)
            .build();
        let formatter = AirbrakeFormatter::new(environment);
        Self {
            printer,
            filter,
            formatter,
        }
    }
}

impl Logger for AirbrakeLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.filter.enabled(metadata)
    }

    fn record(&self, event: &Event) {
        let s = Formatter::format(&self.formatter, event).expect("Unable to format");
        self.printer.write(s);
    }

    fn flush(&self) {}
}

impl AsyncLogger for AirbrakeLogger {
    fn record(&self, event: &AsyncEvent) {
        let s = AsyncFormatter::format(&self.formatter, event).expect("Unable to format");
        self.printer.write(s);
    }
}
