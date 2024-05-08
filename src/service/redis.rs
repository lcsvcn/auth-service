use serde_json::Value;
use redis::{Client, Commands};
    
pub struct RedisService {
    client: Client,
}

impl RedisService {
    pub fn new(redis_url: &str) -> RedisService {
        let client = Client::open(redis_url).expect("Error connecting to Redis");
        RedisService { client }
    }

    pub fn send_event(&mut self, event_data: &Value) -> redis::RedisResult<()> {
        // Get current timestamp in milliseconds
        let mut connection = self.client.get_connection()?;

        connection.publish("events", event_data.to_string())?;

        Ok(())
    }
}