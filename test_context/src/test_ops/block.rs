use crate::context::TestContext;
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use integration_tests_toolset::statistic::statistic_consumer::Statistic;

use super::runnable::Runnable;

// #[cfg(feature = "stress_test")]
#[derive(Debug, Clone, Default)]
pub struct Block<T, const N: usize>
where
    Box<dyn Runnable<T, N>>: Clone,
    T: Clone + std::fmt::Debug,
{
    pub chain: Vec<Box<dyn Runnable<T, N>>>,
    pub concurrent: Vec<Box<dyn Runnable<T, N>>>,
}

impl<T, const N: usize> Block<T, N>
where
    Box<(dyn Runnable<T, N> + 'static)>: Clone,
    T: Clone + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            chain: vec![],
            concurrent: vec![],
        }
    }
}

impl<T: Sync + Send + 'static + std::fmt::Debug + Clone, const N: usize> From<Block<T, N>>
    for Box<dyn Runnable<T, N>>
{
    fn from(block: Block<T, N>) -> Self {
        Box::new(block)
    }
}

#[async_trait]
impl<T: Sync + Send + 'static + std::fmt::Debug + Clone, const N: usize> Runnable<T, N>
    for Block<T, N>
{
    async fn run_impl(&self, context: &TestContext<T, N>) -> anyhow::Result<Option<Statistic>> {
        let unordered_futures = FuturesUnordered::new();

        for op in self.concurrent.iter() {
            unordered_futures.push(op.run(context));
        }

        for op in self.chain.iter() {
            op.run(context).await?;
        }

        unordered_futures
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        Ok(None)
    }

    fn clone_dyn(&self) -> Box<dyn Runnable<T, N>> {
        Box::new((*self).clone())
    }
}
