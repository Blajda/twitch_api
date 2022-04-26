use std::io::Result;
use std::process::{Child, Command};
use std::time::Duration;

pub struct MockServer {
    child: Child,
}

pub struct MockServerBuilder {
    port: u32,
    config: String,
}

impl Default for MockServerBuilder {
    fn default() -> Self {
        MockServerBuilder {
            port: 8080,
            config: String::new(),
        }
    }
}

impl MockServerBuilder {
    pub fn port(&mut self, port: u32) -> &mut Self {
        self.port = port;
        self
    }

    pub fn config(&mut self, config: String) -> &mut Self {
        self.config = config;
        self
    }

    pub fn build(&self) -> Result<MockServer> {
        let mut command = Command::new("twitch");
        command.arg("mock-api");
        command.arg("--port").arg(self.port.to_string());
        if !self.config.is_empty() {
            command.arg("--config").arg(self.config.clone());
        }
        command.arg("start");

        let child = command.spawn()?;
        //TODO: Capture writes to stdout and look for string "Mock server started" or timeout
        //writes should still be passed to the underlying destination
        std::thread::sleep(Duration::from_secs(4));
        Ok(MockServer { child })
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        let _r = self.child.kill();
    }
}
