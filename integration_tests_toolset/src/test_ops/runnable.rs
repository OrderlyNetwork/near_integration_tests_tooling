use crate::statistic::statistic_consumer::{Statistic, StatisticConsumer};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TestContext {
    statistics: Arc<Mutex<Vec<Box<dyn StatisticConsumer>>>>,
}

pub struct Block {
    pub chain: Vec<Box<dyn Runnable>>,
    pub concurrent: Vec<Box<dyn Runnable>>,
}

#[async_trait]
pub trait Runnable: Sync + Send + std::fmt::Debug + 'static {
    #[allow(unused_variables)]
    async fn prepare(&self, context: &TestContext) -> anyhow::Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    async fn run_impl(&self, context: &TestContext) -> anyhow::Result<Option<Statistic>>;

    #[allow(unused_variables)]
    async fn check_results(&self, context: &TestContext) -> anyhow::Result<()> {
        Ok(())
    }

    async fn run(&self, context: &TestContext) -> anyhow::Result<()> {
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

    fn clone_dyn(&self) -> Box<dyn Runnable>;
}

impl Clone for Box<dyn Runnable> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}
