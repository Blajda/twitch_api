use std::error::Error;

use twitch_api::ClientConfig;

use self::mock_client::MockClient;

pub mod mock_client;

pub struct TestContext {
    pub config: ClientConfig,
    pub mock_client: MockClient,
}

pub fn setup() -> Result<TestContext, Box<dyn Error>> {
    let config = ClientConfig {
        api_base_uri: "http://localhost:8080/mock".to_string(),
        auth_base_uri: "http://localhost:8080/auth".to_string(),
        ..ClientConfig::default()
    };

    Ok(TestContext {
        mock_client: MockClient::build(),
        config,
    })
}
