use crate::context::TestContext;
use async_trait::async_trait;
use integration_tests_toolset::statistic::statistic_consumer::Statistic;

#[async_trait]
pub trait Runnable<T: Sync + Send + std::fmt::Debug, U, const N: usize, const M: usize>:
    Sync + Send + std::fmt::Debug
{
    async fn run_impl(
        &self,
        context: &TestContext<T, U, N, M>,
    ) -> anyhow::Result<Option<Statistic>>;

    async fn run(&self, context: &TestContext<T, U, N, M>) -> anyhow::Result<()>
    where
        T: std::fmt::Debug,
    {
        let res = self.run_impl(context).await?;

        if let Some(stat) = res {
            let mut statistics = context.statistics.lock().await;
            for statistic in statistics.iter_mut() {
                statistic.consume_statistic(stat.clone());
            }
        }

        Ok(())
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

#[macro_export]
macro_rules! runnable {(
    $struct_vis:vis struct $struct_name:ident {
         $(
             $(#[$field_meta:meta])*
             $field_vis:vis $field_name:ident : $field_type:ty
         ),*$(,)*
     }$(,)*
     async fn run_impl (&$self:ident, $context:ident: &TestContext<$contract_template:ty, U, N, M>,) -> $func_ret:ty $run_impl_body:block
 ) => {
     #[derive(Debug, Clone)]
     $struct_vis struct $struct_name {
         $($field_vis $field_name: $field_type,)*
     }

     #[async_trait]
     impl<U, const N: usize, const M: usize> Runnable<$contract_template, U, N, M> for $struct_name {
         #[allow(unused_variables)]
         async fn run_impl (&$self, $context: &TestContext<$contract_template, U, N, M>,) -> $func_ret {
             $run_impl_body
         }

         fn clone_dyn(&self) -> Box<dyn Runnable<$contract_template, U, N, M>> {
             Box::new(self.clone())
         }
     }

     impl<U, const N: usize, const M: usize> From<$struct_name> for Box<dyn Runnable<$contract_template, U, N, M>> {
         fn from(op: $struct_name) -> Self {
             Box::new(op)
         }
     }

     impl<U, const N: usize, const M: usize> From<$struct_name> for Block<$contract_template, U, N, M> {
         fn from(op: $struct_name) -> Self {
             Self {
                 chain: vec![Box::new(op)],
                 concurrent: vec![],
             }
         }
     }
 }
}
