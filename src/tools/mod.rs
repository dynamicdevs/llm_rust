use std::error::Error;
use std::string::String;

pub trait Tool {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn call(&self, input: &str) -> Result<String, Box<dyn Error>>;
}
