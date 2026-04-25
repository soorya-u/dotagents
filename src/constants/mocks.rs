// Mock file contents embedded at compile time
pub const ENV_EXAMPLE: &str = include_str!("../mocks/.env.example");
pub const GITIGNORE: &str = include_str!("../mocks/.gitignore.example");
pub const INSTRUCTIONS: &str = include_str!("../mocks/INSTRUCTIONS.md");
pub const COMMAND_HELLO: &str = include_str!("../mocks/commands/hello.md");
pub const SKILL_HELLO: &str = include_str!("../mocks/skills/hello-skill/SKILL.md");
pub const CONFIG: &str = include_str!("../mocks/config.toml");
pub const LOCAL_CONFIG: &str = include_str!("../mocks/local.config.toml");
pub const MCP: &str = include_str!("../mocks/mcp.jsonc");
pub const TEMPLATE_MYCODE_COMMAND: &str = include_str!("../mocks/templates/mycode/command.hbs");
pub const TEMPLATE_MYCODE_SKILL: &str = include_str!("../mocks/templates/mycode/skill.hbs");
pub const TEMPLATE_MYCODE_INSTRUCTIONS: &str =
    include_str!("../mocks/templates/mycode/instructions.hbs");
pub const TEMPLATE_MYCODE_MCP: &str = include_str!("../mocks/templates/mycode/mcp.hbs");
