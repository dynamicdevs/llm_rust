use std::error::Error;
use std::string::String;

use async_trait::async_trait;

#[async_trait]
pub trait Tool: CloneBox + Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    async fn call(&self, input: &str) -> Result<String, Box<dyn Error>>;
}

pub trait CloneBox {
    fn clone_box(&self) -> Box<dyn Tool>;
}

impl<T> CloneBox for T
where
    T: 'static + Tool + Clone,
{
    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}
