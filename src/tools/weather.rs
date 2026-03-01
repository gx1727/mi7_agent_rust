use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::base::{Tool, ToolSchema};

pub struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get current weather for a city"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "City name"
                    }
                },
                "required": ["city"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, anyhow::Error> {
        let city = params["city"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing city parameter"))?;
        
        // Mock weather data
        let weather = json!({
            "city": city,
            "temperature": "22°C",
            "condition": "Sunny",
            "humidity": "45%"
        });
        
        Ok(weather)
    }
}
