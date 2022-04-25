
use std::error::Error;

use twitch_api::ClientConfig;

use self::{twitch_cli::{MockServer, MockServerBuilder}, mock_client::MockClient};

pub mod twitch_cli;
pub mod mock_client;

pub struct TestContext {
    pub server: MockServer,
    pub config: ClientConfig,
    pub mock_client: MockClient,
}

pub fn setup() -> Result<TestContext, Box<dyn Error>> {
    let server = setup_mock_api()?;
    let config = ClientConfig {
        api_base_uri: "http://localhost:8080/mock".to_string(),
        auth_base_uri: "http://localhost:8080/auth".to_string(),
        ..ClientConfig::default()
    };

    Ok(TestContext {
        mock_client: MockClient::build(),
        server,
        config,
    })
}

pub fn setup_mock_api() -> std::io::Result<MockServer> {
    MockServerBuilder::default().build()
}