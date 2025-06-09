# 🔐 TOTP Generator

**TOTP Generator** is a CLI and HTTP API tool for generating Time-based One-Time Passwords (TOTP)—think *Google Authenticator in your terminal*.

Built in Rust, it's designed to run **offline**, securely, and efficiently even on low-powered devices like a **Raspberry Pi**.

You can output TOTP codes in one of three ways:

* **Console UI** (default): a fullscreen terminal interface with live-updating tokens.
* **HTTP API**: exposes endpoints like `GET /list` and `GET /code/<name>`.
* **One-time mode**: print a token directly via CLI with `--one-time`.. Basically Google Authenticator in your terminal.

Other features include:

* lock interface with/out password
* auto-lock after *n* seconds (configurable, can be disabled) or manual lock (Press `l`)
* copy to clipboard (devices that support this can be checked at [copypasta](https://github.com/alacritty/copypasta), the library used for this feature)
* 4 "fonts"
* display how many seconds until each token expires

---

## 📦 Requirements

* A TOML-formatted secrets file (see [Secrets Format](#secrets-format))
* Rust (to build from source) - pre-compiled binaries coming soon

---

## 🚀 Usage

```sh
totp-generator [OPTIONS] --secrets <SECRETS>
```

### 🧭 Modes of Operation

This tool operates in one of the following modes:

* **Console UI** *(default)*: Interactive TOTP code viewer with optional lockout.
* **One-time Mode** (`--one-time <YOUR SECRET'S NAME>`): Generates a TOTP for the secret with code \<YOUR SECRET'S NAME> (see TOML file format).
* **HTTP API Mode** (`--bind <ADDR>`): Runs a web service for serving generated codes via HTTP.

If both `--one-time` and `--bind` are omitted, the application defaults to interactive console UI.

### 🔧 Options

| Flag                 | Env Var                 | Description                                                                                      |
| -------------------- | ----------------------- | ------------------------------------------------------------------------------------------------ |
| `-s`, `--secrets`    | `AUTHENTICATOR_SECRETS` | **Required.** Path to the [secrets TOML file](#secrets-format).                                  |
| `-o`, `--one-time`   |                         | Generate a single TOTP code.                                                                     |
| `-b`, `--bind`       |                         | Launches the server in HTTP mode at the specified address.                                       |
| `-p`, `--port`       |                         | Port to listen on. Default: `3000`.                                                              |
| `-l`, `--lock-after` |                         | Inactivity timeout in seconds before locking UI. Use `0` to disable. Default: `300` (5 minutes). |
| `--number-style`     |                         | Display style for numbers. One of: `standard`, `pipe`, `lite`. Default: `standard`.              |

---

## 📁 Secrets Format

The secrets file must be a valid [TOML](https://toml.io/en/) document. Each entry represents a TOTP configuration.

### 📝 Example [`secrets.toml`](#secrets-format)

```toml
[[entry]]
name = "My GitHub"
secret = "JBSWY3DPEHPK3PXP"
timestep = 30

[[entry]]
name = "Work Email"
# "code" is the short way to address which secret to use in --one-time and HTTP modes
code = "gmail"
secret = "ABCD1234EFGH5678"
```

---

## 🖥 Console UI

In the default mode, the application launches a fullscreen console UI that shows a box for each TOTP entry from your secrets file. Each token automatically refreshes when it expires.

### 🔲 Box Layout (per entry):

* **Top Left**: An identifier (`0..9`, `a..j`) — press this key to copy the token to clipboard.
* **Top Right**: The `code` associated with the entry.
* **Center**: The `name` field.
* **Below Center**: The current TOTP token.
* **Bottom Right**: How many seconds remain until the token expires.

The UI supports auto-locking and manual locking, with indicators and password unlock if configured.

### ⌨️ Key Bindings

* `0`..`9`, `a`..`j`: Copy corresponding TOTP to clipboard
* `q`: Quit the application
* `l`: Lock the interface manually

###

---

## 🌐 HTTP API

When run with the `--bind` option, the program exposes a minimal HTTP API for reading available entries and generating tokens.

### `GET /list`

Returns a list of available TOTP entries based on your secrets file (excluding the `secret` field).

#### ✅ Response (application/json)

```json
[
  {
    "name": "My GitHub",
    "code": "",
    "timestep": 30
  },
  {
    "name": "Work Email",
    "code": "gmail",
    "timestep": 30
  }
]
```

### `GET /code/<YOUR SECRET'S 'CODE'>`

Returns the current TOTP token for the matching entry. `YOUR SECRET'S CODE` corresponds to the `code` field in the TOML entry.

The response format depends on the `Accept` header:

* If `Accept: application/json` is sent, the response is structured:

```json
{
  "timestamp": 1749415312,
  "valid_until": 1749415314,
  "token": "846102"
}
```

* Otherwise, the plain token is returned:

```
415314
```

#### ✅ Example:

```sh
curl http://localhost:3000/code/gmail
```

---

## 📌 Examples

Start the interactive console UI:

```sh
totp-generator --secrets ./secrets.toml
```

Use a custom port with no auto-lock:

```sh
totp-generator --secrets ./secrets.toml --port 8080 --lock-after 0
```

Generate a one-time code (using "gmail" as the secret name, referring to the `code` field of the entry in the [`TOML`](#secrets-format)[ file](#secrets-format)):

```sh
export AUTHENTICATOR_SECRETS=$HOME/.google_authenticator.toml
totp-generator --one-time gmail
```

Run as HTTP server:

```sh
totp-generator --secrets ./secrets.toml --bind 127.0.0.1
```

Display codes in pipe style (Console UI only):

```sh
totp-generator --secrets ./secrets.toml --number-style pipe
```

---

## 🛠 Building from Source

```sh
git clone https://github.com/your/repo.git
cd your-repo
cargo build --release
```

---

## TODO

- [ ] Allow digits to optionally set on secrets
- [ ] Concurrent CUI and HTTP API
- [ ] Messages
- [ ] Tracing

## 📃 License

MIT or Apache-2.0

