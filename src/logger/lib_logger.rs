pub use si_trace_print::*;

pub fn warn(info: String) {
    pf2ñ!("{}", info);
}

/// Print out error and exit.
pub fn rupt(info: &str) -> ! {
    pfn!("{}", info);
    panic!();
}

pub fn print(info: &str) {
    pfn!("{}", info);
}
