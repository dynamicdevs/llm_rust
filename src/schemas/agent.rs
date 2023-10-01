use std::collections::HashMap;

pub enum ToolInput {
    //Will implement this in the future
    StrInput(String),
    DictInput(HashMap<String, String>),
}
pub struct AgentAction {
    pub tool: String,
    pub tool_input: String, //this should be ToolInput in the future
    pub log: String, //esto es el proceso de la ia antes de la respuesta del tool Osea 'debo usar esra herramiensta para saber xxx {tool:xxx,input:yyy}'
}

pub struct AgentFinish {
    pub return_values: String,
}

pub enum AgentEvent {
    Action(AgentAction),
    Finish(AgentFinish),
}
