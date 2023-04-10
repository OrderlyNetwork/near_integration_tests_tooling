use crate::tx_result::{CallResult, Result, TxResult, TxResultDetails, ViewResult};

/// This interface is useful for extracting emitted by the smart contract method events
pub trait LogParser {
    /// Extract all emitted events in raw format returning bunch of Strings
    fn logs(&self) -> Vec<String>;

    /// Extract all events of the same type
    fn events_from_logs<E>(&self) -> Vec<E>
    where
        E: for<'a> serde::Deserialize<'a>;

    /// Assert whether the transaction output details contains particular logs
    fn assert_event<E>(self, event: E) -> Self
    where
        E: for<'a> serde::Deserialize<'a> + PartialEq + Eq + std::fmt::Debug,
        Self: Sized,
    {
        self.assert_events(&[event])
    }

    /// Assert whether the transaction output details contains a bunch of logs of particular type
    fn assert_events<E>(self, events: &[E]) -> Self
    where
        E: for<'a> serde::Deserialize<'a> + PartialEq + Eq + std::fmt::Debug,
        Self: Sized,
    {
        self.check_events(events).unwrap();
        self
    }

    /// Check for the particular event in the transaction
    fn check_event<E>(&self, event: E) -> Result<()>
    where
        E: for<'a> serde::Deserialize<'a> + PartialEq + Eq + std::fmt::Debug,
        Self: Sized,
    {
        self.check_events(&[event])
    }

    /// Check for the bunch of events in particular transaction
    fn check_events<E>(&self, events: &[E]) -> Result<()>
    where
        E: for<'a> serde::Deserialize<'a> + PartialEq + Eq + std::fmt::Debug,
        Self: Sized,
    {
        let mut log_events = self.events_from_logs::<E>();
        for event in events {
            if let Some(pos) = log_events.iter().position(|log_event| log_event == event) {
                log_events.remove(pos);
            } else {
                return Err(crate::error::TestError::Custom(format!(
                    "Event not found: {:?}",
                    event
                )));
            }
        }
        Ok(())
    }
}

impl<T> LogParser for TxResult<T> {
    fn logs(&self) -> Vec<String> {
        match &self.details {
            TxResultDetails::View(ViewResult { logs }) => logs.clone(),
            TxResultDetails::Call(CallResult {
                receipt_outcomes, ..
            }) => receipt_outcomes
                .iter()
                .map(|outcome| outcome.logs.clone())
                .flatten()
                .collect(),
        }
    }

    fn events_from_logs<E>(&self) -> Vec<E>
    where
        E: for<'a> serde::Deserialize<'a>,
    {
        self.logs()
            .iter()
            .filter_map(|log| serde_json::from_str(log).ok())
            .collect()
    }
}
