use futures::{future::try_join_all, try_join, Future, FutureExt, TryFutureExt};
use integration_tests_toolset::{
    error::TestError, statistic::statistic_consumer::Statistic, tx_result::TxResult,
};
use std::pin::Pin;

type ExecutionFuture<'a> = Pin<Box<dyn Future<Output = Result<Statistic, TestError>> + Send + 'a>>;
type CallFuture<'a, T> = Pin<Box<dyn Future<Output = Result<TxResult<T>, TestError>> + Send + 'a>>;

pub enum ExecutionOperation<'a> {
    SubBatch(Batch<'a>),
    Future(ExecutionFuture<'a>),
}

impl<'a> ExecutionOperation<'a> {
    pub async fn run(self) -> Result<Vec<Statistic>, TestError> {
        let res = match self {
            ExecutionOperation::SubBatch(block) => block.run().await?,
            ExecutionOperation::Future(future) => vec![future.await?],
        };

        Ok(res)
    }
}

pub struct Batch<'a> {
    pub chain: Vec<ExecutionOperation<'a>>,
    pub concurrent: Vec<ExecutionOperation<'a>>,
}

impl<'a> Batch<'a> {
    pub fn new() -> Self {
        Self {
            chain: vec![],
            concurrent: vec![],
        }
    }
}

impl<'a> Batch<'a> {
    pub fn run(
        self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Statistic>, TestError>> + Send + 'a>> {
        let async_block = move || async {
            let join_result = try_join!(
                try_join_all(
                    [|| async {
                        let mut statistics = vec![];
                        for op in self.chain.into_iter() {
                            let res = op.run().await?;
                            res.into_iter().for_each(|stat| statistics.push(stat));
                        }
                        Ok::<Vec<Statistic>, TestError>(statistics)
                    }]
                    .into_iter()
                    .map(|a| a())
                ),
                try_join_all(self.concurrent.into_iter().map(|op| op.run())),
            );
            join_result
        };

        async_block()
            .and_then(|join_res| async move {
                let mut res = join_res.0;
                res.extend(join_res.1.into_iter());
                Ok::<Vec<Statistic>, TestError>(res.into_iter().flatten().collect())
            })
            .boxed()
    }

    pub fn add_chain_op(mut self, op: ExecutionOperation<'a>) -> Self {
        self.chain.push(op);
        self
    }

    pub fn add_chain_ops(mut self, ops: Vec<ExecutionOperation<'a>>) -> Self {
        self.chain.extend(ops);
        self
    }

    pub fn add_concurrent_op(mut self, op: ExecutionOperation<'a>) -> Self {
        self.concurrent.push(op);
        self
    }

    pub fn add_concurrent_ops(mut self, ops: Vec<ExecutionOperation<'a>>) -> Self {
        self.concurrent.extend(ops);
        self
    }
}

impl<'a> From<Batch<'a>> for ExecutionOperation<'a> {
    fn from(value: Batch<'a>) -> Self {
        ExecutionOperation::SubBatch(value)
    }
}

impl<'a> From<ExecutionFuture<'a>> for ExecutionOperation<'a> {
    fn from(value: ExecutionFuture<'a>) -> Self {
        ExecutionOperation::Future(value)
    }
}

impl<'a, T: 'a> From<CallFuture<'a, T>> for ExecutionOperation<'a> {
    fn from(value: CallFuture<'a, T>) -> Self {
        let res = value.map(|res| res.map(|tx| Statistic::from(tx))).boxed();
        ExecutionOperation::Future(res)
    }
}

pub fn make_op<'a, T>(
    input: impl Future<Output = Result<TxResult<T>, TestError>> + Send + 'a,
) -> ExecutionOperation<'a> {
    input
        .map(|res| res.map(|tx| Statistic::from(tx)))
        .boxed()
        .into()
}
