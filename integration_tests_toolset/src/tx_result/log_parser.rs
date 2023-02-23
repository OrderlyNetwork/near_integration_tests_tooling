use crate::tx_result::{CallResult, TxResult, TxResultDetails, ViewResult};

pub trait LogParser {
    fn logs(&self) -> Vec<String>;
    fn events_from_logs<E>(&self) -> Vec<E>
    where
        E: for<'a> serde::Deserialize<'a>;

    fn check_for_event<E>(self, event: E) -> Self
    where
        E: for<'a> serde::Deserialize<'a> + PartialEq + std::fmt::Debug,
        Self: Sized,
    {
        self.check_for_events(&[event])
    }

    fn check_for_events<E>(self, events: &[E]) -> Self
    where
        E: for<'a> serde::Deserialize<'a> + PartialEq + std::fmt::Debug,
        Self: Sized,
    {
        let log_events = self.events_from_logs::<E>();
        for event in events {
            assert!(log_events.contains(event), "Event {:?} not found", event);
        }
        self
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
