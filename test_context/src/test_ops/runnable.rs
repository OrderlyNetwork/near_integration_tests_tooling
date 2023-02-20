use crate::context::TestContext;
use async_trait::async_trait;
use integration_tests_toolset::statistic::statistic_consumer::Statistic;

#[async_trait]
pub trait Runnable<T: Sync + Send + std::fmt::Debug, U, const N: usize, const M: usize>:
    Sync + Send + std::fmt::Debug
{
    #[allow(unused_variables)]
    async fn prepare(&self, context: &TestContext<T, U, N, M>) -> anyhow::Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    async fn run_impl(
        &self,
        context: &TestContext<T, U, N, M>,
    ) -> anyhow::Result<Option<Statistic>>;

    #[allow(unused_variables)]
    async fn check_results(&self, context: &TestContext<T, U, N, M>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn run(&self, context: &TestContext<T, U, N, M>) -> anyhow::Result<()>
    where
        T: std::fmt::Debug,
    {
        self.prepare(context).await?;
        let res = self.run_impl(context).await?;

        if let Some(stat) = res {
            let mut statistics = context.statistics.lock().await;
            for statistic in statistics.iter_mut() {
                statistic.consume_statistic(stat.clone());
            }
        }

        self.check_results(context).await
    }

    fn clone_dyn(&self) -> Box<dyn Runnable<T, U, N, M>>;
}

impl<T: Sync + Send + std::fmt::Debug, U, const N: usize, const M: usize> Clone
    for Box<dyn Runnable<T, U, N, M>>
{
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}
