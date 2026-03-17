# ClickUp API v2 — Quick Reference for clickup-rs

Base URL: `https://api.clickup.com/api/v2`

## Authentication

### Personal Token
```
Authorization: pk_YOUR_TOKEN
```

### OAuth2 Flow
1. Redirect user to: `https://app.clickup.com/api?client_id={ID}&redirect_uri={URI}`
2. User authorizes → redirected to `{URI}?code={CODE}`
3. Exchange code: `POST /oauth/token` with `{ client_id, client_secret, code }`
4. Use token: `Authorization: Bearer {TOKEN}`

## Endpoints

### User
| Method | Path | Description |
|--------|------|-------------|
| GET | `/user` | Get authenticated user |

### Workspaces
| Method | Path | Description |
|--------|------|-------------|
| GET | `/team` | List all workspaces |

### Spaces
| Method | Path | Description |
|--------|------|-------------|
| GET | `/team/{team_id}/space` | List spaces in workspace |
| GET | `/space/{space_id}` | Get space details |

### Folders
| Method | Path | Description |
|--------|------|-------------|
| GET | `/space/{space_id}/folder` | List folders in space |

### Lists
| Method | Path | Description |
|--------|------|-------------|
| GET | `/folder/{folder_id}/list` | Lists in a folder |
| GET | `/space/{space_id}/list` | Folderless lists in a space |
| GET | `/list/{list_id}` | Get list details |

### Tasks
| Method | Path | Description |
|--------|------|-------------|
| GET | `/list/{list_id}/task` | List tasks (paginated: `?page=0`) |
| GET | `/task/{task_id}` | Get task detail (`?include_subtasks=true&include_markdown_description=true`) |

### Comments
| Method | Path | Description |
|--------|------|-------------|
| GET | `/task/{task_id}/comment` | List task comments |

## Response Patterns

### Pagination (Tasks)
```json
{
  "tasks": [...],
  "last_page": false
}
```
Increment `page` param until `last_page == true`.

### Error
```json
{
  "err": "Team not found",
  "ECODE": "ITEM_015"
}
```

### Rate Limit Headers
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 96
X-RateLimit-Reset: 1679012345
```

## Sample Responses

### GET /user
```json
{
  "user": {
    "id": 123,
    "username": "John Doe",
    "email": "john@example.com",
    "color": "#7B68EE",
    "profilePicture": "https://...",
    "initials": "JD"
  }
}
```

### GET /team
```json
{
  "teams": [
    {
      "id": "1234",
      "name": "My Workspace",
      "color": "#40BC86",
      "avatar": "https://...",
      "members": [
        {
          "user": {
            "id": 123,
            "username": "John Doe",
            "email": "john@example.com",
            "color": "#7B68EE",
            "profilePicture": null,
            "initials": "JD"
          }
        }
      ]
    }
  ]
}
```

### GET /list/{list_id}/task
```json
{
  "tasks": [
    {
      "id": "abc123",
      "custom_id": null,
      "name": "Example Task",
      "description": "Task description in plain text",
      "text_content": "Task description in plain text",
      "markdown_description": "# Task\n\nThis is **bold** and *italic*\n\n- Item 1\n- Item 2",
      "status": {
        "id": "st_1",
        "status": "in progress",
        "color": "#4194f6",
        "type": "custom"
      },
      "orderindex": "1.00000000000000000000",
      "date_created": "1679012345000",
      "date_updated": "1679012400000",
      "date_closed": null,
      "date_done": null,
      "creator": {
        "id": 123,
        "username": "John Doe",
        "email": "john@example.com",
        "color": "#7B68EE",
        "profilePicture": null,
        "initials": "JD"
      },
      "assignees": [],
      "priority": {
        "id": "2",
        "priority": "high",
        "color": "#ffcc00"
      },
      "due_date": "1679100000000",
      "start_date": null,
      "tags": [
        { "name": "bug", "tag_fg": "#fff", "tag_bg": "#ff0000" }
      ],
      "list": { "id": "list1", "name": "Sprint 1" },
      "folder": { "id": "fold1", "name": "Development" },
      "space": { "id": "sp1" },
      "url": "https://app.clickup.com/t/abc123",
      "parent": null,
      "subtasks": [],
      "custom_fields": [
        {
          "id": "cf1",
          "name": "Story Points",
          "type": "number",
          "value": 5
        }
      ]
    }
  ],
  "last_page": true
}
```
