# SKILLS.md — clickup-rs Development Skills

Development skills for the clickup-rs project are defined in `.github/skills/`.
Each skill has a `SKILL.md` file with scope, deliverables, implementation guidance, and acceptance criteria.
Copilot loads skills automatically when relevant to the current task.

## Skill Index

| # | Skill | Directory | Scope |
|---|-------|-----------|-------|
| 1 | Foundation | `foundation/` | Cargo workspace, Docker, CI/CD |
| 2 | Error & Config | `error-config/` | Error types, config, token storage |
| 3 | HTTP Client | `http-client/` | ClickUpClient, rate limiter, pagination |
| 4 | Domain Models | `domain-models/` | Serde structs for all API entities |
| 5 | API Endpoints | `api-endpoints/` | Typed async endpoint methods |
| 6 | CLI Auth | `cli-auth/` | CLI authentication commands |
| 7 | CLI Data | `cli-data/` | CLI browsing commands with rich output |
| 8 | TUI Scaffold | `tui-scaffold/` | App skeleton, event loop, layout |
| 9 | TUI Workspaces | `tui-workspaces/` | Workspace & space screens |
| 10 | TUI Tasks | `tui-tasks/` | List view, task detail, markdown |
| 11 | TUI Polish | `tui-polish/` | Search, help, themes, caching |
| 12 | Docs & Release | `docs-release/` | README, tests, release automation |
| 13 | Log Analysis | `log-analysis/` | Log format, diagnosis, bug report analysis |

## Dependency Graph

```
Skill 1 (Foundation)
  ├── Skill 2 (Errors/Config) → Skill 3 (HTTP Client) ─┐
  └── Skill 4 (Models) ────────────────────────────────┘
                                                         ↓
                                                    Skill 5 (Endpoints)
                                                    ↙           ↘
                                          Skill 6 (CLI Auth)   Skill 8 (TUI Scaffold)
                                              ↓                    ↓
                                          Skill 7 (CLI Data)  Skill 9 (TUI Workspaces)
                                              ↓                    ↓
                                              ↓               Skill 10 (TUI Tasks)
                                              ↓                    ↓
                                              ↓               Skill 11 (TUI Polish)
                                              ↓                    ↓
                                              └──── Skill 12 (Docs & Release) ────┘
                                                              ↓
                                                     Skill 13 (Log Analysis)
```

Skill 6→7 (CLI track) and Skill 8→9→10→11 (TUI track) are parallelizable after Skill 5.
Skill 13 (Log Analysis) can be developed independently once the logging infrastructure is in place.
