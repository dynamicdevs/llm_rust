const SYSTEM_MESSAGE_PREFIX: &str =
    r#"Answer the following questions as best you can. You have access to the following tools:"#;

const FORMAT_INSTRUCTIONS: &str = r#"The way you use the tools is by specifying a json blob.
Specifically, this json should have a `action` key (with the name of the tool to use) and a `action_input` key (with the input to the tool going here).

The only values that should be in the "action" field are: {tool_names}

The $JSON_BLOB should only contain a SINGLE action, do NOT return a list of multiple actions. Here is an example of a valid $JSON_BLOB:

```
{{{{
  "action": $TOOL_NAME,
  "action_input": $INPUT
}}}}
```

ALWAYS use the following format:

Question: the input question you must answer
Thought: you should always think about what to do
Action:
```
$JSON_BLOB
```
Observation: the result of the action
... (this Thought/Action/Observation can repeat N times)
Thought: I now know the final answer
Final Answer: the final answer to the original input question"#;

const SYSTEM_MESSAGE_SUFFIX: &str =
    r#"Begin! Reminder to always use the exact characters `Final Answer` when responding."#;

const HUMAN_MESSAGE: &str = r#"{input}\n\n{agent_scratchpad}"#;

//historial
//pregunta nueva->cual es el presindente de peru y cuantos anos tiene
//entra al agente
//---------------
//historia=[basia]
//prompt + tools ->google
//pregunta nueve->cual es el presindente de peru y cuantos anos tiene
//me devuelta que usar -> google presindete de peru
//observation:El presidente de peru es xxx
//lugo todo esto se va a ir al siguientre prompt
//que seria algo como Question: cual es el presindente de peru y cuantos anos tiene
////observacion:20//though:
