use near_sdk::serde_json;
use owo_colors::OwoColorize;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};
use workspaces::result::{ExecutionFinalResult, ExecutionResult, Value, ViewResultDetails};

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

pub async fn create_view_result_logger(
    res: ViewResultDetails,
) -> anyhow::Result<ViewResultDetails> {
    if !res.logs.is_empty() {
        for log in res.logs.iter() {
            print_log!("{}", log.bright_yellow());
        }
    }
    Ok(res)
}

pub async fn create_transact_result_logger(
    ident: Option<&str>,
    res: ExecutionFinalResult,
) -> anyhow::Result<ExecutionResult<Value>> {
    for failure in res.receipt_failures() {
        print_log!("{:#?}", failure.bright_red());
    }
    for outcome in res.receipt_outcomes() {
        if !outcome.logs.is_empty() {
            for log in outcome.logs.iter() {
                if log.starts_with("EVENT_JSON:") {
                    let event: EventLogData =
                        serde_json::from_str(&log.replace("EVENT_JSON:", ""))?;
                    let event: Event = event.into();
                    print_log!(
                        "{}: {}\n    {}",
                        "account".bright_cyan(),
                        outcome.executor_id,
                        event
                    );
                } else {
                    print_log!("{}", log.bright_yellow());
                }
            }
        }
    }
    if let Some(ident) = ident {
        print_log!(
            "{} gas burnt: {:.3} {}",
            ident.italic(),
            (res.total_gas_burnt as f64 / 1_000_000_000_000.)
                .bright_magenta()
                .bold(),
            "TGas".bright_magenta().bold()
        );
    }
    Ok(res.into_result()?)
}

pub enum Event {
    FtTransfer(FtTransferEvent),
    Unknown(EventLogData),
}

impl Display for Event {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Event::FtTransfer(event) => formatter.write_fmt(format_args!("{event}")),
            Event::Unknown(event) => formatter.write_fmt(format_args!("{event}")),
        }
    }
}

pub struct FtTransferEvent {
    standard: String,
    version: String,
    event: String,
    data: FtTransferData,
}

impl Display for FtTransferEvent {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{}: {}", "event".bright_cyan(), self.event))?;
        formatter.write_fmt(format_args!(
            "\n    {}: {}",
            "standard".bright_cyan(),
            self.standard
        ))?;
        formatter.write_fmt(format_args!(
            "\n    {}: {}",
            "version".bright_cyan(),
            self.version
        ))?;
        formatter.write_fmt(format_args!(
            "\n    {}: {}",
            "data".bright_cyan(),
            self.data
        ))?;
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct FtTransferData {
    old_owner_id: String,
    new_owner_id: String,
    amount: String,
    memo: Option<String>,
}

impl Display for FtTransferData {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        if let Some(memo) = &self.memo {
            formatter.write_fmt(format_args!(
                "{} --> {} ({}) --> {}",
                self.old_owner_id.bright_blue(),
                self.amount.bright_blue(),
                memo,
                self.new_owner_id.bright_blue(),
            ))?;
        } else {
            formatter.write_fmt(format_args!(
                "{} --> {} --> {}",
                self.old_owner_id.bright_blue(),
                self.amount.bright_blue(),
                self.new_owner_id.bright_blue(),
            ))?;
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct EventLogData {
    standard: String,
    version: String,
    event: String,
    data: Option<Vec<HashMap<String, String>>>,
}

impl Display for EventLogData {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("{}: {}", "event".bright_cyan(), self.event))?;
        formatter.write_fmt(format_args!(
            "\n    {}: {}",
            "standard".bright_cyan(),
            self.standard
        ))?;
        formatter.write_fmt(format_args!(
            "\n    {}: {}",
            "version".bright_cyan(),
            self.version
        ))?;
        if let Some(data) = &self.data {
            formatter.write_fmt(format_args!("\n    {}: {:?}", "data".bright_cyan(), data))?;
        }
        Ok(())
    }
}

impl From<EventLogData> for Event {
    fn from(event_data: EventLogData) -> Self {
        match event_data.event.as_ref() {
            "ft_transfer" => {
                if let Some(data) = event_data
                    .data
                    .clone()
                    .and_then(|data| data.get(0).cloned())
                {
                    if let (Some(old_owner_id), Some(new_owner_id), Some(amount), memo) = (
                        data.get("old_owner_id").cloned(),
                        data.get("new_owner_id").cloned(),
                        data.get("amount").cloned(),
                        data.get("memo").cloned(),
                    ) {
                        return Event::FtTransfer(FtTransferEvent {
                            standard: event_data.standard,
                            version: event_data.version,
                            event: event_data.event,
                            data: FtTransferData {
                                old_owner_id,
                                new_owner_id,
                                amount,
                                memo,
                            },
                        });
                    }
                }
                Event::Unknown(event_data)
            }
            _ => Event::Unknown(event_data),
        }
    }
}
