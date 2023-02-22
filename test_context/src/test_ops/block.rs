use std::{io, marker::PhantomData, pin::Pin};

use crate::context::TestContext;
use async_trait::async_trait;
use futures::{
    future::{join_all, try_join_all, LocalBoxFuture},
    stream::FuturesUnordered,
    try_join, Future, StreamExt,
};
use integration_tests_toolset::{error::TestError, statistic::statistic_consumer::Statistic};

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
        try_join!(
            try_join_all(
                [|| async {
                    for op in self.chain.iter() {
                        op.run(context).await?;
                    }
                    Ok(())
                }]
                .iter()
                .map(|a| a())
            ),
            try_join_all(self.concurrent.iter().map(|op| op.run(context)))
        )?;

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

///// here is the alternative implementation
///
// #[derive(Debug, Clone, Default)]

type Closure<T, U, const N: usize, const M: usize> =
    Box<dyn Fn(&TestContext<T, U, N, M>) -> LocalBoxFuture<Result<(), TestError>>>;
// #[derive(Debug)]
pub struct Block2<T, U, const N: usize, const M: usize>
where
    // Box<dyn Runnable<T, U, N, M>>: Clone,
    // F: Fn(&TestContext<T, U, N, M>) -> LocalBoxFuture<Result<(), TestError>> + Clone,
    T: Clone + std::fmt::Debug + Send + Sync,
    // U: std::fmt::Debug + Clone + 'static,
{
    // _phantom_t: PhantomData<T>,
    // _phantom_u: PhantomData<U>,
    pub chain: Vec<Closure<T, U, N, M>>,
    pub concurrent: Vec<Closure<T, U, N, M>>,
}

impl<T, U, const N: usize, const M: usize> Block2<T, U, N, M>
where
    // Box<(dyn Runnable<T, U, N, M>)>: Clone,
    T: Clone + std::fmt::Debug + Send + Sync,
    // U: std::fmt::Debug + Clone + 'static,
    // F: Fn(&TestContext<T, U, N, M>) -> LocalBoxFuture<Result<(), TestError>> + Clone,
{
    pub fn new() -> Self {
        Self {
            chain: vec![],
            concurrent: vec![],
            // _phantom_t: PhantomData,
            // _phantom_u: PhantomData,
        }
    }
}

// impl<
//         T: Sync + Send + std::fmt::Debug + Clone + 'static,
//         U: std::fmt::Debug + Clone + 'static,
//         const N: usize,
//         const M: usize,
//     > From<Block2<T, U, N, M>> for Box<dyn Runnable<T, U, N, M>>
// {
//     fn from(block: Block2<T, U, N, M>) -> Self {
//         Box::new(block)
//     }
// }

// #[async_trait]
impl<
        T: Clone + std::fmt::Debug + Send + Sync + 'static,
        U,
        // U: std::fmt::Debug + Clone + 'static,
        // F: Fn(&TestContext<T, U, N, M>) -> LocalBoxFuture<Result<(), TestError>> + Clone,
        //  U: std::fmt::Debug + Clone + 'static,
        const N: usize,
        const M: usize,
    > Block2<T, U, N, M>
{
    pub async fn run_impl(
        &self,
        context: &TestContext<T, U, N, M>,
    ) -> anyhow::Result<Option<Statistic>> {
        try_join!(
            try_join_all(
                [|| async {
                    for op in self.chain.iter() {
                        op(context).await?;
                    }
                    Ok(())
                }]
                .iter()
                .map(|a| a())
            ),
            try_join_all(self.concurrent.iter().map(|op| op(context)))
        )?;

        Ok(None)
    }

    // fn clone_dyn(&self) -> Box<dyn Runnable<T, U, N, M>> {
    //     Box::new((*self).clone())
    // }
}

impl<T, U, const N: usize, const M: usize> Block2<T, U, N, M>
where
    // Box<dyn Runnable<T, U, N, M>>: Clone,
    // F: Fn(&TestContext<T, U, N, M>) -> LocalBoxFuture<Result<(), TestError>> + Clone,
    // U: std::fmt::Debug + Clone + 'static,
    T: Clone + std::fmt::Debug + Send + Sync,
{
    pub fn add_chain_op(mut self, op: Closure<T, U, N, M>) -> Self {
        self.chain.push(op);
        self
    }

    pub fn add_chain_ops(mut self, ops: Vec<Closure<T, U, N, M>>) -> Self {
        self.chain.extend(ops.into_iter());
        // self.chain.extend_from_slice(ops);
        self
    }

    pub fn add_concurrent_op(mut self, op: Closure<T, U, N, M>) -> Self {
        self.concurrent.push(op);
        self
    }

    pub fn add_concurrent_ops(mut self, ops: Vec<Closure<T, U, N, M>>) -> Self {
        // self.concurrent.extend_from_slice(ops);
        self.chain.extend(ops.into_iter());
        self
    }
}
