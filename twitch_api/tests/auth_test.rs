mod common;

use serial_test::serial;
use std::error::Error;
use twitch_api::HelixClient;

#[tokio::test]
#[serial]
async fn test_helix_auth_ok() -> Result<(), Box<dyn Error>> {
    let context = common::setup()?;

    let mock_client = &context.mock_client;
    let clients = &mock_client.clients().await?;
    let client_id = &clients.data[0].id;
    let client_secret = &clients.data[0].secret;

    let helix_client = HelixClient::new_with_config(client_id, context.config)
        .authenticate(client_secret)
        .build()
        .await;

    assert!(helix_client.is_ok());
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_helix_auth_failure() -> Result<(), Box<dyn Error>> {
    let context = common::setup()?;

    let mock_client = &context.mock_client;
    let clients = &mock_client.clients().await?;
    let client_id = &clients.data[0].id;
    let client_secret = "INVALID_SECRET";

    let helix_client = HelixClient::new_with_config(client_id, context.config)
        .authenticate(client_secret)
        .build()
        .await;

    let err = helix_client.err().unwrap();
    assert_eq!(err.is_auth_error(), true);
    Ok(())
}
