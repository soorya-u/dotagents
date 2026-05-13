---
name: hello-skill
description: Greets the user and demonstrates skill capabilities. Use when asked to say hello or to show how skills work.
license: MIT
compatibility: Any agent supporting the Agent Skills specification
metadata:
  author: dotagents
  version: "1.0.0"
---

# Hello Skill for {{ var.agent_name }}

This is a sample skill demonstrating the Agent Skills specification format using {{ env.app_name }}

## Instructions

When activated, respond with a friendly greeting and briefly explain that
skills are reusable, model-invoked capabilities that bundle instructions,
scripts, and resources for specific tasks.

## Example

> Hello! I'm using the hello-skill. Skills let agents load focused,
> task-specific knowledge on demand.
