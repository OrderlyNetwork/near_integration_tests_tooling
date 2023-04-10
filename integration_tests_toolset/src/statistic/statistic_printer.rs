use super::mode_printer::ModePrinter;
use crate::error::TestError;

/// Interface for printing the statistic.
/// Also it provides possibility to structure stored statistic data with the make_report method
pub trait StatisticPrinter {
    fn get_printer_mode(&self) -> &ModePrinter;

    fn make_report(&self) -> String;

    fn print_statistic(&self) -> Result<(), TestError> {
        let result = self.make_report();
        let printer_mode = self.get_printer_mode();
        printer_mode.print(result.as_bytes())
    }
}
