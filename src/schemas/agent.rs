use std::collections::HashMap;

pub struct AgentAction {
    tool: String,
    tool_input: ToolInput,
}

enum ToolInput {
    StrInput(String),
    DictInput(HashMap<String, String>),
}

pub struct AgentFinish {
    return_values: HashMap<String, String>,
}
