use std::{collections::HashMap, error::Error};

use async_trait::async_trait;

pub trait InputType {}
impl InputType for HashMap<String, String> {}
impl InputType for String {}

#[async_trait]
pub trait ChainTrait<T: InputType>: Send + Sync {
    async fn run(&mut self, inputs: T) -> Result<String, Box<dyn Error>>;
}
