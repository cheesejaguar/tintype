# Credential verification fixtures

The two PNGs contain the same 8 × 8 procedural gradient. One manifest
declares digital capture and the other declares generated media. These
declarations exercise the reader’s interpretation of signed metadata.
The pixels are a synthetic test pattern in both files.

The files are signed by a locally generated ECDSA P-256 test chain. The
public root in `root.pem` is dedicated to these fixtures. It belongs in
tests only. The signing key is outside the repository. Both media fixtures
and the public root are available under CC0.
