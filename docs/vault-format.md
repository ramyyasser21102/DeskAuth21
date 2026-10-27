# Vault file format

The vault is one file, `vault.da21`. All integers are big-endian. Both versions start with
the four bytes `DA21` and a version byte. Anything else, or a version this build does not
know, is refused and never overwritten.

## Version 1 (vaults made before unlock slots)

```text
"DA21" | version u8 = 1 | memory_kib u32 | passes u32 | lanes u32 | salt[16] | nonce[24] | ciphertext
```

The data is encrypted straight under a key made from the passphrase (Argon2id with the
recorded cost and salt) using XChaCha20-Poly1305. Everything before the nonce is authenticated
as associated data. Such a vault keeps opening, and keeps saving in this format, until it is
upgraded.

## Version 2 (current)

```text
"DA21" | version u8 = 2 | slot_count u8 | slot... | payload_nonce[24] | ciphertext
slot = kind u8 | body_len u16 | body
```

The data is encrypted with XChaCha20-Poly1305 under a random 32-byte **vault key**. Everything
before `payload_nonce` (the preamble and every slot) is authenticated as associated data, so
removing or editing a slot makes the data fail to open.

Each **slot** holds the vault key wrapped (encrypted) under a key made from something the
person has, so a way of unlocking can be added or changed without re-encrypting the data.

| kind | meaning | body |
|---|---|---|
| 1 | passphrase | `memory_kib u32 | passes u32 | lanes u32 | salt[16] | nonce[24] | wrapped_vault_key[48]` |
| 2 | recovery key | `nonce[24] | wrapped_vault_key[48]` |
| other | not understood by this build | kept byte for byte when the vault is saved |

For a passphrase slot the wrapping key is `Argon2id(passphrase, salt, memory, passes, lanes)`,
and the wrapped key is `XChaCha20-Poly1305(wrapping key, nonce, vault key)` with associated
data `"DA21 slot" | kind | memory | passes | lanes | salt | nonce`, so changing the recorded
cost breaks the slot.

For a recovery slot the recovery key is 24 random words from the EFF large word list (about
310 bits), written separated by single spaces in lower case. The wrapping key is
`HMAC-SHA256(key = "DA21 recovery key v1", message = those words)`; no slow derivation is needed
because the words are random and very long. The wrapped key is
`XChaCha20-Poly1305(wrapping key, nonce, vault key)` with associated data
`"DA21 slot" | 2 | nonce`.

### Changing slots

Changing the passphrase, or adding or replacing the recovery key, rewrites slots and leaves the
vault key (and so the data) alone. Both require the current passphrase, even while the vault is
open. The recovery key opens the vault without the passphrase and then sets a new passphrase
slot.

### Limits a reader enforces

At most 8 slots, each body at most 512 bytes, and a slot's recorded key derivation cost no
higher than 256 MiB, 10 passes and 4 lanes, so a hostile file cannot make unlocking allocate
without bound or run for minutes.

## Upgrading from version 1

The session already holds the key it derived from the passphrase, so an upgrade needs no
passphrase: it picks a new random vault key and wraps it under that same key in a passphrase
slot with the same salt and cost. The same passphrase therefore keeps working. The old file is
copied beside the vault as `vault.da21.v1.bak` first (a name that does not end in `.da21`, so
it is never mistaken for a conflicted copy and merged), and if saving fails the vault stays as
it was.

## Sharing a vault between computers

Slots are not merged; the newest file's slots win. Whenever a computer reads a changed vault
file that opens with its own vault key, it adopts that file's slots before it next saves, so a
passphrase or recovery key changed on one computer is not undone when another saves a change of
its own. Conflicted copies are merged for their accounts only; their slots are ignored.

A computer that opens a version 1 file and later finds a version 2 file in the same place, or
the other way round, refuses to merge or overwrite it and asks the person to lock and unlock
again; it never guesses. A build that only knows version 1 refuses a version 2 file the same way.
