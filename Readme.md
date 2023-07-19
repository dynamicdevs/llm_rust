# Large Language Model Interaction Library

This Rust library provides easy-to-use functionality for interacting with various large language models (LLM). It offers methods for generating text completions and embedding documents using APIs of popular language models such as OpenAI's GPT-3.

## Features

- Generate text completions
- Embed documents
- Embed queries
- Manage errors from OpenAI's API

## Usage

### Text Generation

You can generate text completions using the following:

```rust
use llm::llm_trait::LLM;
use llm::ChatLLM;
use llm::openai_models::ChatModel;

let mut chat_llm = ChatLLM::default();
let text = chat_llm.generate_completition("Hello, how are you?").await.unwrap();
println!("{}", text);
```

## Document Embedding

```rust
use llm::embedding::embedder_trait::Embedder;
use llm::OpenAiEmbedder;

let embedder = OpenAiEmbedder::new("OPENAI_API_KEY".to_string());
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
