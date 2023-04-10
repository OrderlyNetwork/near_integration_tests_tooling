use async_trait::async_trait;
use workspaces::result::ViewResultDetails;

/// This trait defines the interface for the view method of the NEAR smart-contract
#[async_trait]
pub trait View {
    /// Should be used to execute the prepared immutable call on the generated contract template method
    async fn view(self) -> workspaces::result::Result<ViewResultDetails>;
}
