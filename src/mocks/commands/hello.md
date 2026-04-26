---
name: hello
description: A Hello Command to greet the User
---

# Hello Command for {{ var.agent_name }}

Greet the User with his name if present, else greet user as stranger. Tell him you are {{ env.app_name }} command.

Context: $USER_INPUT
