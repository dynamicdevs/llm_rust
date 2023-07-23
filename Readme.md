# Large Language Model Interaction Library

This Rust library provides easy-to-use functionality for interacting with various large language models (LLM). It offers methods for generating text completions and embedding documents using APIs of popular language models such as OpenAI's GPT-3.

## Features

- Generate text completions
- Embed documents
- Embed queries
- Manage errors from OpenAI's API
- Chat models

## Usage

### Chat Generation

_You can generate a Open AI text completions using the following:_

```rust
let mut chat_llm = ChatOpenAI::default();
let text = chat_llm.execute("Hello, how are you?").await.unwrap();
println!("{}", text);
```

_or a custom model_

```rust
let mut chat_llm = ChatOpenAI::default().with_model(ChatModel::Gpt3_5Turbo16k);
let text = chat_llm.execute("Hello, how are you?").await.unwrap();
println!("{}", text);
```

_more complex emaple_

```rust
let chat_llm = ChatOpenAI::default().with_model(ChatModel::Gpt3_5Turbo16k);

 let json_str = r#"
    {
        "type": "user",
        "content": "Hello,how are you"
    }
    "#;

    // Deserialize JSON string into a HashMap
let messages: HashMap<String, String> = serde_json::from_str(json_str).unwrap();
let mut messages = messages_from_map(vec![mesagges]).unwrap();

let response =
    .chat_llm
    .generate(vec![messages.clone()])
    .await
    .unwrap();

let messages=messages_to_map(messages)
println!("{:?}", messages);
```

## Document Embedding

```rust
use llm::embedding::embedder_trait::Embedder;
use llm::OpenAiEmbedder;

let embedder = OpenAiEmbedder::default();
let embeddings = embedder.embed_documents(vec!["Hello, how are you?".to_string()]).await.unwrap();
println!("{:?}", embeddings);

let query_embedding = embedder.embed_query("Hello, how are you?").await.unwrap();
println!("{:?}", query_embedding);
```

## Note

You'll need to provide OpenAI's API key which can be set in the environment variable OPENAI_API_KEY or passed directly to the constructors.

## Installation

```toml
[dependencies]
large-language-model-interaction = { version = "*", git = "https://github.com/Abraxas-365/llm_rust.git" }
```
