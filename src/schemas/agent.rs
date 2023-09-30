use std::collections::HashMap;

pub enum ToolInput {
    //Will implement this in the future
    StrInput(String),
    DictInput(HashMap<String, String>),
}
pub struct AgentAction {
    pub tool: String,
    pub tool_input: String, //this should be ToolInput in the future
}

pub struct AgentFinish {
    pub return_values: String,
}

pub enum AgentEvent {
    Action(AgentAction),
    Finish(AgentFinish),
}
