# Example for 01-parse.rs

Given input:

```Rust
#[derive(Builder)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    current_dir: String,
}
```

This version will output such code:

```Rust
impl Command {
    pub fn builder() -> CommandBuilder {
        CommandBuilder
    }
}
pub struct CommandBuilder;
```
