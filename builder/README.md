# Example for 05-method-chaining.rs

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
        CommandBuilder::default()
    }
}
#[derive(Default)]
pub struct CommandBuilder {
    executable: Option<String>,
    args: Option<Vec<String>>,
    current_dir: Option<String>,
}
impl CommandBuilder {
    pub fn executable(mut self, value: String) -> Self {
        self.executable = Some(value);
        self
    }
    pub fn args(mut self, value: Vec<String>) -> Self {
        self.args = Some(value);
        self
    }
    pub fn current_dir(mut self, value: String) -> Self {
        self.current_dir = Some(value);
        self
    }
    pub fn build(self) -> Result<Command, String> {
        let executable = self.executable.ok_or(format!(
            "field \"{}\" required, but not set yet.",
            stringify!(executable),
        ))?;
        let args = self.args.ok_or(format!(
            "field \"{}\" required, but not set yet.",
            stringify!(args),
        ))?;
        let current_dir = self.current_dir.ok_or(format!(
            "field \"{}\" required, but not set yet.",
            stringify!(current_dir),
        ))?;
        Ok(Command {
            executable,
            args,
            current_dir,
        })
    }
}
```
