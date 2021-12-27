use crate::flush;
use crate::prelude::crash;
use std::panic::{self, PanicInfo};

pub fn setup_panic_logger() {
    panic::set_hook(Box::new(move |pi: &PanicInfo<'_>| {
        handle_panic(pi);
    }));
}

fn handle_panic(panic_info: &PanicInfo<'_>) {
    let details = format!("{}", panic_info);
    crash!("{}", details);
    flush();
}
