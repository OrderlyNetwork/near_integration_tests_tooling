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
