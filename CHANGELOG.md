# Changelog


## Bug Fixes

- Updated the config file extension and schema
- Made provider and config cross compatible
- Revamped application config (#5)
- Command extension has been switched from command level to config level (#12)
- Variables are now accessed under var object (#18)
- Cap outbound HTTP timeout to 5s to prevent e2e test hangs
- Make templater initialization fallible, propagate errors gracefully (#68)
- Defer workspace dir creation until after TUI wizard to prevent empty dirs on cancel (#69)
- Add --unreleased flag to git-cliff in release-prep workflow

## Features

- Completed init command (#3)
- Added force flag for init and added metadata for options
- Added windsurf and copilot provider
- Added merge config path
- Added deploy command (#4)
- Added config extraction method to app config (#7)
- Completed the working prototype of deploy command (#8)
- Added all command templates through speckit reference (#14)
- Added dotenv support (#16)
- Added ci pipeline to generate registry.json (#23)

## Miscellaneous

- Project init
- Initiated specify and its files (#1)
- Added necessary dependencies
- Scaffolded necessary files
- Added utility function
- Added handlebars
- Seperated schema and builder
- Fixed mock instruction content
- Added amp settings for instructions and mcp (#21)
- Added code improvements to the code (#22)
- Added gh action write permissions (#24)
- Added manual run option
- Registry.json works if workflow is dispatch
- Renamed run script to dev
- Update registry.json (#37)
- Updated mocks
- Update registry.json (#41)
- Update registry.json (#44)
- Update registry.json (#47)
- Reused things in paths
- Added updated skills of openspec
- Added mise dev command and fixed minor typo
- Bump version to 0.0.0-nightly for pre-release testing

