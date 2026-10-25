// Generates the KDBX test fixtures used by `crates/keepass-core/tests/g2_vault.rs`.
//
// Implementation: **kdbxweb** (JavaScript), deliberately independent of the Rust
// `keepass` crate the app uses. A fixture written and read by the same library
// proves only self-consistency, not correctness.
//
// Every generated file is additionally verified with `keepassxc-cli ls` and
// `keepassxc-cli show --all`, so each one is known-good to a third
// implementation too.
//
// Usage:
//   cd tools/fixtures
//   npm install
//   node gen.js
//
// Covers the version / cipher / KDF matrix and the document shapes the adapter
// must handle: attachments, history, custom icons, and the HarmonyOS private
// markers documented in `crates/keepass-core/src/interop.rs`.

const kdbxweb = require('kdbxweb');
const { argon2id, argon2d } = require('hash-wasm');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// tools/fixtures -> repo root -> crates/keepass-core/tests/fixtures
const OUT = path.resolve(__dirname, '..', '..', 'crates', 'keepass-core', 'tests', 'fixtures');
const PASSWORD = 'test-master-pw';

// kdbxweb 2.x has no bundled Argon2; the host must supply one. hash-wasm is a
// WASM Argon2, independent of any Rust implementation.
kdbxweb.CryptoEngine.setArgon2Impl(async (password, salt, memory, iterations, length, parallelism, type) => {
  const fn = type === kdbxweb.CryptoEngine.Argon2TypeArgon2id ? argon2id : argon2d;
  const bytes = await fn({
    password: new Uint8Array(password),
    salt: new Uint8Array(salt),
    memorySize: memory,
    iterations,
    parallelism,
    hashLength: length,
    outputType: 'binary',
  });
  return bytes;
});

function pw(s) {
  return kdbxweb.ProtectedValue.fromString(s);
}

function newDb(name, { version, cipher, kdf } = {}) {
  const db = kdbxweb.Kdbx.create(new kdbxweb.Credentials(pw(PASSWORD), null), name);
  if (version) db.header.version = version;
  db.header.setKdf(kdf || kdbxweb.Consts.KdfId.Argon2id);
  // kdbxweb has no setCipher(); the header field is the API.
  if (cipher) db.header.dataCipherUuid = new kdbxweb.KdbxUuid(cipher);
  return db;
}

function addEntry(db, group, title, username, password, opts = {}) {
  const e = db.createEntry(group);
  e.fields.set('Title', title);
  e.fields.set('UserName', username);
  e.fields.set('Password', pw(password));
  if (opts.url) e.fields.set('URL', opts.url);
  if (opts.notes) e.fields.set('Notes', opts.notes);
  if (opts.otp) e.fields.set('otp', opts.otp);
  if (opts.tags) e.tags = opts.tags;
  for (const [k, v] of Object.entries(opts.customFields || {})) {
    e.fields.set(k, v.protected ? pw(v.value) : v.value);
  }
  // Entry-level customData is a Map on the entry, NOT a field. kdbxweb models
  // the KDBX <CustomData><Item> block separately from <String> fields.
  if (opts.customData) {
    e.customData = new Map();
    for (const [k, v] of Object.entries(opts.customData)) {
      e.customData.set(k, { value: v });
    }
  }
  return e;
}

/// createBinary returns a Promise — it hashes the bytes asynchronously. Awaiting
/// it before save is mandatory, otherwise the entry holds an unresolved ref and
/// writeBinaries throws.
async function attach(db, entry, files) {
  for (const [name, bytes] of Object.entries(files)) {
    const ref = await db.createBinary(kdbxweb.ProtectedValue.fromBinary(new Uint8Array(bytes)));
    entry.binaries.set(name, ref);
  }
}

async function write(db, file) {
  const data = await db.save();
  const buf = Buffer.from(new Uint8Array(data));
  fs.writeFileSync(path.join(OUT, file), buf);
  console.log('wrote', file, buf.length, 'bytes');
}

(async () => {
  fs.mkdirSync(OUT, { recursive: true });

  // 1. KDBX 4.0, AES-256, Argon2id, password only.
  //    Kept at 4.0 on purpose: this is the "old client" case. The Rust adapter
  //    upgrades it to 4.1 on save because `keepass` cannot write 4.0 at all.
  {
    const db = newDb('V4 AES Argon2id', { version: 0x00040000 });
    addEntry(db, db.getDefaultGroup(), 'Example Login', 'alice', 'correct horse battery staple', {
      url: 'https://example.com',
      notes: '多字节注释用于验证 CJK 往返',
      otp: 'otpauth://totp/Example:alice?secret=JBSWY3DPEHPK3PXP&issuer=Example',
      customData: {
        // A marker from a client we know nothing about. It must survive every
        // save we perform (see round_trip_preserves_foreign_custom_data).
        'KPXC_unknown': 'leave me alone',
      },
    });
    await write(db, 'v4-aes-argon2id.kdbx');
  }

  // 2. KDBX 4.1, AES-256, Argon2id.
  {
    const db = newDb('V41 AES Argon2id', { version: 0x00040001 });
    addEntry(db, db.getDefaultGroup(), 'V41 Entry', 'v41user', 'v41-secret');
    await write(db, 'v4.1-aes-argon2id.kdbx');
  }

  // 3. KDBX 4.0, ChaCha20, Argon2id.
  {
    const db = newDb('V4 ChaCha20', { cipher: kdbxweb.Consts.CipherId.ChaCha20 });
    addEntry(db, db.getDefaultGroup(), 'ChaCha Entry', 'chachauser', 'chacha-secret');
    await write(db, 'v4-chacha20-argon2id.kdbx');
  }

  // 4. KDBX 3.1, AES-256, AES-KDF.
  {
    const db = newDb('V31 AES-KDF', { version: 0x00030001, kdf: kdbxweb.Consts.KdfId.Aes });
    addEntry(db, db.getDefaultGroup(), 'V31 Entry', 'v31user', 'v31-secret');
    await write(db, 'v3.1-aes-aeskdf.kdbx');
  }

  // 5. Attachments, including bytes that must survive the round trip exactly.
  {
    const db = newDb('Attachments');
    const e = addEntry(db, db.getDefaultGroup(), 'With Files', 'attuser', 'att-secret');
    await attach(db, e, {
      'note.txt': new TextEncoder().encode('attachment body\n').buffer,
      'binary.bin': new Uint8Array([0, 1, 2, 253, 254, 255]).buffer,
    });
    await write(db, 'v4-attachments.kdbx');
  }

  // 6. History: three revisions of the same entry.
  {
    const db = newDb('History');
    const g = db.getDefaultGroup();
    const e = addEntry(db, g, 'History Entry', 'histuser', 'version-1');
    e.pushHistory();
    e.fields.set('Password', pw('version-2'));
    e.pushHistory();
    e.fields.set('Password', pw('version-3'));
    await write(db, 'v4-history.kdbx');
  }

  // 7. Custom icon. entry.icon must be 69 (CUSTOM_ICON_ID) or KeePass ignores
  //    the customIcon reference; the HarmonyOS client does the same.
  //
  //    KDBX 4.1 is required: 4.0 stores custom icons inside <Meta> too, but ICU
  //    name/lastModified and the entry reference behave consistently only on 4.1,
  //    and `keepass` can only ever write 4.1 back.
  {
    const db = newDb('Icons', { version: 0x00040001 });
    const e = addEntry(db, db.getDefaultGroup(), 'Icon Entry', 'iconuser', 'icon-secret');
    const png = Buffer.from(
      '89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d4944415478da63fcffff3f030005fe02fea735c24f0000000049454e44ae426082',
      'hex',
    );
    // meta.customIcons is Map<uuidString, {data, name, lastModified}>.
    // KdbxUuid() with NO argument produces the all-zero UUID (kdbxweb docs say
    // "random", the source disagrees), which KeePassXC rejects as "missing icon
    // UUID". Pass 16 real random bytes.
    const uuidBytes = crypto.randomBytes(16);
    const iconUuid = new kdbxweb.KdbxUuid(
      uuidBytes.buffer.slice(uuidBytes.byteOffset, uuidBytes.byteOffset + 16),
    );
    db.meta.customIcons.set(iconUuid.id, { data: new Uint8Array(png) });
    e.customIcon = iconUuid;
    e.icon = 69;
    await write(db, 'v4-custom-icons.kdbx');
  }

  // 8. HarmonyOS private markers, SshKeys JSON, template identity.
  //
  //    NOTE on marker values: `KeePassHomo.Favorite` is normally the literal
  //    "true". `keepass` 0.17 mis-parses that as base64 (see
  //    docs/architecture.md §13), so this fixture uses values that are not
  //    valid base64. The real-world "true" case is covered by the separate
  //    `v4-homo-favorite-true.kdbx` fixture and a test that pins the defect.
  {
    const db = newDb('Homo Markers');
    const g = db.getDefaultGroup();
    addEntry(db, g, 'Homo Login', 'huser', 'hpass', {
      tags: ['work'],
      customData: {
        'KeePassHomo.Favorite': 'yes',
        'KeePassHomo.Archived': 'no',
        'KeePassHomo.FieldTypes': '{"URL":"url","Notes":"custom"}',
        KPEX_LayoutType: 'Login',
      },
    });
    addEntry(db, g, 'Homo SSH', 'sshuser', 'ssh-secret', {
      customFields: {
        SshKeys: {
          value: '{"version":1,"keys":[{"name":"default","algorithm":"ed25519","bits":256,"addedAt":1750000000000,"privateKey":"redacted","publicKey":"ssh-ed25519 AAAA"}]}',
          protected: true,
        },
      },
      customData: {
        KPEX_LayoutType: 'SshKey',
      },
    });
    await write(db, 'v4-homo-markers.kdbx');
  }

  // 8b. The real-world HarmonyOS marker value, kept separate so the upstream
  //     defect stays visible instead of being hidden by a friendlier fixture.
  {
    const db = newDb('Homo Favorite True');
    addEntry(db, db.getDefaultGroup(), 'True Favorite', 'u', 'p', {
      customData: {
        'KeePassHomo.Favorite': 'true',
      },
    });
    await write(db, 'v4-homo-favorite-true.kdbx');
  }

  // 9. Key-file-only vault (no password).
  {
    const keyfilePath = path.join(OUT, 'test.keyx');
    fs.writeFileSync(keyfilePath, crypto.randomBytes(32));
    const keyfileBytes = new Uint8Array(fs.readFileSync(keyfilePath));
    const db = kdbxweb.Kdbx.create(new kdbxweb.Credentials(null, keyfileBytes), 'KeyFile Vault');
    db.header.setKdf(kdbxweb.Consts.KdfId.Argon2id);
    addEntry(db, db.getDefaultGroup(), 'KeyFile Entry', 'kfuser', 'kf-secret');
    await write(db, 'v4-aes-argon2id-keyfile.kdbx');
  }

  console.log('done');
})().catch((e) => {
  console.error('FAILED', e);
  process.exit(1);
});
