# Timed One-Time Password (TOTP) Generator

Rust based implementation of tools like Google Authenticator. Meant to be used offline (safely) and on low powered devices such as Raspberry Pi.

Choose an output between:

- a console UI as seen below (the main focus of development)
- a HTTP API (GET /code/[which secret from config])
- a one-time command line `--one-time [which secret from config]`

Other features include: 

- lock interface with/out password
- auto-lock after _n_ seconds (configurable, can be disabled)
- manual lock (Press `l`)
- copy to clipboard (devices that support this can be checked at [copypasta](https://github.com/alacritty/copypasta), the library used for this feature
- 4 "fonts"
- countdown of validity of token

## Running it (Options)

| table
Config Options:
| Property | Type | Description |
| --number-style |standard,lite,utf8,pipe|The style to render the numbers. `standard` should fit on all screens; `utf8` requires the most space|


## TODO

- [ ] cross compiled binaries and/or how to install
- [ ] allow http interface and CUI concurrently
- [ ] `valid_until` field in HTTP interface
