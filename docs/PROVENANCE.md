# Inspect embedded Content Credentials

The optional `provenance` feature adds a CLI command that validates embedded C2PA Content Credentials on local files. It reports credential status and a small set of declared creation methods. The `detect` command independently examines pixels with Tintype’s model. Run both when you have the original file.

```sh
cargo build --release --locked --bin tintype --features provenance
target/release/tintype provenance image.png --json
target/release/tintype detect image.png --json
```

The credential check works without model weights. Provide PEM trust bundles for claim signatures and timestamps when you want to verify trust against a known authority. Obtain and update the bundles independently of the files under review. Use the C2PA Conformance Program’s [claim-signing and timestamp trust lists](https://github.com/c2pa-org/conformance-public/tree/main/trust-list), or your organization’s own approved bundles. Keep the two purposes separate.

```sh
target/release/tintype provenance image.png \
  --trust-anchors C2PA-TRUST-LIST.pem \
  --tsa-anchors C2PA-TSA-TRUST-LIST.pem \
  --json
```

The result describes what the validator found in the file.

| Status | Meaning |
|---|---|
| `trusted` | The embedded credential passed validation against the supplied trust policy. |
| `valid-untrusted` | Cryptographic validation passed and the signer could not be traced to a configured trust anchor. |
| `invalid` | The validator could not establish the credential’s integrity. The JSON report may include validation codes. |
| `absent` | The validator found no embedded credential. |
| `remote-only` | The file points to a credential elsewhere. The command leaves it unchecked. |
| `unsupported` | This file format could not be checked. |

When a validated active manifest explicitly declares a supported creation method, the result includes its `c2pa.created` action and `digital_source_type`. `trainedAlgorithmicMedia` declares generated media, `compositeWithTrainedAlgorithmicMedia` declares a composite with AI-generated elements, and `digitalCapture` declares digital capture. Read the declaration in light of the credential’s status. A trusted credential authenticates the signed statement and its association with the file. Use independent evidence to establish what happened in a scene. A missing credential leaves origin unresolved. [C2PA specification](https://spec.c2pa.org/specifications/specifications/2.5/specs/C2PA_Specification.html) · [IPTC source types](https://cv.iptc.org/newscodes/digitalsourcetype/)

## Same image, different files

Tintype’s demo is an original ImageGen illustration. [Download the untouched, signed PNG](assets/provenance-original.png). We checked that file and three variants locally on Apple silicon. The validated creation declaration on the original was `trainedAlgorithmicMedia`.

| File | Credential with C2PA trust bundles | Default CPU detection score |
|---|---|---:|
| Original ImageGen PNG | Trusted | 0.874, flagged |
| Original pixels with the credential chunk removed | Absent | 0.874, flagged |
| Resized 880 × 660 demo PNG | Absent | 0.298, missed |
| Original PNG with a signed byte altered | Invalid, `assertion.dataHash.mismatch` | — |

The original without configured trust anchors reports `valid-untrusted`. The original SHA-256 is `324bb86fbea4292280a786c7cb32fa46e8524ac134a319a3be8049840a5d0ee3`. The original image is available under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) to the extent possible. The embedded credentials are unchanged. The locally signed [verification fixtures](../tests/fixtures/provenance/) exercise trust policies in CI and use a test-only root. See [the launch guide](LAUNCH.md) for the generation prompt and resized sample. [OpenAI provenance documentation](https://help.openai.com/en/articles/8912793-c2pa-in-chatgpt-images)

## Scope

The verifier uses the [CAI Rust SDK](https://github.com/contentauth/c2pa-rs), licensed MIT or Apache-2.0, with Rust-native crypto and HTTP features disabled. It reads the original file stream, skips external manifests and sidecars, and leaves live revocation checks disabled. It uses embedded revocation and timestamp information where supported by the validator and local trust policy. The command reads embedded credentials in JPEG, PNG, WebP, GIF, TIFF (including DNG, ARW and NEF), JPEG XL containers, AVIF, HEIC/HEIF, MP4, MOV, M4V, and AVI. It accepts regular files up to 512 MiB and caps decompressed manifests at 8 MiB.

The CLI command adds no credential parsing to the HTTP request path. It handles claims in the active manifest. Keep the original file to retain any credential that an editor or service may remove.
