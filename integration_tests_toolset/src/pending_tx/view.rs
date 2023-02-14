use async_trait::async_trait;
use workspaces::result::ViewResultDetails;

#[async_trait]
pub trait View {
    async fn view(self) -> workspaces::result::Result<ViewResultDetails>;
}
