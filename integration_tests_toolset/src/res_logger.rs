use crate::error::Result;
pub use owo_colors::OwoColorize;
use workspaces::result::{ExecutionFinalResult, ViewResultDetails};

pub trait ResLogger<R> {
    fn check_res_log_failures(&self) -> Result<()>;
}

impl ResLogger<ViewResultDetails> for ViewResultDetails {
    fn check_res_log_failures(&self) -> Result<()> {
        Ok(())
    }
}

#[macro_export]
macro_rules! print_log {
    ( $x:expr, $($y:expr),+ ) => {
        let thread_name = std::thread::current().name().unwrap().to_string();
        if thread_name == "main" {
            println!($x, $($y),+);
        } else {
            println!(
                concat!("{}\n    ", $x),
                thread_name.bold(),
                $($y),+
            );
        }
    };
}

impl ResLogger<ExecutionFinalResult> for ExecutionFinalResult {
    fn check_res_log_failures(&self) -> Result<()> {
        for failure in self.receipt_failures() {
            print_log!("{:#?}", failure.bright_red());
            failure.clone().into_result()?;
        }
        self.clone().into_result()?;
        Ok(())
    }
}
