# cdx-cli

Command-line interface for working with Codex Document Format (.cdx) files.

## Installation

```bash
cargo install cdx-cli
```

Or build from source:

```bash
cargo build -p cdx-cli --release
```

## Usage

```
cdx <command> [options]

Commands:
  create      Create a new Codex document
  validate    Validate document structure and hashes
  inspect     Display document information
  sign        Add a digital signature
  verify      Verify signatures and integrity
  extract     Extract content or assets
  completions Generate shell completions
```

### Create a Document

```bash
# Create a simple document
cdx create -t "My Document" output.cdx

# Create with author and input file
cdx create -t "Report" -a "John Doe" -i content.md output.cdx

# Create with specific state
cdx create -t "Final Report" --state published output.cdx
```

### Validate a Document

```bash
cdx validate document.cdx
```

### Inspect a Document

```bash
# Basic inspection
cdx inspect document.cdx

# Show block details
cdx inspect document.cdx --blocks

# Show signature details
cdx inspect document.cdx --signatures

# Show provenance chain
cdx inspect document.cdx --provenance
```

### Sign a Document

```bash
# Sign with ECDSA (ES256)
cdx sign document.cdx -k private-key.pem -n "Author Name"

# Sign with EdDSA
cdx sign document.cdx -k ed25519-key.pem -n "Author Name" -a EdDSA

# Sign to a new file
cdx sign document.cdx -k key.pem -n "Author" -o signed-document.cdx
```

### Verify a Document

```bash
# Verify document integrity
cdx verify document.cdx

# Verify with public key
cdx verify document.cdx -k public-key.pem
```

### Extract Content

```bash
# Extract content as JSON
cdx extract document.cdx --content

# Extract as plain text
cdx extract document.cdx --text

# Extract a specific asset
cdx extract document.cdx --asset image.png

# Extract all assets
cdx extract document.cdx --all-assets -o ./extracted/
```

## Global Options

```
-v, --verbose    Increase output verbosity
-q, --quiet      Suppress non-error output
--json           Output as JSON (for scripting)
--color <WHEN>   Color output [auto, always, never]
```

## Shell Completions

```bash
# Bash
cdx completions bash > /etc/bash_completion.d/cdx

# Zsh
cdx completions zsh > ~/.zfunc/_cdx

# Fish
cdx completions fish > ~/.config/fish/completions/cdx.fish

# PowerShell
cdx completions powershell > $PROFILE.CurrentUserAllHosts
```

## JSON Output

All commands support `--json` flag for machine-readable output:

```bash
cdx inspect document.cdx --json | jq '.document_id'
```

## License

MIT OR Apache-2.0
