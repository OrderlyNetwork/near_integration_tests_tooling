use crate::context::TestContext;
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use integration_tests_toolset::statistic::statistic_consumer::Statistic;

use super::runnable::Runnable;

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
    Box<(dyn Runnable<T, N>)>: Clone,
    T: Clone + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            chain: vec![],
            concurrent: vec![],
        }
    }
}

impl<T: Sync + Send + std::fmt::Debug + Clone + 'static, const N: usize> From<Block<T, N>>
    for Box<dyn Runnable<T, N>>
{
    fn from(block: Block<T, N>) -> Self {
        Box::new(block)
    }
}

#[async_trait]
impl<T: Sync + Send + std::fmt::Debug + Clone + 'static, const N: usize> Runnable<T, N>
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

impl<T, const N: usize> Block<T, N>
where
    Box<dyn Runnable<T, N>>: Clone,
    T: Clone + std::fmt::Debug,
{
    pub fn add_chain_op(mut self, op: Box<dyn Runnable<T, N>>) -> Self {
        self.chain.push(op);
        self
    }

    pub fn add_chain_ops(mut self, ops: &[Box<dyn Runnable<T, N>>]) -> Self {
        self.chain.extend_from_slice(ops);
        self
    }

    pub fn add_concurrent_op(mut self, op: Box<dyn Runnable<T, N>>) -> Self {
        self.concurrent.push(op);
        self
    }

    pub fn add_concurrent_ops<'a>(mut self, ops: &'a [Box<dyn Runnable<T, N>>]) -> Self {
        self.concurrent.extend_from_slice(ops);
        self
    }
}
