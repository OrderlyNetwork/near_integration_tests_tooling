use crate::context::TestContext;
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use integration_tests_toolset::statistic::statistic_consumer::Statistic;

use super::runnable::Runnable;

// TODO: add possibility to block to print and clean statistics
// possibly implement it as trait?
// or as parameter to block?
#[derive(Debug, Clone, Default)]
pub struct Block<T, U, const N: usize, const M: usize>
where
    Box<dyn Runnable<T, U, N, M>>: Clone,
    T: Clone + std::fmt::Debug,
{
    pub chain: Vec<Box<dyn Runnable<T, U, N, M>>>,
    pub concurrent: Vec<Box<dyn Runnable<T, U, N, M>>>,
}

impl<T, U, const N: usize, const M: usize> Block<T, U, N, M>
where
    Box<(dyn Runnable<T, U, N, M>)>: Clone,
    T: Clone + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            chain: vec![],
            concurrent: vec![],
        }
    }
}

impl<
        T: Sync + Send + std::fmt::Debug + Clone + 'static,
        U: std::fmt::Debug + Clone + 'static,
        const N: usize,
        const M: usize,
    > From<Block<T, U, N, M>> for Box<dyn Runnable<T, U, N, M>>
{
    fn from(block: Block<T, U, N, M>) -> Self {
        Box::new(block)
    }
}

#[async_trait]
impl<
        T: Sync + Send + std::fmt::Debug + Clone + 'static,
        U: std::fmt::Debug + Clone + 'static,
        const N: usize,
        const M: usize,
    > Runnable<T, U, N, M> for Block<T, U, N, M>
{
    async fn run_impl(
        &self,
        context: &TestContext<T, U, N, M>,
    ) -> anyhow::Result<Option<Statistic>> {
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

    fn clone_dyn(&self) -> Box<dyn Runnable<T, U, N, M>> {
        Box::new((*self).clone())
    }
}

impl<T, U, const N: usize, const M: usize> Block<T, U, N, M>
where
    Box<dyn Runnable<T, U, N, M>>: Clone,
    T: Clone + std::fmt::Debug,
{
    pub fn add_chain_op(mut self, op: Box<dyn Runnable<T, U, N, M>>) -> Self {
        self.chain.push(op);
        self
    }

    pub fn add_chain_ops(mut self, ops: &[Box<dyn Runnable<T, U, N, M>>]) -> Self {
        self.chain.extend_from_slice(ops);
        self
    }

    pub fn add_concurrent_op(mut self, op: Box<dyn Runnable<T, U, N, M>>) -> Self {
        self.concurrent.push(op);
        self
    }

    pub fn add_concurrent_ops<'a>(mut self, ops: &'a [Box<dyn Runnable<T, U, N, M>>]) -> Self {
        self.concurrent.extend_from_slice(ops);
        self
    }
}
