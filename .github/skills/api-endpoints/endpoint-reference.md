# ClickUp API v2 — Endpoint Reference

Base URL: `https://api.clickup.com/api/v2`

## Endpoint Table

| Method | Path | Response wrapper | Returns |
|--------|------|-----------------|---------|
| GET | `/user` | `{ "user": ... }` | `User` |
| GET | `/team` | `{ "teams": [...] }` | `Vec<Workspace>` |
| GET | `/team/{team_id}/space?archived=false` | `{ "spaces": [...] }` | `Vec<Space>` |
| GET | `/space/{space_id}` | Direct object | `Space` |
| GET | `/space/{space_id}/folder?archived=false` | `{ "folders": [...] }` | `Vec<Folder>` |
| GET | `/folder/{folder_id}/list?archived=false` | `{ "lists": [...] }` | `Vec<List>` |
| GET | `/space/{space_id}/list?archived=false` | `{ "lists": [...] }` | `Vec<List>` |
| GET | `/list/{list_id}` | Direct object | `List` |
| GET | `/list/{list_id}/task?page={n}` | `{ "tasks": [...], "last_page": bool }` | `Vec<Task>` (paginated) |
| GET | `/task/{task_id}?include_subtasks=true&include_markdown_description=true` | Direct object | `Task` |
| GET | `/task/{task_id}/comment` | `{ "comments": [...] }` | `Vec<Comment>` |

## Pagination

Only task listing (`/list/{list_id}/task`) uses pagination:
- Query parameter: `page` (0-indexed)
- Response includes `last_page: bool`
- Increment `page` until `last_page == true`

## Common Query Parameters

| Endpoint | Parameter | Default | Purpose |
|----------|-----------|---------|---------|
| `/team/{id}/space` | `archived` | `false` | Exclude archived spaces |
| `/space/{id}/folder` | `archived` | `false` | Exclude archived folders |
| `/folder/{id}/list` | `archived` | `false` | Exclude archived lists |
| `/space/{id}/list` | `archived` | `false` | Exclude archived folderless lists |
| `/task/{id}` | `include_subtasks` | `false` | Include subtask data |
| `/task/{id}` | `include_markdown_description` | `false` | Include markdown description field |
