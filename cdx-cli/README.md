# cdx-cli

Command-line interface for working with CDX Document Format (.cdx) files.

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
  create           Create a new CDX document
  validate         Validate document structure and hashes
  inspect          Display document information
  status           Show comprehensive document status
  sign             Add a digital signature
  verify           Verify signatures and integrity
  extract          Extract content or assets
  submit-review    Submit document for review (draft -> review)
  freeze           Freeze document (review -> frozen)
  publish          Publish document (frozen -> published)
  revert           Revert document to draft (review -> draft)
  fork             Fork document to create new version with lineage
  prove            Generate a Merkle proof for a block
  verify-proof     Verify a Merkle proof against a document
  show-lineage     Show document lineage (ancestor chain)
  get-metadata     Display document metadata
  set-metadata     Set document metadata fields
  pack             Pack a directory or JSON into a .cdx archive
  diff             Compare two CDX documents
  show-timestamps  Show timestamps in a document
  verify-timestamps Verify timestamps in a document
  add-timestamp    Add a timestamp record to a document
  timestamp-acquire Acquire a timestamp from a timestamp authority
  encrypt          Encrypt a document with password-based encryption
  decrypt          Decrypt a password-encrypted document
  completions      Generate shell completions
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

### Document Status

```bash
# Show comprehensive document status
cdx status document.cdx
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

### State Transitions

```bash
# Submit for review (draft -> review)
cdx submit-review document.cdx

# Freeze document (review -> frozen)
cdx freeze document.cdx

# Publish document (frozen -> published)
cdx publish document.cdx

# Revert to draft (review -> draft, only if no signatures)
cdx revert document.cdx
```

### Forking and Lineage

```bash
# Fork a document to create new version
cdx fork document.cdx -o forked.cdx -n "Updated section 3"

# Show document lineage
cdx show-lineage document.cdx
```

### Merkle Proofs

```bash
# Generate a proof for a specific block
cdx prove document.cdx --block-id "block-123" -o proof.json

# Generate a proof by block index
cdx prove document.cdx --block-index 5 -o proof.json

# Verify a proof
cdx verify-proof document.cdx proof.json
```

### Metadata Management

```bash
# Get document metadata
cdx get-metadata document.cdx

# Set metadata fields
cdx set-metadata document.cdx --title "New Title" --creator "Jane Doe"
cdx set-metadata document.cdx --description "A detailed report" --language "en"
```

### Timestamps

```bash
# Show document timestamps
cdx show-timestamps document.cdx

# Verify timestamps
cdx verify-timestamps document.cdx

# Acquire timestamp from TSA
cdx timestamp-acquire document.cdx --method rfc3161

# Add timestamp record manually
cdx add-timestamp document.cdx --method rfc3161 --authority "tsa.example.com" --token "<base64>"
```

### Encryption

```bash
# Encrypt a document (will prompt for password)
cdx encrypt document.cdx

# Encrypt with password
cdx encrypt document.cdx -p "secret" -o encrypted.cdx

# Decrypt a document
cdx decrypt encrypted.cdx -o decrypted.cdx
```

### Pack Archives

```bash
# Pack from JSON (Pandoc output)
cdx pack content.json -o document.cdx --from-json

# Pack from directory
cdx pack ./unpacked/ -o document.cdx
```

### Compare Documents

```bash
# Diff two documents
cdx diff doc1.cdx doc2.cdx
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
