# YNAT

A Terminal User Interface (TUI) application for interacting with your YNAB (You Need A Budget) account.

## Features

- OAuth 2.0 authentication with YNAB
- Secure token storage using system keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Automatic token refresh
- View and browse your YNAB budgets
- Clean, interactive terminal interface

## Setup

### 1. Create a YNAB OAuth Application

1. Go to https://app.ynab.com/settings/developer
2. Click "New Application"
3. Fill in the details:
   - **Application Name**: YNAT (or whatever you prefer)
   - **Redirect URI**: `http://localhost:8080/callback`
   - **Application Website**: (optional)
4. Click "Save"
5. Copy your **Client ID** and **Client Secret**

### 2. Configure the Application

1. Copy the example configuration file:
   ```bash
   cp config.example.toml config.toml
   ```

2. Edit `config.toml` and add your credentials:
   ```toml
   [oauth]
   client_id = "your_client_id_here"
   client_secret = "your_client_secret_here"
   redirect_uri = "http://localhost:8080/callback"
   ```

**Important**: The `config.toml` file is git-ignored to prevent accidentally committing your credentials.

### 3. Run the Application

```bash
cargo run
```

## First-Time Authentication

1. When you first run the application, it will check if you're authenticated
2. If not authenticated, press 'a' to start the authorization flow
3. Your browser will open automatically to the YNAB authorization page
4. Click "Authorize" to grant access
5. You'll be redirected back and the application will save your access token securely
6. Future runs will automatically use your saved token

## Key Bindings

### Authentication Screen
- **'a'**: Start authorization (when not authenticated)
- **'r'**: Retry authentication (after an error)
- **'q'**: Quit the application

### Budgets Screen
- **↑/↓**: Navigate through budgets
- **'q'**: Quit the application

## Security

- Access tokens are stored in `~/Library/Caches/ynat/token.json` with 0600 permissions (owner read/write only)
- Tokens are automatically refreshed before expiration
- OAuth state parameter is validated to prevent CSRF attacks
- Client credentials are read from `config.toml` (not committed to git)

## Token Storage Location

- **macOS/Linux**: `~/Library/Caches/ynat/token.json` (macOS) or `~/.cache/ynat/token.json` (Linux)
- File permissions set to 0600 (readable/writable only by owner)

## Troubleshooting

### Browser doesn't open automatically

If the browser fails to open, you'll see a URL in the terminal. Copy and paste it into your browser manually.

### Port 8080 already in use

The default redirect URI uses port 8080. If this port is in use:
1. Choose a different port (e.g., 8081)
2. Update the redirect_uri in `config.toml`
3. Update the redirect URI in your YNAB OAuth application settings

### Token file permissions

The token file is created with 0600 permissions (owner read/write only). If you need to manually delete your token, remove `~/Library/Caches/ynat/token.json`.

### Configuration errors

Make sure your `config.toml` file exists and contains valid `client_id` and `client_secret` values from your YNAB OAuth application.

## Development

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```

### Running in release mode

```bash
cargo run --release
```

## Architecture

The application is structured into several modules:

- **auth**: OAuth2 authentication, token storage, callback server
- **api**: YNAB API client (to be implemented)
- **ui**: Terminal user interface components
- **config**: Configuration management

## Current Features

- ✅ OAuth 2.0 authentication
- ✅ Secure token storage and refresh
- ✅ View and browse budgets

## Future Features

- View budget details and accounts
- View transactions
- Add new transactions
- Budget category management
- Reports and analytics

## License

MIT
