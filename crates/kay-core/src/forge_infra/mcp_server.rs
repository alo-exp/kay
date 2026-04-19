use std::collections::BTreeMap;

use crate::forge_app::McpServerInfra;
use crate::forge_domain::{Environment, McpServerConfig};

use crate::forge_infra::mcp_client::ForgeMcpClient;

#[derive(Clone)]
pub struct ForgeMcpServer;

#[async_trait::async_trait]
impl McpServerInfra for ForgeMcpServer {
    type Client = ForgeMcpClient;

    async fn connect(
        &self,
        config: McpServerConfig,
        env_vars: &BTreeMap<String, String>,
        environment: &Environment,
    ) -> anyhow::Result<Self::Client> {
        Ok(ForgeMcpClient::new(config, env_vars, environment.clone()))
    }
}
