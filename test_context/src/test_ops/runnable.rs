use crate::context::TestContext;
use async_trait::async_trait;
use integration_tests_toolset::statistic::statistic_consumer::Statistic;

// #[cfg(feature = "stress_test")]
pub struct Block<T, const N: usize> {
    pub chain: Vec<Box<dyn Runnable<T, N>>>,
    pub concurrent: Vec<Box<dyn Runnable<T, N>>>,
}

// #[cfg(feature = "stress_test")]
#[async_trait]
pub trait Runnable<T: Sync + Send + 'static, const N: usize>:
    Sync + Send + std::fmt::Debug + 'static
{
    #[allow(unused_variables)]
    async fn prepare(&self, context: &TestContext<T, N>) -> anyhow::Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    async fn run_impl(&self, context: &TestContext<T, N>) -> anyhow::Result<Option<Statistic>>;

    #[allow(unused_variables)]
    async fn check_results(&self, context: &TestContext<T, N>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn run(&self, context: &TestContext<T, N>) -> anyhow::Result<()> {
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

    fn clone_dyn(&self) -> Box<dyn Runnable<T, N>>;
}

// #[cfg(feature = "stress_test")]
impl<T: Sync + Send + 'static, const N: usize> Clone for Box<dyn Runnable<T, N>> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}
